use bevy::prelude::Vec2;
use serde::{Deserialize, Serialize};

use crate::{
    adventure::{Target, dusk_open},
    duel::LAST_LEVEL,
    garden::{Crop, Garden},
    regions::{self, Frontier},
    town::{self, Npc},
};

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StoryProgress {
    pub met_juna: bool,
    tuned: [bool; 3],
    pub harbor_lit: bool,
}

impl StoryProgress {
    pub fn tuned(&self, region: Frontier) -> bool {
        self.tuned[region as usize]
    }

    pub fn voices(&self) -> usize {
        self.tuned.iter().filter(|&&lit| lit).count()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resonator {
    Copper,
    Wind,
    Moor,
    Harbor,
}

impl Resonator {
    pub const ALL: [Self; 4] = [Self::Copper, Self::Wind, Self::Moor, Self::Harbor];

    pub fn from_frontier(region: Frontier) -> Self {
        match region {
            Frontier::Copper => Self::Copper,
            Frontier::Wind => Self::Wind,
            Frontier::Moor => Self::Moor,
        }
    }

    pub fn position(self) -> Vec2 {
        match self {
            Self::Copper => Vec2::new(153.1, -4.65),
            Self::Wind => Vec2::new(34., 146.8),
            Self::Moor => Vec2::new(-154., 19.1),
            Self::Harbor => Vec2::new(56.4, 11.1),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Copper => "Klangstein der Klamm",
            Self::Wind => "Klangstein der Weide",
            Self::Moor => "Klangstein im Moor",
            Self::Harbor => "Hafenleuchte",
        }
    }

    pub fn frontier(self) -> Option<Frontier> {
        match self {
            Self::Copper => Some(Frontier::Copper),
            Self::Wind => Some(Frontier::Wind),
            Self::Moor => Some(Frontier::Moor),
            Self::Harbor => None,
        }
    }

    pub fn active(self, garden: &Garden) -> bool {
        self.frontier().map_or(garden.story.harbor_lit, |frontier| {
            garden.story.tuned(frontier)
        })
    }

    pub fn model(self, lit: bool) -> &'static str {
        match (self, lit) {
            (Self::Copper, false) => "resonance_klamm_idle",
            (Self::Copper, true) => "resonance_klamm_lit",
            (Self::Wind, false) => "resonance_weide_idle",
            (Self::Wind, true) => "resonance_weide_lit",
            (Self::Moor, false) => "resonance_moor_idle",
            (Self::Moor, true) => "resonance_moor_lit",
            (Self::Harbor, false) => "resonance_hafen_idle",
            (Self::Harbor, true) => "resonance_hafen_lit",
        }
    }

    fn echo(self) -> &'static str {
        match self {
            Self::Copper => "Im Metall klingt der Takt der alten Schmiede wieder.",
            Self::Wind => "Die Pfeife fängt den Mühlenwind ein. Eine zweite Stimme antwortet.",
            Self::Moor => "Unter dem Schilf antwortet die dritte Stimme aus dem Stein.",
            Self::Harbor => "Die drei Stimmen tragen wieder bis nach Eichenhafen.",
        }
    }

    pub fn describe(self, garden: &Garden) -> (String, &'static str, bool) {
        if self.active(garden) {
            return (self.echo().into(), "Erklingt", false);
        }
        if !garden.story.met_juna {
            return (
                "Juna in Eichenhafen kennt die alten Klangsteine.".into(),
                "Verstummt",
                false,
            );
        }
        if let Some(frontier) = self.frontier() {
            if !frontier.unlocked(garden) {
                return (
                    "Öffne zuerst den Weg in diese Region.".into(),
                    "Gesperrt",
                    false,
                );
            }
            return (
                "Stimme den Stein und wecke das Echo der Region.".into(),
                "Stimmen",
                true,
            );
        }
        if garden.story.voices() < 3 {
            return (
                format!("{}/3 Klangsteine gestimmt.", garden.story.voices()),
                "Verstummt",
                false,
            );
        }
        if garden.rpg.cleared < LAST_LEVEL {
            return (
                format!(
                    "Das Echo reicht bis Ebene {} der Dämmergänge.",
                    garden.rpg.cleared
                ),
                "Verstummt",
                false,
            );
        }
        (
            "Die Küste ist bereit für ihr Lichtfest.".into(),
            "Fest entzünden",
            true,
        )
    }

    pub fn work(self, garden: &mut Garden) -> Result<String, String> {
        let (hint, _, enabled) = self.describe(garden);
        if !enabled {
            return Err(hint);
        }
        if let Some(frontier) = self.frontier() {
            garden.story.tuned[frontier as usize] = true;
            garden.farm.coins = garden.farm.coins.saturating_add(12);
            return Ok(format!(
                "{} · {}/3 Stimmen · +12 Münzen",
                self.echo(),
                garden.story.voices()
            ));
        }
        garden.story.harbor_lit = true;
        garden.farm.coins = garden.farm.coins.saturating_add(90);
        for crop in Crop::ALL {
            garden.add_seeds(crop, 3);
        }
        Ok("Das Lichtfest von Eichenhafen · +90 Münzen und neues Saatgut".into())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GuideTab {
    Story,
    Orders,
    Town,
    Watches,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Guide {
    Juna,
    Echo(Frontier),
    Dusk,
    Festival,
    SeasonMeeting,
    Farm,
    FirstMorning,
    Workshop,
    Pier,
    Watch(crate::waywatch::Watch),
    WatchFinale,
    Resident(Npc),
    Town(Npc),
    Regional(Frontier),
}

impl Guide {
    pub fn entries(tab: GuideTab) -> impl Iterator<Item = Guide> {
        let entries: &[Guide] = match tab {
            GuideTab::Story => &[
                Self::Juna,
                Self::Echo(Frontier::Copper),
                Self::Echo(Frontier::Wind),
                Self::Echo(Frontier::Moor),
                Self::Dusk,
                Self::Festival,
            ],
            GuideTab::Orders => &[
                Self::FirstMorning,
                Self::Workshop,
                Self::Pier,
                Self::Farm,
                Self::Town(Npc::Mara),
                Self::Town(Npc::Fiete),
                Self::Town(Npc::Bo),
                Self::Regional(Frontier::Copper),
                Self::Regional(Frontier::Wind),
                Self::Regional(Frontier::Moor),
            ],
            GuideTab::Watches => &[
                Self::Watch(crate::waywatch::Watch::Fern),
                Self::Watch(crate::waywatch::Watch::Tide),
                Self::Watch(crate::waywatch::Watch::Kiln),
                Self::Watch(crate::waywatch::Watch::Wind),
                Self::Watch(crate::waywatch::Watch::Fen),
                Self::WatchFinale,
            ],
            GuideTab::Town => &[
                Self::Pier,
                Self::Resident(Npc::Bo),
                Self::Resident(Npc::Fiete),
                Self::Resident(Npc::Lina),
                Self::Resident(Npc::Miro),
                Self::Resident(Npc::Mara),
            ],
        };
        entries.iter().copied()
    }

    pub fn title(self, garden: &Garden) -> &'static str {
        match self {
            Self::Juna => "Eine verstummte Küste",
            Self::Echo(frontier) => Resonator::from_frontier(frontier).name(),
            Self::Dusk => "Unter dem Nebelmoor",
            Self::Festival => "Lichtfest in Eichenhafen",
            Self::SeasonMeeting => "Dorftisch in Eichenhafen",
            Self::Farm => garden.task().0,
            Self::Workshop => "Die alte Hofwerkbank",
            Self::FirstMorning => "Dein erster Morgen",
            Self::Pier => "Der alte Fischersteg",
            Self::Watch(site) => site.name(),
            Self::WatchFinale => "Junas Inselkarte",
            Self::Resident(npc) => npc.name(),
            Self::Town(npc) => town::order_name(npc),
            Self::Regional(frontier) => garden
                .regions
                .next_order(frontier)
                .unwrap_or(frontier.district().name()),
        }
    }

    pub fn detail(self, garden: &Garden) -> String {
        match self {
            Self::Juna if garden.story.met_juna => {
                "Erfüllt · Juna hat dir von den Klangsteinen erzählt.".into()
            }
            Self::Workshop if garden.workshop.restored => {
                "Erfüllt · Drei Möbelbaupläne gelernt.".into()
            }
            Self::Workshop => format!(
                "{} · Wähle die alte Werkbank vor der Hütte.",
                crate::workshop::Project::Restore.detail(garden)
            ),
            Self::Juna => "Sprich mit Juna am Hafen von Eichenhafen.".into(),
            Self::Echo(frontier) if garden.story.tuned(frontier) => {
                "Gestimmt · Das Echo ist wieder zu hören.".into()
            }
            Self::Echo(_) if !garden.story.met_juna => {
                "Juna kennt die Geschichte dieser Spur.".into()
            }
            Self::Echo(frontier) if !frontier.unlocked(garden) => {
                format!("Weg gesperrt · {}", frontier.describe(garden).0)
            }
            Self::Echo(frontier) => match frontier {
                Frontier::Copper => "Suche am alten Erzgleis östlich des Bachs.".into(),
                Frontier::Wind => "Folge den Flachsfeldern südlich der Mühle.".into(),
                Frontier::Moor => "Überquere den Nordsteg zu den Weiden am Moorufer.".into(),
            },
            Self::Dusk if garden.rpg.cleared == LAST_LEVEL => {
                "Erfüllt · Die letzte Glocke klingt wieder.".into()
            }
            Self::Dusk if dusk_open(garden) && garden.rpg.camp() == 0 => {
                format!(
                    "{}/{} Hüter besiegt · Erste Rast nach Ebene 5",
                    garden.rpg.cleared, LAST_LEVEL
                )
            }
            Self::Dusk if dusk_open(garden) => format!(
                "{}/{} Hüter besiegt · Glockenrast {}",
                garden.rpg.cleared,
                LAST_LEVEL,
                garden.rpg.camp()
            ),
            Self::Dusk if Frontier::Moor.unlocked(garden) => {
                "Hilf Edda bei ihren ersten zwei Aufträgen.".into()
            }
            Self::Dusk => "Der Moorweg führt zu Edda und zum Steinbogen.".into(),
            Self::Festival if garden.story.harbor_lit => {
                "Erfüllt · Die Hafenleuchte brennt in Eichenhafen.".into()
            }
            Self::Festival if garden.story.voices() == 3 && garden.rpg.cleared >= LAST_LEVEL => {
                "Geh zur Hafenleuchte und entzünde das Lichtfest.".into()
            }
            Self::Festival => {
                "Stimme alle drei Klangsteine und erreiche die letzte Dämmerglocke.".into()
            }
            Self::SeasonMeeting => crate::seasons::meeting_status(garden),
            Self::Farm if garden.task().1 => "Abgabebereit · +5 Münzen und Saatgut".into(),
            Self::Farm if garden.challenge >= 8 => "Erfüllt".into(),
            Self::Farm => "Hofaufgabe · +5 Münzen und Saatgut".into(),
            Self::FirstMorning => garden.onboarding.hint().into(),
            Self::Pier if !garden.harbor.started => {
                "Sprich mit Bo oder lies den Aushang am Bautisch.".into()
            }
            Self::Pier => format!(
                "{} · {}",
                garden.harbor.phase.label(),
                crate::harbor::supplies(garden)
            ),
            Self::Watch(site) => site.summary(garden),
            Self::WatchFinale if garden.watches.gathered => {
                "Erfüllt · Die fünf Lichter stehen auf der Karte.".into()
            }
            Self::WatchFinale => format!("{}/5 Lichter · Danach zu Juna", garden.watches.lights()),
            Self::Resident(npc) => npc.whereabouts(garden).into(),
            Self::Town(npc) => town::order_detail(npc, garden),
            Self::Regional(frontier) if !frontier.unlocked(garden) => {
                format!("Weg gesperrt · {}", frontier.describe(garden).0)
            }
            Self::Regional(frontier) => {
                let done = garden.regions.story_steps(frontier);
                if done >= 3 {
                    return "Erfüllt · Alle drei Aufträge der Region".into();
                }
                let (detail, action, _) = regions::story(frontier, garden);
                let supplies = detail.lines().last().unwrap_or_default();
                format!("{done}/3 erfüllt · {action} · {supplies}")
            }
        }
    }

    pub fn target(self, garden: &Garden) -> Option<Target> {
        let person = |npc| Some(Target::Person(npc));
        let gateway = |region: Frontier| {
            if garden.town.completed() <= region as usize {
                return town::next_city_order(garden).and_then(person);
            }
            Some(Target::Gate(region))
        };
        match self {
            Self::Juna => (!garden.story.met_juna).then_some(Target::Person(Npc::Juna)),
            Self::Echo(_) if !garden.story.met_juna => person(Npc::Juna),
            Self::Echo(frontier) if garden.story.tuned(frontier) => None,
            Self::Echo(frontier) if !frontier.unlocked(garden) => gateway(frontier),
            Self::Echo(frontier) => Some(Target::Resonator(Resonator::from_frontier(frontier))),
            Self::Dusk if garden.rpg.cleared >= LAST_LEVEL => None,
            Self::Dusk if dusk_open(garden) => Some(Target::DuskGate),
            Self::Dusk if Frontier::Moor.unlocked(garden) => person(Npc::Edda),
            Self::Dusk => gateway(Frontier::Moor),
            Self::Festival if garden.story.harbor_lit => None,
            Self::Festival if garden.story.voices() == 3 && garden.rpg.cleared >= LAST_LEVEL => {
                Some(Target::Resonator(Resonator::Harbor))
            }
            Self::Festival | Self::Farm => None,
            Self::FirstMorning => match garden.onboarding {
                crate::onboarding::Onboarding::Harbor => person(Npc::Bo),
                crate::onboarding::Onboarding::Done => None,
                _ => Some(Target::Door),
            },
            Self::Workshop => (!garden.workshop.restored).then_some(Target::Bench),
            Self::Pier => Some(match garden.harbor.phase {
                crate::harbor::Phase::Damaged if !garden.harbor.started => Target::Person(Npc::Bo),
                crate::harbor::Phase::Damaged if garden.harbor.salvage_available() => {
                    Target::HarborDriftwood
                }
                crate::harbor::Phase::Landing if !garden.harbor.fittings => Target::Bench,
                _ => Target::HarborBoard,
            }),
            Self::Watch(site) => site.route(garden),
            Self::WatchFinale => crate::waywatch::finale_route(garden),
            Self::Resident(npc) => person(npc),
            Self::SeasonMeeting => Some(Target::SeasonMeeting),
            Self::Town(npc) => (!town::order_done(npc, garden)).then_some(Target::Person(npc)),
            Self::Regional(frontier) if !frontier.unlocked(garden) => gateway(frontier),
            Self::Regional(frontier) if garden.regions.story_steps(frontier) >= 3 => None,
            Self::Regional(frontier) => person(match frontier {
                Frontier::Copper => Npc::Runa,
                Frontier::Wind => Npc::Nele,
                Frontier::Moor => Npc::Edda,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_harbor_festival_needs_all_echoes_and_the_last_floor_and_pays_once() {
        let mut garden = Garden::default();
        assert_eq!(Guide::Juna.target(&garden), Some(Target::Person(Npc::Juna)));
        assert!(Resonator::Copper.work(&mut garden).is_err());
        garden.story.met_juna = true;
        assert_eq!(
            Guide::Echo(Frontier::Copper).target(&garden),
            Some(Target::Person(Npc::Mara))
        );
        garden.regions = serde_json::from_str(r#"{"gates":[2,2,2]}"#).unwrap();
        for site in [Resonator::Copper, Resonator::Wind, Resonator::Moor] {
            site.work(&mut garden).unwrap();
            assert!(site.work(&mut garden).is_err());
        }
        assert!(Resonator::Harbor.work(&mut garden).is_err());
        garden.rpg.cleared = LAST_LEVEL;
        let coins = garden.farm.coins;
        assert_eq!(
            Guide::Festival.target(&garden),
            Some(Target::Resonator(Resonator::Harbor))
        );
        Resonator::Harbor.work(&mut garden).unwrap();
        assert!(Resonator::Harbor.work(&mut garden).is_err());
        assert_eq!(garden.farm.coins, coins + 90);
        assert!(garden.story.harbor_lit);
    }
}
