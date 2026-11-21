use crate::{
    adventure::Target,
    craft::Recipe,
    duel::LAST_LEVEL,
    garden::Garden,
    harbor::{self, Phase},
    onboarding::Onboarding,
    regions::Frontier,
    story::{Guide, Resonator},
    town::{self, Npc},
    tutorial::{Lesson, Prologue},
    waywatch::{Stage, Watch},
    workshop::Project,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Tutorial(Lesson),
    Garden,
    MeetBo,
    Watch(Watch),
    Pier,
    MeetJuna,
    Gate(Frontier),
    Echo(Frontier),
    ReturnToJuna,
    Festival,
    Done,
}

pub fn festival_ready(garden: &Garden) -> bool {
    garden.story.voices() == 3
        && (garden.rpg.cleared >= LAST_LEVEL
            || (garden.watches.gathered && garden.harbor.phase == Phase::Open))
}

pub fn next(garden: &Garden) -> Step {
    if let Some(lesson) = Prologue::current(garden) {
        return Step::Tutorial(lesson);
    }
    if garden.story.harbor_lit {
        return Step::Done;
    }
    if festival_ready(garden) {
        return Step::Festival;
    }
    match garden.onboarding {
        Onboarding::Harbor => return Step::MeetBo,
        Onboarding::Done => {}
        _ => return Step::Garden,
    }
    if Watch::Fern.stage(garden) != Stage::Complete {
        return Step::Watch(Watch::Fern);
    }
    if garden.harbor.phase != Phase::Open {
        return Step::Pier;
    }
    if !garden.story.met_juna {
        return Step::MeetJuna;
    }
    for watch in [Watch::Tide, Watch::Kiln, Watch::Wind, Watch::Fen] {
        if let Some(frontier) = watch.gate().filter(|region| !region.unlocked(garden)) {
            return Step::Gate(frontier);
        }
        if watch.stage(garden) != Stage::Complete {
            return Step::Watch(watch);
        }
        if let Some(frontier) = watch.gate().filter(|&region| !garden.story.tuned(region)) {
            return Step::Echo(frontier);
        }
    }
    Step::ReturnToJuna
}

impl Step {
    pub fn hint(self, garden: &Garden, controller: bool) -> String {
        match self {
            Self::Tutorial(lesson) => lesson.instruction(garden, controller, false),
            Self::Garden if controller => "Stell dich vor ein Beet und drücke A. X öffnet das Werkzeugrad.".into(),
            Self::Garden => garden.onboarding.hint().into(),
            Self::MeetBo => format!("Sprich mit Bo. {}", Npc::Bo.whereabouts(garden)),
            Self::Watch(site) => site.summary(garden),
            Self::Pier => garden.harbor.phase.label().into(),
            Self::Done => String::new(),
            _ if controller => "↑ Journal · Nächsten Schritt ansehen   Y · Karte".into(),
            _ => "Öffne das Journal oder folge der Markierung auf der Karte.".into(),
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Tutorial(lesson) => lesson.title(),
            Self::Garden => "Dein erster Morgen",
            Self::MeetBo => "Besuch bei Bo",
            Self::Watch(site) => site.name(),
            Self::Pier => "Der Fischersteg",
            Self::MeetJuna => "Junas Geschichte",
            Self::Gate(region) => region.district().name(),
            Self::Echo(region) => Resonator::from_frontier(region).name(),
            Self::ReturnToJuna => "Junas Inselkarte",
            Self::Festival => "Lichtfest in Eichenhafen",
            Self::Done => "Das Licht kehrt zurück",
        }
    }

    pub fn chapter(self) -> &'static str {
        match self {
            Self::Tutorial(_) => "Prolog · Ankommen auf Waldwinkel",
            Self::Garden | Self::MeetBo | Self::Watch(Watch::Fern) | Self::Pier => {
                "Kapitel 1 · Das erste Licht"
            }
            Self::MeetJuna | Self::Gate(_) | Self::Watch(_) | Self::Echo(_) => {
                "Kapitel 2 · Wege über die Insel"
            }
            Self::ReturnToJuna | Self::Festival => "Kapitel 3 · Das Lichtfest",
            Self::Done => "Hauptgeschichte abgeschlossen",
        }
    }

    pub fn detail(self, garden: &Garden) -> String {
        match self {
            Self::Tutorial(lesson) => lesson.instruction(garden, false, false),
            Self::Garden => garden.onboarding.hint().into(),
            Self::MeetBo => self.hint(garden, false),
            Self::Watch(Watch::Fern) if Watch::Fern.stage(garden) == Stage::Surveyed
                && !garden.workshop.restored => Project::Restore.detail(garden),
            Self::Watch(Watch::Fern) if Watch::Fern.stage(garden) == Stage::Surveyed
                && !Watch::Fern.ready(garden) => {
                    format!("Fertige an der Hofwerkbank zwei Fichtenbretter. {}", Recipe::Boards.describe(garden, 1))
                }
            Self::Watch(site) => site.note(garden),
            Self::Pier if !garden.harbor.started => {
                "Zeig Bo das erste Licht und frag nach dem Fischersteg. Am Kartenbinder daheim kannst du jetzt einen Kampfstil wählen.".into()
            }
            Self::Pier => format!(
                "{}\n{}",
                garden.harbor.phase.instructions(),
                harbor::supplies(garden)
            ),
            Self::MeetJuna => "Triff Juna in Eichenhafen. Sie kann die Zeichen der Wachten lesen.".into(),
            Self::Gate(region) => {
                let (requirements, _, _) = region.describe(garden);
                if garden.town.completed() <= region as usize {
                    let who = town::next_city_order(garden).map_or("den Nachbarn", Npc::name);
                    format!("Hilf zuerst {who} in Eichenhafen. {requirements}")
                } else {
                    requirements
                }
            }
            Self::Echo(_) => "Stimme den Klangstein. Seine Stimme trägt bis zur Hafenleuchte.".into(),
            Self::ReturnToJuna => "Bring Juna die Geschichten der fünf Wachten für ihre Inselkarte.".into(),
            Self::Festival => "Entzünde die Hafenleuchte. Die Nachbarn warten am Wasser.".into(),
            Self::Done => "Dein Hof und die Insel sind offen. Die Dämmergänge, Wachtprüfungen und Jahreszeitenfeste gehen weiter.".into(),
        }
    }

    pub fn target(self, garden: &Garden) -> Option<Target> {
        match self {
            Self::Tutorial(lesson) => lesson.target(garden, crate::adventure::Area::Island),
            Self::Garden | Self::Done => None,
            Self::MeetBo => Some(Target::Person(Npc::Bo)),
            Self::Watch(Watch::Fern)
                if Watch::Fern.stage(garden) == Stage::Surveyed && !Watch::Fern.ready(garden) =>
            {
                Some(Target::Bench)
            }
            Self::Watch(site) => site.route(garden),
            Self::Pier => Guide::Pier.target(garden),
            Self::MeetJuna | Self::ReturnToJuna => Some(Target::Person(Npc::Juna)),
            Self::Gate(region) if garden.town.completed() <= region as usize => {
                town::next_city_order(garden).map(Target::Person)
            }
            Self::Gate(region) => Some(Target::Gate(region)),
            Self::Echo(region) => Some(Target::Resonator(Resonator::from_frontier(region))),
            Self::Festival => Some(Target::Resonator(Resonator::Harbor)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        craft::Bench,
        garden::{Action, Crop},
        regions::Bell,
    };

    #[test]
    fn a_new_farm_can_light_fern_with_bos_supplies_and_return_home() {
        let mut garden = Garden::default();
        assert_eq!(next(&garden), Step::Garden);
        garden.onboarding = Onboarding::Till;
        for action in [Action::Till, Action::Sow, Action::Water] {
            garden.work(0, Crop::Carrot, action).unwrap();
        }
        assert_eq!(next(&garden).target(&garden), Some(Target::Person(Npc::Bo)));
        assert!(Onboarding::greet(&mut garden, Npc::Bo));
        assert_eq!(
            next(&garden).target(&garden),
            Some(Target::Watch(Watch::Fern))
        );
        assert!(Watch::Fern.begin(&mut garden));
        assert_eq!(
            next(&garden).target(&garden),
            Some(Target::WatchTrace(Watch::Fern))
        );
        Watch::Fern.inspect(&mut garden).unwrap();
        assert_eq!(next(&garden).target(&garden), Some(Target::Bench));
        Project::Restore.build(&mut garden).unwrap();
        Recipe::Boards.make(&mut garden, Bench::Work, 1).unwrap();
        Watch::Fern.restore(&mut garden).unwrap();
        Watch::Fern
            .trial(&garden)
            .unwrap()
            .reward(&mut garden)
            .unwrap();
        assert!(Bell::Farm.ready(&garden) && Bell::Forest.ready(&garden));
        assert_eq!(next(&garden), Step::Pier);
    }

    #[test]
    fn the_five_lights_and_reopened_pier_lead_to_a_finale_without_an_expedition() {
        let mut garden = Garden::default();
        garden.onboarding = Onboarding::Done;
        garden.story.met_juna = true;
        garden.regions = serde_json::from_str(r#"{"gates":[2,2,2]}"#).unwrap();
        garden.watches = serde_json::from_str(
            r#"{"stages":["Complete","Complete","Complete","Complete","Complete"],"ranks":[1,0,1,0,1]}"#,
        ).unwrap();
        assert_eq!(next(&garden), Step::Pier);
        garden.harbor.phase = Phase::Open;
        for region in Frontier::ALL {
            assert_eq!(next(&garden), Step::Echo(region));
            Resonator::from_frontier(region).work(&mut garden).unwrap();
        }
        assert_eq!(next(&garden), Step::ReturnToJuna);
        assert!(Resonator::Harbor.work(&mut garden).is_err());
        crate::waywatch::conclude(&mut garden).unwrap();
        assert_eq!(next(&garden), Step::Festival);
        assert_eq!(garden.rpg.cleared, 0);
        Resonator::Harbor.work(&mut garden).unwrap();
        assert_eq!(next(&garden), Step::Done);
        assert!(Resonator::Harbor.work(&mut garden).is_err());
    }
}
