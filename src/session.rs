use std::{io, path::PathBuf};

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
    Unreadable,
}

#[derive(Resource)]
pub struct SaveSlots {
    files: [PathBuf; SLOT_COUNT],
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
            if files[0].try_exists()? || !legacy.try_exists()? {
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
            status,
            note,
        }
    }

    fn inspect(file: &std::path::Path) -> SlotStatus {
        match Garden::load(file) {
            Ok(garden) => Self::summary(&garden),
            Err(error) if error.kind() == io::ErrorKind::NotFound => SlotStatus::Empty,
            Err(_) => SlotStatus::Unreadable,
        }
    }

    fn summary(garden: &Garden) -> SlotStatus {
        let place = garden
            .checkpoint
            .map_or_else(|| Area::Island.name(), Checkpoint::place);
        SlotStatus::Saved {
            day: garden.day,
            place,
            quests: garden.town.completed(),
        }
    }

    fn open(&mut self, index: usize) -> io::Result<Garden> {
        if matches!(self.status[index], SlotStatus::Unreadable) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Dieser Spielstand ist nicht lesbar.",
            ));
        }
        let garden = match Garden::load(&self.files[index]) {
            Ok(garden) => garden,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let garden = Garden::default();
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
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Title,
    Playing,
    Pause,
}

#[derive(Resource, Default)]
pub struct Session {
    screen: Screen,
    index: Option<usize>,
    message: String,
    last_saved: f32,
    last_clock: u16,
    save_failed: bool,
}

impl Session {
    pub fn playing(&self) -> bool {
        self.screen == Screen::Playing
    }

    fn save(
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
) -> bool {
    session.playing() && !dialog.open() && !farm_menu.open()
}

#[derive(Component)]
pub struct MenuScreen(Screen);

#[derive(Component, Clone, Copy)]
pub enum MenuButton {
    Slot(usize),
    Resume,
    Save,
    Title,
    UiSize,
}

#[derive(Component)]
pub enum MenuText {
    SlotMeta(usize),
    SlotAction(usize),
    Message(Screen),
    ActiveSlot,
    UiSize,
}

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>) {
    for screen in [Screen::Title, Screen::Pause] {
        commands
            .spawn((
                MenuScreen(screen),
                Node {
                    display: if screen == Screen::Title {
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
                            width: Val::Px(if screen == Screen::Title { 546. } else { 398. }),
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
                        panel.spawn(fonts.title(
                            if screen == Screen::Title {
                                "Waldwinkel"
                            } else {
                                "Spiel pausiert"
                            },
                            39.,
                        ));
                        if screen == Screen::Title {
                            panel.spawn(fonts.body("Wähle deinen Spielstand.", 16.));
                            for index in 0..SLOT_COUNT {
                                panel
                                    .spawn((
                                        Node {
                                            flex_direction: FlexDirection::Row,
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::SpaceBetween,
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
                                        .with_children(
                                            |labels| {
                                                labels.spawn(fonts.bold(
                                                    &format!("Spielstand {}", index + 1),
                                                    18.,
                                                ));
                                                labels.spawn((
                                                    MenuText::SlotMeta(index),
                                                    fonts.body("Frei", 13.),
                                                ));
                                            },
                                        );
                                        row.spawn((
                                            MenuButton::Slot(index),
                                            ui::dialog_button(146.),
                                        ))
                                        .with_child((
                                            MenuText::SlotAction(index),
                                            fonts.button("Neues Spiel", 15.),
                                        ));
                                    });
                            }
                        } else {
                            panel.spawn((MenuText::ActiveSlot, fonts.body("", 15.)));
                            for (action, label) in [
                                (MenuButton::Resume, "Weiterspielen"),
                                (MenuButton::Save, "Jetzt speichern"),
                                (MenuButton::UiSize, "Darstellung"),
                                (MenuButton::Title, "Speichern & Hauptmenü"),
                            ] {
                                let mut button = panel.spawn((action, ui::dialog_button(350.)));
                                if matches!(action, MenuButton::UiSize) {
                                    button.with_child((MenuText::UiSize, fonts.button(label, 16.)));
                                } else {
                                    button.with_child(fonts.button(label, 16.));
                                }
                            }
                        }
                        panel.spawn((MenuText::Message(screen), fonts.body("", 13.)));
                    });
            });
    }
}

pub fn input(
    buttons: Query<(&MenuButton, &Interaction), Changed<Interaction>>,
    keys: Res<ButtonInput<KeyCode>>,
    ready: Res<Ready>,
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
    if keys.just_pressed(KeyCode::Escape) {
        if farm_menu.battle().is_some() {
            return;
        }
        if farm_menu.open() {
            farm_menu.close();
            *gesture = Gesture::default();
            *controls = Controls::default();
            return;
        }
        if dialog.open() {
            dialog.close();
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
        (Screen::Pause, MenuButton::UiSize) => {
            g.large_ui = !g.large_ui;
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
                    session.screen = Screen::Title;
                    session.index = None;
                    session.message.clear();
                    slots.note.clear();
                }
                Ok(()) => session.message = "Spielstand gespeichert.".into(),
                Err(error) => {
                    warn!("Spielstand konnte nicht gespeichert werden: {error}");
                    session.message = "Speichern fehlgeschlagen. Bitte erneut versuchen.".into();
                }
            }
        }
        _ => {}
    }
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
    slots: Res<SaveSlots>,
    garden: Res<Garden>,
    ready: Res<Ready>,
    mut overlay: Query<(&MenuScreen, &mut Node)>,
    mut texts: Query<(&MenuText, &mut Text)>,
    mut buttons: Query<(&MenuButton, &Interaction, &mut BackgroundColor)>,
) {
    for (screen, mut node) in &mut overlay {
        let display = if screen.0 == session.screen {
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
            MenuText::SlotAction(i) => match slots.status[i] {
                SlotStatus::Empty => "Neues Spiel",
                SlotStatus::Saved { .. } => "Fortsetzen",
                SlotStatus::Unreadable => "Nicht lesbar",
            }
            .into(),
            MenuText::SlotMeta(i) => match slots.status[i] {
                SlotStatus::Empty => "Freier Platz".into(),
                SlotStatus::Saved { day, place, quests } => {
                    format!("Tag {day} · {place} · Aufträge {quests}/3")
                }
                SlotStatus::Unreadable => "Datei bleibt erhalten.".into(),
            },
            MenuText::Message(screen) => {
                if screen != session.screen {
                    String::new()
                } else if !session.message.is_empty() {
                    session.message.clone()
                } else if !ready.0 {
                    "Die Insel lädt …".into()
                } else if screen == Screen::Title {
                    slots.note.clone()
                } else {
                    String::new()
                }
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
        };
        if text.0 != value {
            text.0 = value;
        }
    }
    for (button, interaction, mut bg) in &mut buttons {
        let enabled = match button {
            MenuButton::Slot(i) => ready.0 && !matches!(slots.status[*i], SlotStatus::Unreadable),
            _ => true,
        };
        let color = ui::button_color(*interaction, enabled);
        if bg.0 != color {
            bg.0 = color;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::SPAWN;
    use std::fs;

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
}
