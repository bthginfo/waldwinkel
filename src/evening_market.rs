use bevy::prelude::Vec2;
use serde::{Deserialize, Serialize};

use crate::{
    craft::{self, Need, Recipe},
    duel::{Card, Ration},
    garden::Garden,
    homestead::Product,
    processing::Dish,
    town::{Npc, QuestMark, order_done},
};

pub const PLACE: Vec2 = Vec2::new(51.55, 7.35);
pub const HALF_SIZE: Vec2 = Vec2::new(1.03, 0.55);
pub const OPENS: u16 = 18 * 60;
pub const CLOSES: u16 = 21 * 60 + 30;
pub const FIRST_DEPARTURE: u16 = OPENS - 12;
pub const TITLE: &str = "Ein Tisch für den Abend";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum Stage {
    #[default]
    Invitation,
    Build,
    Tasting,
    Opening,
    Open,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Progress {
    pub stage: Stage,
    first_evening: u32,
    sold: [Option<u32>; Ration::ALL.len()],
}

const STALL: [Need; 2] = [
    Need::Stock(Product::Crafted(Recipe::Boards), 2),
    Need::Stone(2),
];
const TASTING: [Need; 1] = [Need::Stock(Product::Prepared(Dish::CarrotSoup), 1)];

pub fn invited(garden: &Garden) -> bool {
    order_done(Npc::Mara, garden)
}

pub fn hours(garden: &Garden) -> bool {
    (OPENS..CLOSES).contains(&garden.clock.minute())
}

pub fn scheduled(garden: &Garden) -> bool {
    garden.evening_market.stage >= Stage::Opening
        && garden.day >= garden.evening_market.first_evening
}

pub fn trading(garden: &Garden) -> bool {
    garden.evening_market.stage == Stage::Open && hours(garden)
}

pub fn can_advance(garden: &Garden) -> bool {
    invited(garden)
        && match garden.evening_market.stage {
            Stage::Invitation => true,
            Stage::Build => craft::ready(&STALL, garden, 1),
            Stage::Tasting => craft::ready(&TASTING, garden, 1),
            Stage::Opening => scheduled(garden) && hours(garden),
            Stage::Open => false,
        }
}

pub fn mark(garden: &Garden) -> Option<QuestMark> {
    if !can_advance(garden) {
        None
    } else if garden.evening_market.stage == Stage::Invitation {
        Some(QuestMark::Available)
    } else {
        Some(QuestMark::TurnIn)
    }
}

pub fn detail(garden: &Garden) -> String {
    if !invited(garden) {
        return "Hilf Mara zuerst bei ihrem Küchenauftrag.".into();
    }
    match garden.evening_market.stage {
        Stage::Invitation => "Mara hat eine Idee für den Marktplatz.".into(),
        Stage::Build => format!(
            "Ein Stand am Brunnen · {}\nFertige die Bretter an deiner Hofwerkbank.",
            craft::detail(&STALL, garden, 1)
        ),
        Stage::Tasting => format!(
            "Die erste Kostprobe · {}\nKoche in der Hofküche mit 2 Möhren und 1 Pilz.",
            craft::detail(&TASTING, garden, 1)
        ),
        Stage::Opening if !scheduled(garden) => {
            "Sprich morgen zwischen 18:00 und 21:30 mit Mara am neuen Stand.".into()
        }
        Stage::Opening => "Sprich zwischen 18:00 und 21:30 mit Mara am neuen Stand.".into(),
        Stage::Open => "Abendmarkt eröffnet · täglich 18:00–21:30 · Proviant am Brunnen".into(),
    }
}

pub fn greeting(garden: &Garden) -> &'static str {
    match garden.evening_market.stage {
        Stage::Invitation => {
            "Als alle die Suppe probierten, blieben sie länger am Brunnen als sonst. Ich möchte ihnen dort abends einen Platz zum Essen geben."
        }
        Stage::Build => {
            "Fiete bringt die Tücher, Lina hat noch Pflanzkübel. Wir brauchen Bretter und Steine für den Tisch."
        }
        Stage::Tasting => {
            "Der Tisch steht. Lass uns zum ersten Abend deine Möhren-Pilz-Suppe anbieten. Dann wissen die Leute, dass sie willkommen sind."
        }
        Stage::Opening => {
            "Das Essen ist bereit. Wenn es dämmert, hängen wir die Laternen auf und ich stelle die Schüsseln hinaus."
        }
        Stage::Open if hours(garden) => {
            "Lina hat die Laternen angezündet. Wenn du eine Wacht besuchen willst, pack ich dir gern etwas ein."
        }
        Stage::Open => {
            "Heute Abend treffen wir uns wieder am Brunnen, sobald die Laternen an sind."
        }
    }
}

pub fn action(garden: &Garden) -> &'static str {
    match garden.evening_market.stage {
        Stage::Invitation => "Mara helfen",
        Stage::Build => "Stand bauen",
        Stage::Tasting => "Suppe übergeben",
        Stage::Opening => "Abendmarkt eröffnen",
        Stage::Open => "Proviant ansehen",
    }
}

pub fn advance(garden: &mut Garden) -> Result<String, String> {
    if !can_advance(garden) {
        return Err(detail(garden));
    }
    let (next, message) = match garden.evening_market.stage {
        Stage::Invitation => (
            Stage::Build,
            "Auftrag angenommen · Ein Tisch für den Abend".into(),
        ),
        Stage::Build => {
            craft::take(&STALL, garden, 1);
            (Stage::Tasting, "Der neue Stand steht am Brunnen.".into())
        }
        Stage::Tasting => {
            craft::take(&TASTING, garden, 1);
            let late = garden.clock.minute() >= FIRST_DEPARTURE;
            garden.evening_market.first_evening = garden.day.saturating_add(u32::from(late));
            let when = if late { "morgen" } else { "heute" };
            (
                Stage::Opening,
                format!("Kostprobe bereit · Komm {when} ab 18:00 zum Abendmarkt."),
            )
        }
        Stage::Opening => {
            garden.farm.coins = garden.farm.coins.saturating_add(28);
            let new_card = garden.rpg.learn(Card::LanternWard);
            let extra = if new_card {
                " · Karte Laternenschutz"
            } else {
                ""
            };
            (
                Stage::Open,
                format!("Der Abendmarkt ist eröffnet · +28 Münzen{extra}"),
            )
        }
        Stage::Open => unreachable!(),
    };
    garden.evening_market.stage = next;
    Ok(message)
}

pub fn price(ration: Ration) -> u32 {
    ration.dish().value() + 6
}

pub fn stocked(garden: &Garden, ration: Ration) -> bool {
    garden.evening_market.sold[ration as usize] != Some(garden.day)
}

pub fn can_buy(garden: &Garden, ration: Ration) -> bool {
    trading(garden)
        && stocked(garden, ration)
        && garden.farm.coins >= price(ration)
        && garden.stock(Product::Prepared(ration.dish())) < u32::MAX
}

pub fn buy(garden: &mut Garden, ration: Ration) -> Result<String, String> {
    if !can_buy(garden, ration) {
        return Err(
            "Der Proviant ist heute ausverkauft oder dein Geld reicht gerade nicht.".into(),
        );
    }
    garden.farm.coins -= price(ration);
    garden.evening_market.sold[ration as usize] = Some(garden.day);
    garden.add_stock(Product::Prepared(ration.dish()), 1);
    Ok(format!("{} eingepackt", ration.dish().name()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::day::FarmClock;

    #[test]
    fn maras_market_unlocks_in_order_and_serves_once_per_ration_and_day() {
        let mut garden: Garden =
            serde_json::from_str(r#"{"town":{"quests":["Done","Offered","Offered"]}}"#).unwrap();
        assert_eq!(mark(&garden), Some(QuestMark::Available));
        advance(&mut garden).unwrap();
        assert!(advance(&mut garden).is_err());
        garden.add_stock(Product::Crafted(Recipe::Boards), 2);
        garden.stone = 2;
        advance(&mut garden).unwrap();
        assert_eq!(garden.stone, 0);
        garden.add_stock(Product::Prepared(Dish::CarrotSoup), 1);
        advance(&mut garden).unwrap();
        assert!(advance(&mut garden).is_err());
        garden.clock = FarmClock::from_minutes(OPENS.into());
        assert_eq!(mark(&garden), Some(QuestMark::TurnIn));
        advance(&mut garden).unwrap();
        assert!(garden.rpg.knows(Card::LanternWard));
        let coins = garden.farm.coins;
        buy(&mut garden, Ration::Soup).unwrap();
        assert!(buy(&mut garden, Ration::Soup).is_err());
        assert_eq!(garden.farm.coins, coins - price(Ration::Soup));
        garden.sleep();
        assert!(!trading(&garden));
        garden.clock = FarmClock::from_minutes(OPENS.into());
        assert!(stocked(&garden, Ration::Soup));
        assert!(advance(&mut garden).is_err());
    }

    #[test]
    fn a_late_delivery_schedules_the_inauguration_for_the_next_evening() {
        let mut garden: Garden = serde_json::from_str(
            r#"{"town":{"quests":["Done","Offered","Offered"]},"evening_market":{"stage":"Tasting"}}"#,
        )
        .unwrap();
        garden.add_stock(Product::Prepared(Dish::CarrotSoup), 1);
        garden.clock = FarmClock::from_minutes(OPENS.into());
        assert!(advance(&mut garden).unwrap().contains("morgen"));
        assert!(!scheduled(&garden));
        assert!(!can_advance(&garden));
        garden.sleep();
        garden.clock = FarmClock::from_minutes(OPENS.into());
        assert!(can_advance(&garden));
    }
}
