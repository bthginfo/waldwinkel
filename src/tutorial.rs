use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    Adventure, Notice,
    adventure::{Area, Target},
    garden::{Action, Crop, Garden, SavePath, Stage},
    homestead::Product,
    onboarding::Onboarding,
    processing::Dish,
    session::Session,
    ui::UiFonts,
    world::{Gardener, PLOTS},
};

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Prologue {
    pub moved: bool,
    pub plot: Option<usize>,
    pub planted: bool,
    pub watered: bool,
    pub chopped: bool,
    pub quarried: bool,
    pub boarded: bool,
    pub harvested: bool,
    pub cooked: bool,
    pub sparred: bool,
    pub mapped: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Lesson {
    Walk,
    Till,
    Sow,
    Water,
    Wood,
    Stone,
    Bench,
    Boards,
    Sleep,
    Tend,
    Harvest,
    Kitchen,
    Cards,
    Map,
    Complete,
}

impl Prologue {
    pub fn current(garden: &Garden) -> Option<Lesson> {
        let p = garden.prologue.as_ref()?;
        let plot = p.plot.and_then(|i| garden.plots.get(i)).copied();
        Some(if !p.moved {
            Lesson::Walk
        } else if p.plot.is_none() {
            Lesson::Till
        } else if !p.planted {
            Lesson::Sow
        } else if !p.watered && !plot.is_some_and(|p| p.remaining().is_some_and(|(_, wet)| wet)) {
            Lesson::Water
        } else if !p.chopped {
            Lesson::Wood
        } else if !p.quarried {
            Lesson::Stone
        } else if !garden.workshop.restored {
            Lesson::Bench
        } else if !p.boarded {
            Lesson::Boards
        } else if !p.harvested {
            match plot {
                Some(plot) if plot.stage() == Stage::Ripe => Lesson::Harvest,
                Some(plot) if plot.remaining().is_some_and(|(_, wet)| !wet) => Lesson::Tend,
                _ => Lesson::Sleep,
            }
        } else if !p.cooked {
            Lesson::Kitchen
        } else if !p.sparred {
            Lesson::Cards
        } else if !p.mapped {
            Lesson::Map
        } else {
            Lesson::Complete
        })
    }

    pub fn farm(garden: &mut Garden, index: usize, action: Action, wet: bool) {
        let Some(p) = &mut garden.prologue else {
            return;
        };
        match action {
            Action::Till => {
                p.moved = true;
                p.plot.get_or_insert(index);
            }
            Action::Sow if !p.planted => {
                p.moved = true;
                p.plot = Some(index);
                p.planted = true;
                p.watered = wet;
            }
            Action::Water if p.plot == Some(index) => p.watered = true,
            Action::Harvest if !p.harvested => {
                p.harvested = true;
                p.watered = true;
                // Keep ingredients in the player's bag for the next hands-on lesson.
                let short = 2_u32.saturating_sub(garden.crop_total(Crop::Carrot));
                garden.add_stock(
                    Product::Crop(Crop::Carrot, crate::garden::Quality::Normal),
                    short,
                );
                garden.add_stock(Product::Mushroom, 1);
            }
            _ => {}
        }
    }

    fn finish(garden: &mut Garden) -> bool {
        if Self::current(garden) != Some(Lesson::Complete) {
            return false;
        }
        garden.prologue = None;
        garden.onboarding = Onboarding::Harbor;
        garden.farm.coins = garden.farm.coins.saturating_add(10);
        true
    }
}

impl Lesson {
    pub fn title(self) -> &'static str {
        match self {
            Self::Walk => "Erste Schritte",
            Self::Till | Self::Sow | Self::Water => "Ein eigenes Beet",
            Self::Wood | Self::Stone => "Holz und Stein",
            Self::Bench | Self::Boards => "Die Hofwerkbank",
            Self::Sleep | Self::Tend | Self::Harvest => "Von der Saat zur Ernte",
            Self::Kitchen => "Ein Topf für unterwegs",
            Self::Cards => "Übungsduell am Kartenbinder",
            Self::Map | Self::Complete => "Die große Insel",
        }
    }

    pub fn instruction(self, garden: &Garden, pad: bool, indoors: bool) -> String {
        let plot = garden
            .prologue
            .as_ref()
            .and_then(|p| p.plot)
            .map_or(1, |i| i + 1);
        match self {
            Self::Walk if pad => "Bewege den Tanuki mit dem linken Stick ein Stück über den Hof.".into(),
            Self::Walk => "Zieh den runden Stick links unten ein Stück in eine Richtung.".into(),
            Self::Till if pad => "Stell dich vor das markierte Beet und drücke A. Mit X öffnest du das Werkzeugrad.".into(),
            Self::Till => "Tippe auf das markierte Wiesenbeet. Wähle danach „Beet anlegen“.".into(),
            Self::Sow if pad => format!("Wähle mit LT die Möhren. Stell dich vor Beet {plot} und drücke A zum Säen."),
            Self::Sow => format!("Wähle über „Saatgut“ die Möhren. Tippe am gewählten Beet {plot} auf „Aussäen“."),
            Self::Water | Self::Tend if indoors => format!("Geh durch die Haustür zurück zum Hof. Beet {plot} braucht Wasser."),
            Self::Water | Self::Tend if pad => format!("Wähle über X die Gießkanne. Stell dich vor Beet {plot} und drücke A."),
            Self::Water | Self::Tend => format!("Wähle bei „Werkzeuge“ die Gießkanne. Tippe auf Beet {plot} und dann auf „Gießen“."),
            Self::Wood if pad => "Wähle über X die Axt. Stell dich vor den markierten Baum und drücke A.".into(),
            Self::Wood => "Tippe am markierten Baum auf „Markiertes Ziel“ und dann auf „Baum fällen“.".into(),
            Self::Stone if pad => "Wähle über X die Spitzhacke. Stell dich vor den Findling und drücke A.".into(),
            Self::Stone => "Tippe am Findling auf „Markiertes Ziel“ und dann auf „Stein abbauen“.".into(),
            Self::Bench => "Wähle die kaputte Werkbank vor dem Haus. Öffne die Bauprojekte und setze sie instand.".into(),
            Self::Boards if !crate::craft::Recipe::Boards.can_make(garden, crate::craft::Bench::Work, 1) =>
                "Fälle noch einen Baum für die Bretter. Fertige sie dann an der Werkbank unter „Materialien“.".into(),
            Self::Boards => "Öffne die Werkbank und gehe zu „Materialien“. Fertige eine Ladung Fichtenbretter.".into(),
            Self::Sleep if indoors && pad => "Stell dich vor das Bett und drücke A. Beim Schlafen speichert Waldwinkel deinen Hof.".into(),
            Self::Sleep if indoors => "Wähle das Bett. Dein Hof wird gespeichert und die Beete wachsen über Nacht.".into(),
            Self::Sleep => "Wähle die Haustür und drücke „Eintreten“. Geh drinnen zum Bett und schlafe.".into(),
            Self::Harvest if indoors => format!("Geh durch die Haustür zurück zum Hof. Beet {plot} ist erntebereit."),
            Self::Harvest if pad => format!("Wähle über X den Erntekorb. Schau zu Beet {plot} und drücke A."),
            Self::Harvest => format!("Wähle bei „Werkzeuge“ den Erntekorb und ernte Beet {plot}."),
            Self::Kitchen if indoors => "Geh durch die Haustür zur Hofküche. Sie steht neben den Beeten.".into(),
            Self::Kitchen if Dish::CarrotSoup.ready(garden) => "Öffne die Hofküche und koche Möhren-Pilz-Suppe. Füge die Zutaten hinzu, rühre und schöpfe ab.".into(),
            Self::Kitchen => "Für die Suppe brauchst du zwei Möhren und einen Pilz. Pilze findest du am Farnbach.".into(),
            Self::Cards if indoors => "Öffne den Kartenbinder und wähle „Üben“. Der Strohwächter zeigt dir Angriff, Schutz und Zugwechsel.".into(),
            Self::Cards => "Geh durch die Haustür zum Kartenbinder. Dort wartet ein Übungsduell.".into(),
            Self::Map if indoors => "Geh durch die Tür nach draußen. In der Übersicht siehst du die Wege und markierten Ziele.".into(),
            Self::Map if pad => "Drücke Y für die Inselkarte. Die Markierung zeigt dir den Weg zu Bo.".into(),
            Self::Map => "Tippe auf „Karte“ oben links. Die Markierung zeigt dir den Weg zu Bo.".into(),
            Self::Complete => "Der Prolog ist geschafft. Bo wartet in Eichenhafen.".into(),
        }
    }

    pub fn target(self, garden: &Garden, area: Area) -> Option<Target> {
        if area == Area::Home {
            return Some(match self {
                Self::Sleep => Target::Bed,
                Self::Cards => Target::Binder,
                _ => Target::Exit,
            });
        }
        if !area.outdoors() {
            return None;
        }
        Some(match self {
            Self::Wood if !garden.felled.contains(&10) => Target::Tree(10),
            Self::Wood => Target::Tree(3),
            Self::Stone if !garden.broken.contains(&7) => Target::Rock(7),
            Self::Stone => Target::Rock(6),
            Self::Boards
                if !crate::craft::Recipe::Boards.can_make(garden, crate::craft::Bench::Work, 1) =>
            {
                garden
                    .felled
                    .contains(&10)
                    .then_some(Target::Tree(3))
                    .unwrap_or(Target::Tree(10))
            }
            Self::Bench | Self::Boards => Target::Bench,
            Self::Sleep | Self::Cards => Target::Door,
            Self::Kitchen => Target::Kitchen,
            Self::Complete => Target::Person(crate::town::Npc::Bo),
            _ => return None,
        })
    }

    pub fn plot(self, garden: &Garden) -> Option<usize> {
        match self {
            Self::Till => Some(4),
            Self::Sow | Self::Water | Self::Tend | Self::Harvest => garden.prologue.as_ref()?.plot,
            _ => None,
        }
    }

    pub fn action(self) -> Option<Action> {
        match self {
            Self::Till => Some(Action::Till),
            Self::Sow => Some(Action::Sow),
            Self::Water | Self::Tend => Some(Action::Water),
            Self::Harvest => Some(Action::Harvest),
            _ => None,
        }
    }
}

#[derive(Component)]
pub struct TutorialPanel;
#[derive(Component)]
pub struct TutorialTitle;
#[derive(Component)]
pub struct TutorialText;

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>) {
    commands
        .spawn((
            TutorialPanel,
            Visibility::Hidden,
            GlobalZIndex(5),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(128.),
                left: Val::Percent(31.),
                width: Val::Percent(37.),
                padding: UiRect::axes(Val::Px(14.), Val::Px(8.)),
                border: UiRect::all(Val::Px(1.)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BorderColor(Color::srgb_u8(196, 174, 127)),
            BorderRadius::all(Val::Px(12.)),
            BackgroundColor(Color::srgba_u8(31, 57, 49, 242)),
        ))
        .with_children(|p| {
            let (label, font, _) = fonts.title("", 16.);
            p.spawn((
                TutorialTitle,
                label,
                font,
                TextColor(Color::srgb_u8(255, 215, 142)),
            ));
            let (label, font, _) = fonts.body("", 14.);
            p.spawn((
                TutorialText,
                label,
                font,
                TextColor(Color::srgb_u8(255, 247, 218)),
            ));
        });
}

pub fn update(
    mut garden: ResMut<Garden>,
    adventure: Res<Adventure>,
    player: Single<&Gardener>,
    controls: Res<crate::Controls>,
    session: Res<Session>,
    overlays: (
        Res<crate::farm_ui::FarmMenu>,
        Res<crate::wheel::Wheel>,
        Res<crate::decorate::Layout>,
        Res<crate::night::Night>,
    ),
    controller: Res<crate::controller::Controller>,
    dialog: Res<crate::town::Dialog>,
    path: Res<SavePath>,
    time: Res<Time>,
    mut notice: ResMut<Notice>,
    mut panel: Single<&mut Visibility, With<TutorialPanel>>,
    mut title: Single<&mut Text, (With<TutorialTitle>, Without<TutorialText>)>,
    mut body: Single<&mut Text, (With<TutorialText>, Without<TutorialTitle>)>,
) {
    let (menu, wheel, layout, night) = overlays;
    let active = session.playing() && !menu.open() && !dialog.open()
        && !wheel.open() && !layout.open() && !night.active();
    if !active || garden.onboarding == Onboarding::Welcome {
        panel.set_if_neq(Visibility::Hidden);
        return;
    }
    let Some(p) = garden.prologue.as_ref() else {
        panel.set_if_neq(Visibility::Hidden);
        return;
    };
    let moved =
        (player.position - crate::world::SPAWN).length() > 1.25 && adventure.area.outdoors();
    if !p.moved && moved {
        garden.prologue.as_mut().unwrap().moved = true;
    }
    if controls.map && adventure.area.outdoors() && !garden.prologue.as_ref().unwrap().mapped {
        garden.prologue.as_mut().unwrap().mapped = true;
    }
    let Some(mut lesson) = Prologue::current(&garden) else {
        return;
    };
    if Prologue::finish(&mut garden) {
        panel.set_if_neq(Visibility::Hidden);
        crate::adventure::commit(
            &garden,
            &path,
            &mut notice,
            time.elapsed_secs(),
            "Prolog geschafft · +10 Münzen · Triff Bo in Eichenhafen".into(),
            None,
            None,
        );
        return;
    }
    if garden.clock.night() && lesson.action().is_some() {
        lesson = Lesson::Sleep;
    }
    if !adventure.map {
        panel.set_if_neq(Visibility::Inherited);
        title.set_if_neq(Text::new(format!("Prolog · {}", lesson.title())));
        body.set_if_neq(Text::new(lesson.instruction(
            &garden,
            controller.active,
            adventure.area == Area::Home,
        )));
    } else {
        panel.set_if_neq(Visibility::Hidden);
    }
}

pub fn suggested_plot(garden: &Garden) -> Option<usize> {
    Prologue::current(garden)?
        .plot(garden)
        .filter(|&i| i < PLOTS.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        campaign::{self, Step},
        craft::{Bench, Recipe},
        duel::{Battle, Card, Outcome},
        processing::Preparation,
        workshop::Project,
    };

    #[test]
    fn prologue_uses_real_resources_and_opens_the_story_once() {
        let mut garden = Garden::default();
        garden.prologue = Some(Prologue::default());
        garden.onboarding = Onboarding::Till;
        assert_eq!(campaign::next(&garden), Step::Tutorial(Lesson::Walk));
        for action in [Action::Till, Action::Sow, Action::Water] {
            if garden.action(4, Crop::Carrot) == Some(action) {
                garden.work(4, Crop::Carrot, action).unwrap();
            }
        }
        assert_eq!(Prologue::current(&garden), Some(Lesson::Wood));
        assert!(garden.chop_tree(10));
        assert_eq!(Prologue::current(&garden), Some(Lesson::Stone));
        garden.stone += garden.stone_per_rock();
        garden.prologue.as_mut().unwrap().quarried = true;
        Project::Restore.build(&mut garden).unwrap();
        assert!(garden.chop_tree(3));
        Recipe::Boards.make(&mut garden, Bench::Work, 1).unwrap();
        assert_eq!(Prologue::current(&garden), Some(Lesson::Sleep));
        garden.sleep();
        if garden.action(4, Crop::Carrot) == Some(Action::Water) {
            assert_eq!(Prologue::current(&garden), Some(Lesson::Tend));
            garden.work(4, Crop::Carrot, Action::Water).unwrap();
        }
        garden.sleep();
        assert_eq!(Prologue::current(&garden), Some(Lesson::Harvest));
        garden.work(4, Crop::Carrot, Action::Harvest).unwrap();
        assert!(Dish::CarrotSoup.ready(&garden));
        let mut soup = Preparation::new(Dish::CarrotSoup, 0.);
        for at in [0., 0.9, 4.1, 7.3, 7.6] {
            assert!(soup.try_step(0, at));
        }
        assert_eq!(soup.finish(&mut garden), Some(1));
        assert_eq!(Prologue::current(&garden), Some(Lesson::Cards));
        let snapshot = serde_json::to_vec(&garden).unwrap();
        let mut garden = Garden::decode(&snapshot).unwrap();
        assert_eq!(Prologue::current(&garden), Some(Lesson::Cards));
        let mut battle = Battle::training();
        let hp = battle.hp;
        assert!(battle.play(0));
        assert_eq!(battle.guard, 8);
        assert!(battle.play(0));
        assert!(battle.play(1));
        battle.end_turn();
        assert_eq!(battle.hp, hp);
        for card in [Card::SeedShot, Card::TwigStrike, Card::TrailRush] {
            let index = battle.hand.iter().position(|&c| c == card).unwrap();
            assert!(battle.play(index));
        }
        assert_eq!(battle.outcome, Outcome::Won);
        let progress = garden.prologue.as_mut().unwrap();
        progress.sparred = true;
        progress.mapped = true;
        let balance = garden.farm.coins;
        assert!(Prologue::finish(&mut garden));
        assert!(!Prologue::finish(&mut garden));
        assert_eq!(garden.farm.coins, balance + 10);
        assert_eq!(campaign::next(&garden), Step::MeetBo);
        assert_eq!(garden.rpg.cleared, 0);
    }
}
