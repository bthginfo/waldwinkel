use std::{
    fs, io,
    path::{Path, PathBuf},
};

use bevy::{
    prelude::*,
    window::{AppLifecycle, WindowCloseRequested, WindowFocused},
};
use serde::{Deserialize, Serialize};

use crate::{
    Controls, GardenFocus, Notice, Ready,
    adventure::{Adventure, Area},
    garden::{Garden, SavePath},
    input::Gesture,
    town::Dialog,
    travel::District,
    ui::{self, UiFonts},
    world::Gardener,
};

const SLOT_COUNT: usize = 3;
const SAVE_INTERVAL: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub area: Area,
    pub position: [f32; 2],
}

impl Checkpoint {
    pub fn new(area: Area, position: Vec2) -> Self {
        Self {
            area,
            position: position.to_array(),
        }
    }

    pub fn move_to_mainland(self) -> Self {
        let old = Vec2::from_array(self.position);
        match self.area {
            Area::Island | Area::Expedition(_) => self,
            Area::District(district) => Self::new(
                Area::Island,
                district.origin() + (old - district.former_origin()),
            ),
            Area::Home => Self::new(self.area, self.area.origin() + (old - Vec2::new(85., 0.))),
            Area::Mine(_) => Self::new(self.area, self.area.origin() + (old - Vec2::new(160., 0.))),
        }
    }

    fn place(self) -> &'static str {
        if self.area == Area::Island {
            District::at(Vec2::from_array(self.position))
                .map_or_else(|| self.area.name(), District::name)
        } else {
            self.area.name()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SlotStatus {
    Empty,
    Saved {
        day: u32,
        place: &'static str,
        quests: usize,
    },
    Recoverable {
        day: u32,
        place: &'static str,
    },
    Unreadable,
}

impl SlotStatus {
    fn details(&self) -> String {
        match *self {
            Self::Empty => "Freier Platz".into(),
            Self::Saved { day, place, quests } => format!("Tag {day} · {place} · Aufträge {quests}/6"),
            Self::Recoverable { day, place } => format!("Sicherung · Tag {day} · {place}"),
            Self::Unreadable => "Keine lesbare Sicherung gefunden.".into(),
        }
    }
}

#[derive(Resource)]
pub struct SaveSlots {
    files: [PathBuf; SLOT_COUNT],
    legacy: PathBuf,
    status: [SlotStatus; SLOT_COUNT],
    note: String,
}

impl SaveSlots {
    pub fn new(legacy: PathBuf) -> Self {
        let files = std::array::from_fn(|index| {
            legacy.with_file_name(format!("spielstand-{}.json", index + 1))
        });
        let mut note = String::new();
        let inherited = (|| -> io::Result<bool> {
            if files[0].try_exists()?
                || Garden::backup_path(&files[0]).try_exists()?
                || !legacy.try_exists()?
            {
                return Ok(false);
            }
            Garden::load(&legacy)?.save(&files[0])?;
            Ok(true)
        })();
        let mut status = files.each_ref().map(|file| Self::inspect(file));
        match inherited {
            Ok(true) => note = "Dein bisheriger Hof wurde in Spielstand 1 übernommen.".into(),
            Ok(false) => {}
            Err(error) => {
                warn!("Übernahme des bisherigen Spielstands fehlgeschlagen: {error}");
                status[0] = SlotStatus::Unreadable;
                note = "Der bisherige Spielstand konnte nicht gelesen werden.".into();
            }
        }
        Self {
            files,
            legacy,
            status,
            note,
        }
    }

    fn inspect(file: &Path) -> SlotStatus {
        let error = match Garden::load(file) {
            Ok(garden) => return Self::summary(&garden),
            Err(error) => error,
        };
        match Garden::load(&Garden::backup_path(file)) {
            Ok(garden) => SlotStatus::Recoverable {
                day: garden.day,
                place: garden.checkpoint.map_or("Waldwinkel", Checkpoint::place),
            },
            Err(backup)
                if error.kind() == io::ErrorKind::NotFound
                    && backup.kind() == io::ErrorKind::NotFound =>
            {
                SlotStatus::Empty
            }
            Err(_) => SlotStatus::Unreadable,
        }
    }

    fn delete(&mut self, index: usize) -> io::Result<()> {
        let file = self.files[index].as_path();
        let backup = Garden::backup_path(file);
        let legacy_backup = Garden::backup_path(&self.legacy);
        let legacy_files = (index == 0).then_some([self.legacy.as_path(), legacy_backup.as_path()]);
        let copies = [file, backup.as_path()]
            .into_iter()
            .chain(legacy_files.into_iter().flatten());
        // Remove the playable copy last so a cleanup error leaves it available.
        // The old single-slot file must also go, or startup would reimport it.
        let last = match Self::inspect(file) {
            SlotStatus::Saved { .. } => file,
            SlotStatus::Recoverable { .. } => backup.as_path(),
            _ if index == 0 => self.legacy.as_path(),
            _ => file,
        };
        for path in copies.clone() {
            remove_if_present(&Garden::temporary_path(path))?;
        }
        remove_if_present(&file.with_extension("beschaedigt.json"))?;
        for path in copies.filter(|&path| path != last) {
            remove_if_present(path)?;
        }
        remove_if_present(last)?;
        self.status[index] = SlotStatus::Empty;
        if index == 0 {
            self.note.clear();
        }
        Ok(())
    }

    fn summary(garden: &Garden) -> SlotStatus {
        let place = garden
            .checkpoint
            .map_or_else(|| Area::Island.name(), Checkpoint::place);
        SlotStatus::Saved {
            day: garden.day,
            place,
            quests: garden.town.total_completed(),
        }
    }

    fn open(&mut self, index: usize) -> io::Result<Garden> {
        let file = &self.files[index];
        if matches!(Self::inspect(file), SlotStatus::Recoverable { .. }) {
            let garden = Garden::load(&Garden::backup_path(file))?;
            let damaged = file.with_extension("beschaedigt.json");
            if file.try_exists()? && !damaged.try_exists()? {
                fs::copy(file, damaged)?;
            }
            garden.save(file)?;
            self.status[index] = Self::summary(&garden);
            return Ok(garden);
        }
        if matches!(self.status[index], SlotStatus::Unreadable) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Dieser Spielstand ist nicht lesbar.",
            ));
        }
        let garden = match Garden::load(&self.files[index]) {
            Ok(garden) => garden,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let mut garden = Garden::default();
                garden.prologue = Some(Default::default());
                garden.save(&self.files[index])?;
                garden
            }
            Err(error) => {
                self.status[index] = SlotStatus::Unreadable;
                return Err(error);
            }
        };
        self.status[index] = Self::summary(&garden);
        Ok(garden)
    }

    #[cfg(any(target_os = "android", test))]
    fn import(&mut self, slot: usize, data: &[u8]) -> io::Result<()> {
        let file = &self.files[slot];
        if !matches!(Self::inspect(file), SlotStatus::Empty) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Der Platz ist belegt.",
            ));
        }
        let garden = Garden::decode(data)?;
        garden.save(file)?;
        self.status[slot] = Self::summary(&garden);
        Ok(())
    }
}

fn remove_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Title,
    Playing,
    Pause,
    ConfirmDelete(usize),
}

impl Screen {
    fn menu(self) -> Option<MenuScreen> {
        match self {
            Self::Title => Some(MenuScreen::Title),
            Self::Pause => Some(MenuScreen::Pause),
            Self::ConfirmDelete(_) => Some(MenuScreen::ConfirmDelete),
            Self::Playing => None,
        }
    }
}

#[derive(Resource, Default)]
pub struct Session {
    screen: Screen,
    index: Option<usize>,
    message: String,
    last_saved: f32,
    last_clock: u16,
    save_failed: bool,
    #[cfg(target_os = "android")]
    importing: bool,
    #[cfg(target_os = "android")]
    document: Option<DocumentTask>,
}

#[cfg(target_os = "android")]
#[derive(Clone, Copy)]
enum DocumentTask {
    Export,
    Import(usize),
}

impl Session {
    pub fn playing(&self) -> bool {
        self.screen == Screen::Playing
    }

    pub fn slot(&self) -> Option<usize> {
        self.index
    }

    #[cfg(target_os = "android")]
    pub fn confirming_delete(&self) -> bool {
        matches!(self.screen, Screen::ConfirmDelete(_))
    }

    fn choosing_slot(&self) -> bool {
        #[cfg(target_os = "android")]
        if self.importing || self.document.is_some() {
            return false;
        }
        self.screen == Screen::Title
    }

    fn return_to_title(&mut self) {
        self.screen = Screen::Title;
        self.index = None;
        self.message.clear();
    }

    pub(crate) fn save(
        &mut self,
        slots: &mut SaveSlots,
        garden: &mut Garden,
        point: Checkpoint,
        now: f32,
    ) -> io::Result<()> {
        let Some(index) = self.index else {
            return Ok(());
        };
        let previous = garden.checkpoint.replace(point);
        self.last_saved = now;
        let result = garden.save(&slots.files[index]);
        self.save_failed = result.is_err();
        if let Err(error) = result {
            garden.checkpoint = previous;
            return Err(error);
        }
        self.last_clock = garden.clock.minute();
        slots.status[index] = SaveSlots::summary(garden);
        Ok(())
    }
}

pub fn playing(
    session: Res<Session>,
    dialog: Res<Dialog>,
    farm_menu: Res<crate::farm_ui::FarmMenu>,
    wheel: Res<crate::wheel::Wheel>,
    night: Res<crate::night::Night>,
    layout: Res<crate::decorate::Layout>,
) -> bool {
    session.playing() && !dialog.open() && !farm_menu.open()
        && !wheel.open() && !night.active() && !layout.open()
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum MenuScreen {
    Title,
    Pause,
    ConfirmDelete,
}

#[derive(Component, Clone, Copy)]
pub enum MenuButton {
    Slot(usize),
    DeleteSlot(usize),
    CancelDelete,
    ConfirmDelete,
    Resume,
    Save,
    Title,
    UiSize,
    Sound,
    #[cfg(target_os = "android")]
    Import,
    #[cfg(target_os = "android")]
    Export,
}

#[derive(Component)]
pub enum MenuText {
    TitlePrompt,
    SlotMeta(usize),
    SlotAction(usize),
    DeleteTitle,
    DeleteDetails,
    Message(MenuScreen),
    ActiveSlot,
    PadHelp(MenuScreen),
    UiSize,
    Sound,
    #[cfg(target_os = "android")]
    Import,
}

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>) {
    use crate::controller::{Panel, SecondaryFocus};

    for screen in [MenuScreen::Title, MenuScreen::Pause, MenuScreen::ConfirmDelete] {
        commands
            .spawn((
                screen,
                if screen == MenuScreen::ConfirmDelete {
                    Panel::ConfirmDelete
                } else {
                    Panel::Session
                },
                Node {
                    display: if screen == MenuScreen::Title {
                        Display::Flex
                    } else {
                        Display::None
                    },
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.087, 0.173, 0.15, 0.63)),
                GlobalZIndex(100),
            ))
            .with_children(|overlay| {
                overlay
                    .spawn((
                        Node {
                            width: Val::Px(match screen {
                                MenuScreen::Title => 566.,
                                MenuScreen::Pause => 398.,
                                MenuScreen::ConfirmDelete => 444.,
                            }),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Stretch,
                            row_gap: Val::Px(10.),
                            padding: UiRect::all(Val::Px(24.)),
                            border: UiRect::all(Val::Px(1.)),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(26.)),
                        BackgroundColor(ui::cream()),
                        BorderColor(ui::stroke()),
                        ui::shadow(),
                    ))
                    .with_children(|panel| {
                        if screen == MenuScreen::Title {
                            panel.spawn(fonts.title("Waldwinkel", 39.));
                            panel.spawn((
                                MenuText::TitlePrompt,
                                fonts.body("Wähle deinen Spielstand.", 16.),
                            ));
                            for index in 0..SLOT_COUNT {
                                spawn_slot(panel, &fonts, index);
                            }
                            #[cfg(target_os = "android")]
                            panel
                                .spawn((MenuButton::Import, ui::dialog_button(236.)))
                                .with_child((
                                    MenuText::Import,
                                    fonts.button("Spielstand importieren", 16.),
                                ));
                        } else if screen == MenuScreen::Pause {
                            panel.spawn(fonts.title("Spiel pausiert", 39.));
                            panel.spawn((MenuText::ActiveSlot, fonts.body("", 15.)));
                            for (action, label) in [
                                (MenuButton::Resume, "Weiterspielen"),
                                (MenuButton::Save, "Jetzt speichern"),
                                (MenuButton::UiSize, "Darstellung"),
                                (MenuButton::Sound, "Ton"),
                                (MenuButton::Title, "Speichern & Hauptmenü"),
                            ] {
                                let mut button = panel.spawn((action, ui::dialog_button(350.)));
                                if matches!(action, MenuButton::UiSize) {
                                    button.with_child((MenuText::UiSize, fonts.button(label, 16.)));
                                } else if matches!(action, MenuButton::Sound) {
                                    button.with_child((MenuText::Sound, fonts.button(label, 16.)));
                                } else {
                                    button.with_child(fonts.button(label, 16.));
                                }
                            }
                            #[cfg(target_os = "android")]
                            panel
                                .spawn((MenuButton::Export, ui::dialog_button(350.)))
                                .with_child(fonts.button("Spielstand exportieren", 16.));
                        } else {
                            panel.spawn((MenuText::DeleteTitle, fonts.title("", 33.)));
                            panel.spawn((MenuText::DeleteDetails, fonts.bold("", 15.)));
                            panel.spawn(fonts.body(
                                "Der Fortschritt und die lokale Sicherung werden dauerhaft gelöscht.",
                                16.,
                            ));
                            panel
                                .spawn(Node {
                                    column_gap: Val::Px(10.),
                                    justify_content: JustifyContent::SpaceBetween,
                                    ..default()
                                })
                                .with_children(|row| {
                                    row.spawn((MenuButton::CancelDelete, ui::dialog_button(192.)))
                                        .with_child(fonts.button("Abbrechen", 16.));
                                    row.spawn((
                                        MenuButton::ConfirmDelete,
                                        SecondaryFocus,
                                        ui::dialog_button(192.),
                                    ))
                                    .with_child(fonts.button("Endgültig löschen", 16.));
                                });
                        }
                        panel.spawn((MenuText::Message(screen), fonts.body("", 13.)));
                        panel.spawn((MenuText::PadHelp(screen), fonts.body("", 13.)));
                    });
            });
    }
}

fn spawn_slot(panel: &mut ChildSpawnerCommands, fonts: &UiFonts, index: usize) {
    panel
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Px(8.),
                padding: UiRect::axes(Val::Px(13.), Val::Px(7.)),
                min_height: Val::Px(60.),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(241, 229, 198)),
            BorderRadius::all(Val::Px(15.)),
        ))
        .with_children(|row| {
            row.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(1.),
                ..default()
            })
            .with_children(|labels| {
                labels.spawn(fonts.bold(&format!("Spielstand {}", index + 1), 18.));
                labels.spawn((MenuText::SlotMeta(index), fonts.body("Frei", 13.)));
            });
            row.spawn(Node {
                column_gap: Val::Px(8.),
                flex_shrink: 0.,
                ..default()
            })
            .with_children(|actions| {
                actions
                    .spawn((MenuButton::Slot(index), ui::dialog_button(146.)))
                    .with_child((MenuText::SlotAction(index), fonts.button("Neues Spiel", 15.)));
                actions
                    .spawn((
                        MenuButton::DeleteSlot(index),
                        crate::controller::SecondaryFocus,
                        Visibility::Hidden,
                        ui::dialog_button(88.),
                    ))
                    .with_child(fonts.bold("Löschen", 14.));
            });
        });
}

pub fn input(
    buttons: Query<(&MenuButton, &Interaction), Changed<Interaction>>,
    keys: Res<ButtonInput<KeyCode>>,
    layers: (Res<Ready>, Res<crate::wheel::Wheel>, Res<crate::night::Night>, Res<crate::decorate::Layout>),
    time: Res<Time>,
    mut session: ResMut<Session>,
    mut slots: ResMut<SaveSlots>,
    mut g: ResMut<Garden>,
    mut path: ResMut<SavePath>,
    mut adventure: ResMut<Adventure>,
    mut player: Single<(&mut Gardener, &mut Transform)>,
    mut gesture: ResMut<Gesture>,
    mut controls: ResMut<Controls>,
    mut focus: ResMut<GardenFocus>,
    mut notice: ResMut<Notice>,
    mut dialog: ResMut<Dialog>,
    mut farm_menu: ResMut<crate::farm_ui::FarmMenu>,
) {
    let (ready, wheel, night, layout) = layers;
    if wheel.open() || night.active() || layout.open() {
        return;
    }
    #[cfg(target_os = "android")]
    if session.document.is_some() {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        if matches!(session.screen, Screen::ConfirmDelete(_)) {
            session.return_to_title();
            return;
        }
        #[cfg(target_os = "android")]
        if session.screen == Screen::Title && session.importing {
            session.importing = false;
            session.message.clear();
            return;
        }
        if farm_menu.battle().is_some() {
            return;
        }
        if farm_menu.open() {
            if matches!(farm_menu.clip(), Some((crate::cinematic::Clip::Arrival, _)))
                && g.onboarding == crate::onboarding::Onboarding::Welcome
            {
                g.onboarding = crate::onboarding::Onboarding::Till;
                crate::adventure::commit(
                    &g,
                    &path,
                    &mut notice,
                    time.elapsed_secs(),
                    "Prolog gestartet · Dein erster Morgen".into(),
                    None,
                    None,
                );
            }
            farm_menu.back();
            *gesture = Gesture::default();
            *controls = Controls::default();
            return;
        }
        if dialog.open() {
            dialog.back();
            *gesture = Gesture::default();
            *controls = Controls::default();
            return;
        }
        if session.screen == Screen::Playing {
            session.screen = Screen::Pause;
            session.message.clear();
        } else if session.screen == Screen::Pause {
            session.screen = Screen::Playing;
        }
        *gesture = Gesture::default();
        *controls = Controls::default();
        return;
    }
    let clicked = buttons.iter().find_map(|(button, interaction)| {
        (*interaction == Interaction::Pressed).then_some(*button)
    });
    let shortcut = match session.screen {
        Screen::Title => [
            (KeyCode::Digit1, 0),
            (KeyCode::Digit2, 1),
            (KeyCode::Digit3, 2),
        ]
        .into_iter()
        .find_map(|(key, index)| keys.just_pressed(key).then_some(MenuButton::Slot(index))),
        Screen::Pause if keys.just_pressed(KeyCode::KeyS) => Some(MenuButton::Save),
        _ => None,
    };
    let Some(action) = clicked.or(shortcut) else {
        return;
    };
    let (player, transform) = &mut *player;
    match (session.screen, action) {
        (Screen::Title, MenuButton::DeleteSlot(index))
            if ready.0 && session.choosing_slot() && slots.status[index] != SlotStatus::Empty =>
        {
            session.screen = Screen::ConfirmDelete(index);
            session.message.clear();
        }
        (Screen::ConfirmDelete(_), MenuButton::CancelDelete) => session.return_to_title(),
        (Screen::ConfirmDelete(index), MenuButton::ConfirmDelete) => match slots.delete(index) {
            Ok(()) => {
                session.return_to_title();
                session.message = format!("Spielstand {} gelöscht. Der Platz ist wieder frei.", index + 1);
            }
            Err(error) => {
                warn!("Spielstand {} konnte nicht gelöscht werden: {error}", index + 1);
                session.message = "Löschen fehlgeschlagen. Bitte versuche es erneut.".into();
            }
        },
        #[cfg(target_os = "android")]
        (Screen::Title, MenuButton::Import) if ready.0 => {
            session.importing = !session.importing;
            session.message.clear();
        }
        #[cfg(target_os = "android")]
        (Screen::Title, MenuButton::Slot(index)) if ready.0 && session.importing => {
            if matches!(slots.status[index], SlotStatus::Empty) {
                match crate::mobile::open_document(index, None) {
                    Ok(()) => session.document = Some(DocumentTask::Import(index)),
                    Err(error) => {
                        warn!("Importdialog konnte nicht geöffnet werden: {error}");
                        session.message = "Die Dateiauswahl konnte nicht geöffnet werden.".into();
                    }
                }
            } else {
                session.message = "Wähle zum Importieren einen freien Platz.".into();
            }
        }
        #[cfg(target_os = "android")]
        (Screen::Pause, MenuButton::Export) => {
            let Some(index) = session.index else { return };
            let checkpoint = Checkpoint::new(adventure.area, player.position);
            let result = session
                .save(&mut slots, &mut g, checkpoint, time.elapsed_secs())
                .and_then(|_| serde_json::to_vec(&*g).map_err(io::Error::from))
                .and_then(|bytes| {
                    crate::mobile::open_document(index, Some(&bytes)).map_err(io::Error::other)
                });
            match result {
                Ok(()) => session.document = Some(DocumentTask::Export),
                Err(error) => {
                    warn!("Export fehlgeschlagen: {error}");
                    session.message = "Der Spielstand konnte nicht exportiert werden.".into();
                }
            }
        }
        (Screen::Title, MenuButton::Slot(index)) if ready.0 => match slots.open(index) {
            Ok(garden) => {
                let checkpoint = garden.checkpoint;
                *g = garden;
                path.0.clone_from(&slots.files[index]);
                adventure.restore(checkpoint, player, transform, &g);
                *gesture = Gesture::default();
                *controls = Controls::default();
                *focus = GardenFocus::default();
                *notice = Notice::default();
                dialog.close();
                farm_menu.close();
                if g.onboarding == crate::onboarding::Onboarding::Welcome {
                    farm_menu.show(crate::farm_ui::FarmPage::Cinematic(
                        crate::cinematic::Clip::Arrival,
                    ));
                }
                session.screen = Screen::Playing;
                session.index = Some(index);
                session.last_saved = time.elapsed_secs();
                session.last_clock = g.clock.minute();
                session.save_failed = false;
                session.message.clear();
            }
            Err(error) => {
                warn!("Spielstand {} nicht geöffnet: {error}", index + 1);
                session.message = format!("Spielstand {} konnte nicht geöffnet werden.", index + 1);
            }
        },
        (Screen::Pause, MenuButton::Resume) => {
            session.screen = Screen::Playing;
        }
        (Screen::Pause, MenuButton::UiSize | MenuButton::Sound) => {
            if matches!(action, MenuButton::Sound) {
                (g.sound_muted, g.music_muted) = match (g.sound_muted, g.music_muted) {
                    (true, _) => (false, false),
                    (false, false) => (false, true),
                    (false, true) => (true, false),
                };
            } else {
                g.large_ui = !g.large_ui;
            }
            let checkpoint = Checkpoint::new(adventure.area, player.position);
            if let Err(error) = session.save(&mut slots, &mut g, checkpoint, time.elapsed_secs()) {
                warn!("Darstellung konnte nicht gespeichert werden: {error}");
                session.message = "Darstellung gilt bis zum nächsten Speichern.".into();
            }
        }
        (Screen::Pause, MenuButton::Save | MenuButton::Title) => {
            let checkpoint = Checkpoint::new(adventure.area, player.position);
            match session.save(&mut slots, &mut g, checkpoint, time.elapsed_secs()) {
                Ok(()) if matches!(action, MenuButton::Title) => {
                    session.return_to_title();
                    slots.note.clear();
                }
                Ok(()) => {
                    session.message = "Spielstand gespeichert.".into();
                    *notice = crate::feedback::make(crate::feedback::Tone::Success, session.message.clone(), time.elapsed_secs());
                }
                Err(error) => {
                    warn!("Spielstand konnte nicht gespeichert werden: {error}");
                    session.message = "Speichern fehlgeschlagen. Bitte erneut versuchen.".into();
                    *notice = crate::feedback::make(crate::feedback::Tone::Problem, session.message.clone(), time.elapsed_secs());
                }
            }
        }
        _ => {}
    }
}

#[cfg(target_os = "android")]
pub fn finish_document(mut session: ResMut<Session>, mut slots: ResMut<SaveSlots>) {
    let Some(task) = session.document else { return };
    let payload = match crate::mobile::document_result() {
        Ok(None) => return,
        Ok(Some(bytes)) => bytes,
        Err(error) => {
            warn!("Dateiauswahl fehlgeschlagen: {error}");
            vec![0]
        }
    };
    session.document = None;
    session.message = match (task, payload.split_first()) {
        (_, None) => "Dateiauswahl abgebrochen.".into(),
        (DocumentTask::Export, Some((1, _))) => "Spielstand als Datei exportiert.".into(),
        (DocumentTask::Import(slot), Some((1, bytes))) => match slots.import(slot, bytes) {
            Ok(()) => {
                session.importing = false;
                format!("Sicherung in Spielstand {} importiert.", slot + 1)
            }
            Err(error) => {
                warn!("Sicherung konnte nicht importiert werden: {error}");
                "Die Sicherung ist nicht lesbar oder der Platz ist belegt.".into()
            }
        },
        _ => "Die Datei konnte nicht gelesen oder geschrieben werden.".into(),
    };
}

pub fn open_menu(
    controls: Res<Controls>,
    mut session: ResMut<Session>,
    mut gesture: ResMut<Gesture>,
) {
    if controls.menu && session.playing() {
        session.screen = Screen::Pause;
        session.message.clear();
        *gesture = Gesture::default();
    }
}

pub fn autosave(
    mut session: ResMut<Session>,
    mut slots: ResMut<SaveSlots>,
    mut garden: ResMut<Garden>,
    adventure: Res<Adventure>,
    player: Single<&Gardener>,
    time: Res<Time>,
    mut focus: EventReader<WindowFocused>,
    mut lifecycle: EventReader<AppLifecycle>,
    mut closing: EventReader<WindowCloseRequested>,
) {
    let background = focus.read().any(|e| !e.focused);
    let suspended = lifecycle
        .read()
        .any(|e| matches!(e, AppLifecycle::WillSuspend | AppLifecycle::Suspended));
    let close = closing.read().next().is_some();
    if session.index.is_none() {
        return;
    }
    let now = time.elapsed_secs();
    let checkpoint = Checkpoint::new(adventure.area, player.position);
    let changed_area = garden
        .checkpoint
        .is_none_or(|p| p.area != adventure.area || p.place() != checkpoint.place());
    let moved = garden
        .checkpoint
        .is_none_or(|p| Vec2::from_array(p.position).distance_squared(player.position) > 0.35);
    let clock_advanced = garden.clock.minute().abs_diff(session.last_clock) >= 25;
    if (background
        || suspended
        || close
        || changed_area && !session.save_failed
        || now - session.last_saved >= SAVE_INTERVAL
            && (moved || clock_advanced || session.save_failed))
        && let Err(error) = session.save(&mut slots, &mut garden, checkpoint, now)
    {
        warn!("Automatisches Speichern fehlgeschlagen: {error}");
    }
}

pub fn update(
    session: Res<Session>,
    controller: Res<crate::controller::Controller>,
    slots: Res<SaveSlots>,
    garden: Res<Garden>,
    ready: Res<Ready>,
    mut overlay: Query<(&MenuScreen, &mut Node)>,
    mut texts: Query<(&MenuText, &mut Text)>,
    mut buttons: Query<(
        &MenuButton,
        &Interaction,
        &mut BackgroundColor,
        &mut crate::controller::FocusEnabled,
        &mut Visibility,
    )>,
) {
    let current = session.screen.menu();
    for (screen, mut node) in &mut overlay {
        let display = if Some(*screen) == current {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    for (query, mut text) in &mut texts {
        let value = match *query {
            MenuText::TitlePrompt => {
                #[cfg(target_os = "android")]
                if session.importing {
                    text.0 = "Wähle einen freien Platz für die Sicherung.".into();
                    continue;
                }
                "Wähle deinen Spielstand.".into()
            }
            #[cfg(target_os = "android")]
            MenuText::Import => if session.importing {
                "Zurück"
            } else {
                "Spielstand importieren"
            }
            .into(),
            #[cfg(target_os = "android")]
            MenuText::SlotAction(i)
                if session.importing && slots.status[i] == SlotStatus::Empty =>
            {
                "Hier importieren".into()
            }
            MenuText::SlotAction(i) => match slots.status[i] {
                SlotStatus::Empty => "Neues Spiel",
                SlotStatus::Saved { .. } => "Fortsetzen",
                SlotStatus::Recoverable { .. } => "Wiederherstellen",
                SlotStatus::Unreadable => "Nicht lesbar",
            }
            .into(),
            MenuText::SlotMeta(i) => slots.status[i].details(),
            MenuText::DeleteTitle => match session.screen {
                Screen::ConfirmDelete(i) => format!("Spielstand {} löschen?", i + 1),
                _ => continue,
            },
            MenuText::DeleteDetails => match session.screen {
                Screen::ConfirmDelete(i) => slots.status[i].details(),
                _ => continue,
            },
            MenuText::Message(screen) => {
                if Some(screen) != current {
                    String::new()
                } else if !session.message.is_empty() {
                    session.message.clone()
                } else if !ready.0 {
                    "Die Insel lädt …".into()
                } else if screen == MenuScreen::Title {
                    slots.note.clone()
                } else {
                    String::new()
                }
            }
            MenuText::PadHelp(_) if !controller.active => String::new(),
            MenuText::PadHelp(MenuScreen::Title) => {
                "Steuerkreuz · Spielstand wählen   A · Bestätigen".into()
            }
            MenuText::PadHelp(MenuScreen::ConfirmDelete) => {
                "Steuerkreuz · Auswählen   A · Bestätigen   B · Abbrechen".into()
            }
            MenuText::PadHelp(MenuScreen::Pause) => {
                "A · Aktion vor dir   X · Werkzeugrad   Y · Inselkarte\nLT · Saatgut / Einrichten   View · Tasche   ↑ · Journal\nLB/RB · Zoomen   RT · Düngen   B · Zurück".into()
            }
            MenuText::ActiveSlot => format!(
                "Spielstand {} · Tag {}",
                session.index.unwrap_or(0) + 1,
                garden.day
            ),
            MenuText::UiSize => if garden.large_ui {
                "Darstellung · Groß"
            } else {
                "Darstellung · Standard"
            }
            .into(),
            MenuText::Sound => if garden.sound_muted {
                "Ton · Aus"
            } else if garden.music_muted {
                "Ton · Ohne Musik"
            } else {
                "Ton · Ein"
            }
            .into(),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
    for (button, interaction, mut bg, mut focus, mut visibility) in &mut buttons {
        let enabled = match button {
            MenuButton::Slot(i) => {
                #[cfg(target_os = "android")]
                let unavailable = session.document.is_some()
                    || session.importing && slots.status[*i] != SlotStatus::Empty;
                #[cfg(not(target_os = "android"))]
                let unavailable = false;
                ready.0 && !unavailable && slots.status[*i] != SlotStatus::Unreadable
            }
            MenuButton::DeleteSlot(i) => {
                let occupied = session.choosing_slot() && slots.status[*i] != SlotStatus::Empty;
                visibility.set_if_neq(if occupied {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                });
                ready.0 && occupied
            }
            _ => true,
        };
        focus.0 = enabled;
        let color = match (*button, *interaction) {
            (MenuButton::DeleteSlot(_), Interaction::Pressed) if enabled => Color::srgb_u8(209, 164, 143),
            (MenuButton::DeleteSlot(_), Interaction::Hovered) if enabled => Color::srgb_u8(224, 182, 159),
            (MenuButton::DeleteSlot(_), Interaction::None) if enabled => Color::srgb_u8(233, 207, 182),
            (MenuButton::ConfirmDelete, Interaction::Pressed) => Color::srgb_u8(109, 43, 31),
            (MenuButton::ConfirmDelete, Interaction::Hovered) => Color::srgb_u8(158, 73, 50),
            (MenuButton::ConfirmDelete, Interaction::None) => Color::srgb_u8(139, 59, 40),
            _ => ui::button_color(*interaction, enabled),
        };
        if bg.0 != color {
            bg.0 = color;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::SPAWN;

    #[test]
    fn an_import_preserves_occupied_slots_and_recovers_the_previous_save() {
        let root = std::env::temp_dir().join(format!("waldwinkel-recovery-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let mut slots = SaveSlots::new(root.join("garten.json"));
        assert!(slots.import(0, b"{}").is_err());
        assert!(!slots.files[0].exists());
        assert!(slots.import(0, br#"{"version":99}"#).is_err());
        slots
            .import(0, br#"{"version":14,"day":8,"wood":11}"#)
            .unwrap();
        assert!(slots.import(0, br#"{"version":14,"day":1}"#).is_err());
        let mut garden = slots.open(0).unwrap();
        assert_eq!((garden.day, garden.wood), (8, 11));
        garden.day = 9;
        garden.save(&slots.files[0]).unwrap();
        fs::write(&slots.files[0], "interrupted").unwrap();
        assert!(matches!(
            SaveSlots::inspect(&slots.files[0]),
            SlotStatus::Recoverable { day: 8, .. }
        ));
        assert!(slots.import(0, br#"{"version":14,"day":1}"#).is_err());
        assert_eq!(slots.open(0).unwrap().day, 8);
        assert_eq!(
            fs::read_to_string(slots.files[0].with_extension("beschaedigt.json")).unwrap(),
            "interrupted"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn legacy_import_and_new_games_keep_progress_independent() {
        let root = std::env::temp_dir().join(format!("waldwinkel-slots-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let legacy = root.join("garten.json");
        fs::write(&legacy, r#"{"version":3,"day":9,"harvested":14}"#).unwrap();
        let mut slots = SaveSlots::new(legacy.clone());
        let mut garden = slots.open(1).unwrap();
        assert_eq!((garden.day, garden.harvested), (1, 0));
        garden.day = 2;
        garden.checkpoint = Some(Checkpoint::new(Area::Island, SPAWN));
        garden.save(&slots.files[1]).unwrap();
        let mut existing = slots.open(0).unwrap();
        assert_eq!((existing.day, existing.harvested), (9, 14));
        existing.day = 10;
        existing.save(&slots.files[0]).unwrap();
        let mut reopened = SaveSlots::new(legacy.clone());
        assert_eq!(reopened.open(0).unwrap().day, 10);
        assert_eq!(reopened.open(1).unwrap().day, 2);
        assert_eq!(
            reopened.open(1).unwrap().checkpoint.unwrap().position,
            SPAWN.to_array()
        );
        assert_eq!(Garden::load(&legacy).unwrap().day, 9);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_deleted_slot_stays_empty_and_leaves_other_saves_and_exports_intact() {
        let root = std::env::temp_dir().join(format!("waldwinkel-delete-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let legacy = root.join("garten.json");
        let old = br#"{"version":3,"day":9,"harvested":14}"#;
        fs::write(&legacy, old).unwrap();
        let mut slots = SaveSlots::new(legacy.clone());
        let export = root.join("Hof-Export.json");
        fs::write(&export, old).unwrap();
        let file = slots.files[0].clone();
        let backup = Garden::backup_path(&file);
        let legacy_backup = Garden::backup_path(&legacy);
        let debris = [
            backup.clone(),
            legacy_backup.clone(),
            file.with_extension("beschaedigt.json"),
            Garden::temporary_path(&file),
            Garden::temporary_path(&backup),
            Garden::temporary_path(&legacy),
            Garden::temporary_path(&legacy_backup),
        ];
        for path in &debris {
            fs::write(path, old).unwrap();
        }
        slots
            .import(1, br#"{"version":18,"day":5,"wood":23}"#)
            .unwrap();
        let other = fs::read(&slots.files[1]).unwrap();
        slots.delete(0).unwrap();
        assert_eq!(slots.status[0], SlotStatus::Empty);
        assert!(slots.note.is_empty());
        assert!(debris.iter().chain([&file, &legacy]).all(|path| !path.exists()));
        assert_eq!(fs::read(&slots.files[1]).unwrap(), other);
        assert_eq!(fs::read(&export).unwrap(), old);
        let mut reopened = SaveSlots::new(legacy);
        assert_eq!(reopened.status[0], SlotStatus::Empty);
        let fresh = reopened.open(0).unwrap();
        assert_eq!((fresh.day, fresh.harvested), (1, 0));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_failed_delete_keeps_the_playable_backup_and_can_be_retried() {
        let root = std::env::temp_dir().join(format!("waldwinkel-retry-delete-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let mut slots = SaveSlots::new(root.join("garten.json"));
        let file = slots.files[1].clone();
        let backup = Garden::backup_path(&file);
        fs::create_dir(&file).unwrap();
        fs::write(&backup, br#"{"version":18,"day":12}"#).unwrap();
        assert!(slots.delete(1).is_err());
        assert_eq!(Garden::load(&backup).unwrap().day, 12);
        fs::remove_dir(&file).unwrap();
        slots.delete(1).unwrap();
        assert!(!backup.exists());
        fs::write(&backup, b"unreadable").unwrap();
        assert_eq!(SaveSlots::inspect(&file), SlotStatus::Unreadable);
        slots.delete(1).unwrap();
        assert_eq!(SaveSlots::inspect(&file), SlotStatus::Empty);
        fs::remove_dir_all(root).unwrap();
    }
}
