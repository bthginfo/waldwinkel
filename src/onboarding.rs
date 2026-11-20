use serde::{Deserialize, Serialize};

use crate::{
    garden::{Action, Crop, Garden},
    town::Npc,
};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Onboarding {
    #[default]
    Welcome,
    Till,
    Sow,
    Water,
    Harbor,
    Done,
}

impl Onboarding {
    pub fn hint(self) -> &'static str {
        match self {
            Self::Welcome | Self::Till => "Tippe auf ein leeres Beet und wähle „Hacken“.",
            Self::Sow => "Wähle das gehackte Beet und säe deine Möhren.",
            Self::Water => "Gieße deine Saat. Im Bett beginnst du später den nächsten Tag.",
            Self::Harbor => {
                "Sprich mit Bo in Eichenhafen. Er kennt die Geschichte des alten Fischerstegs."
            }
            Self::Done => "Erfüllt · Dein Hof ist angelegt. Gegossene Pflanzen wachsen über Nacht.",
        }
    }

    pub fn farm_action(&mut self, action: Action, watered: bool) {
        *self = match (*self, action) {
            (Self::Till, Action::Till) => Self::Sow,
            (Self::Sow, Action::Sow) if watered => Self::Harbor,
            (Self::Sow, Action::Sow) => Self::Water,
            (Self::Water, Action::Water) => Self::Harbor,
            (previous, _) => previous,
        };
    }

    pub fn greet(garden: &mut Garden, person: Npc) -> bool {
        if person != Npc::Bo || garden.onboarding != Self::Harbor {
            return false;
        }
        garden.onboarding = Self::Done;
        garden.add_seeds(Crop::Turnip, 3);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_morning_uses_real_garden_actions_and_a_single_bo_reward() {
        let mut garden = Garden::default();
        assert_eq!(garden.onboarding, Onboarding::Welcome);
        garden.onboarding = Onboarding::Till;
        for (action, next) in [
            (Action::Till, Onboarding::Sow),
            (Action::Sow, Onboarding::Water),
            (Action::Water, Onboarding::Harbor),
        ] {
            assert!(garden.work(0, Crop::Carrot, action).is_some());
            assert_eq!(garden.onboarding, next);
        }
        let seeds = garden.seed_count(Crop::Turnip);
        assert!(!Onboarding::greet(&mut garden, Npc::Fiete));
        assert!(Onboarding::greet(&mut garden, Npc::Bo));
        assert!(!Onboarding::greet(&mut garden, Npc::Bo));
        assert_eq!(garden.seed_count(Crop::Turnip), seeds + 3);
    }
}
