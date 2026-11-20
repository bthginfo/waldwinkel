use serde::{Deserialize, Serialize};

use crate::{
    craft::{self, Need, Recipe},
    garden::{Crop, Garden},
    homestead::Product,
    processing::Dish,
    waywatch::{Stage, Watch},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activity {
    Harvest,
    Fell,
    Mine,
    Fish,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Order {
    OrchardCrates,
    AfterTheStorm,
    AQuietMorning,
    SeasonTable,
    PantryBox,
    RailCrew,
    SailRepair,
    EveningLights,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Ledger {
    active: Option<Order>,
    progress: u32,
    completed: u32,
    claimed: [u32; Order::ALL.len()],
}

impl Ledger {
    pub fn active(&self) -> Option<Order> {
        self.active
    }

    pub fn count(&self) -> u32 {
        self.completed
    }

    pub fn record(&mut self, activity: Activity, count: u32) {
        if let Some((kind, required, _)) = self.active.and_then(Order::activity)
            && kind == activity
        {
            self.progress = self.progress.saturating_add(count).min(required);
        }
    }
}

impl Order {
    const ALL: [Self; 8] = [
        Self::OrchardCrates,
        Self::AfterTheStorm,
        Self::AQuietMorning,
        Self::SeasonTable,
        Self::PantryBox,
        Self::RailCrew,
        Self::SailRepair,
        Self::EveningLights,
    ];

    pub fn name(self) -> &'static str {
        [
            "Kisten für Lina",
            "Nach dem Sturm",
            "Morgens am Wasser",
            "Ein Korb vom Hof",
            "Vorrat fürs Rathaus",
            "Für das Erzgleis",
            "Segel für den Hafen",
            "Licht auf dem Deich",
        ][self as usize]
    }

    fn activity(self) -> Option<(Activity, u32, &'static str)> {
        match self {
            Self::OrchardCrates => Some((Activity::Harvest, 8, "Pflanzen geerntet")),
            Self::AfterTheStorm => Some((Activity::Fell, 4, "Bäume gefällt")),
            Self::AQuietMorning => Some((Activity::Fish, 3, "Fische gefangen")),
            Self::RailCrew => Some((Activity::Mine, 5, "Erzadern abgebaut")),
            _ => None,
        }
    }

    fn goods(self) -> &'static [Need] {
        use Need::*;
        match self {
            Self::SeasonTable => &[
                Crop(crate::garden::Crop::Carrot, 3),
                Stock(Product::Apple, 2),
            ],
            Self::PantryBox => &[
                Stock(Product::Prepared(Dish::BerryJam), 2),
                Stock(Product::Prepared(Dish::HazelFlour), 1),
            ],
            Self::SailRepair => &[
                Stock(Product::Crafted(Recipe::Canvas), 2),
                Stock(Product::Crafted(Recipe::Rivets), 2),
            ],
            Self::EveningLights => &[
                Stock(Product::Crafted(Recipe::Oil), 2),
                Stock(Product::Crafted(Recipe::TideGlass), 1),
            ],
            _ => &[],
        }
    }

    fn available(self, g: &Garden) -> bool {
        match self {
            Self::SailRepair => {
                Watch::Wind.stage(g) >= Stage::Complete && Watch::Kiln.stage(g) >= Stage::Complete
            }
            Self::EveningLights => {
                Watch::Fen.stage(g) >= Stage::Complete && Watch::Tide.stage(g) >= Stage::Complete
            }
            _ => true,
        }
    }

    fn prize(self) -> u32 {
        [26, 15, 23, 29, 44, 34, 63, 78][self as usize]
    }

    pub fn cycle(day: u32) -> u32 {
        1 + day.saturating_sub(1) / 3
    }

    pub fn offers(g: &Garden) -> impl Iterator<Item = Self> + '_ {
        let current = Self::cycle(g.day);
        let start = (current as usize - 1) % Self::ALL.len();
        g.ledger.active.into_iter().chain(
            (0..Self::ALL.len())
                .map(move |offset| Self::ALL[(start + offset) % Self::ALL.len()])
                .filter(move |&order| {
                    order.available(g)
                        && g.ledger.active != Some(order)
                        && g.ledger.claimed[order as usize] != current
                })
                .take(3),
        )
    }

    pub fn detail(self, g: &Garden) -> String {
        let step = if let Some((_, want, label)) = self.activity() {
            let have = if g.ledger.active == Some(self) {
                g.ledger.progress
            } else {
                0
            };
            format!("{have}/{want} {label}")
        } else {
            craft::detail(self.goods(), g, 1)
        };
        format!("{step} · {} Münzen", self.prize())
    }

    pub fn action(self, g: &Garden) -> &'static str {
        if g.ledger.active == Some(self) {
            "Einlösen"
        } else {
            "Annehmen"
        }
    }

    pub fn can_work(self, g: &Garden) -> bool {
        if g.ledger.active == Some(self) {
            self.activity().map_or_else(
                || craft::ready(self.goods(), g, 1),
                |(_, need, _)| g.ledger.progress >= need,
            )
        } else {
            g.ledger.active.is_none() && Self::offers(g).any(|order| order == self)
        }
    }

    pub fn abandon(g: &mut Garden) -> bool {
        if g.ledger.active.take().is_none() {
            return false;
        }
        g.ledger.progress = 0;
        true
    }

    pub fn work(self, g: &mut Garden) -> Result<String, String> {
        if !self.can_work(g) {
            return Err(self.detail(g));
        }
        if g.ledger.active.is_none() {
            g.ledger.active = Some(self);
            g.ledger.progress = 0;
            return Ok(format!("Auftrag angenommen · {}", self.name()));
        }
        craft::take(self.goods(), g, 1);
        g.ledger.active = None;
        g.ledger.progress = 0;
        g.ledger.completed = g.ledger.completed.saturating_add(1);
        g.ledger.claimed[self as usize] = Self::cycle(g.day);
        g.farm.coins = g.farm.coins.saturating_add(self.prize());
        let seed = Crop::ALL[(self as usize + Self::cycle(g.day) as usize) % Crop::ALL.len()];
        g.add_seeds(seed, 2);
        Ok(format!(
            "Auftrag erledigt · +{} Münzen und 2 × {}",
            self.prize(),
            seed.name()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garden::Action;

    #[test]
    fn a_three_day_rotation_keeps_active_work_and_pays_only_once() {
        let mut g = Garden::default();
        let order = Order::AfterTheStorm;
        g.chop_tree(99);
        order.work(&mut g).unwrap();
        for id in [100, 101, 102] {
            assert!(g.chop_tree(id));
        }
        assert!(!order.can_work(&g));
        assert!(!g.chop_tree(102));
        for _ in 0..3 {
            g.sleep();
        }
        assert_eq!(Order::offers(&g).next(), Some(order));
        assert!(!order.can_work(&g));
        g.chop_tree(103);
        let coins = g.farm.coins;
        order.work(&mut g).unwrap();
        assert_eq!(g.farm.coins, coins + 15);
        assert_eq!(g.ledger.count(), 1);
        assert!(order.work(&mut g).is_err());
    }

    #[test]
    fn an_order_counts_real_harvests_after_watering_and_nights() {
        let mut g = Garden::default();
        let order = Order::OrchardCrates;
        order.work(&mut g).unwrap();
        g.add_seeds(Crop::Carrot, 8);
        for i in 0..8 {
            for task in [Action::Till, Action::Sow, Action::Water] {
                assert!(g.work(i, Crop::Carrot, task).is_some());
            }
        }
        g.sleep();
        for i in 0..8 {
            g.work(i, Crop::Carrot, Action::Water);
        }
        g.sleep();
        for i in 0..8 {
            assert!(g.work(i, Crop::Carrot, Action::Harvest).is_some());
        }
        order.work(&mut g).unwrap();
        assert_eq!(g.crop_total(Crop::Carrot), 24);
    }
}
