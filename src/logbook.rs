use serde::{Deserialize, Serialize};

use crate::{
    craft::Recipe,
    garden::{Crop, Garden},
    homestead::Product,
    processing::Dish,
    regions::Bell,
    waywatch::{Stage, Watch},
};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Logbook {
    crops: u8,
    parts: u8,
    dishes: u8,
    batches: u32,
    meals: u32,
    claimed: [u8; Path::ALL.len()],
}

impl Logbook {
    pub fn crop(&mut self, crop: Crop) {
        self.crops |= 1 << crop as u8;
    }

    pub fn part(&mut self, part: Recipe, batches: u32) {
        self.parts |= 1 << part as u8;
        self.batches = self.batches.saturating_add(batches);
    }

    pub fn meal(&mut self, dish: Dish) {
        self.dishes |= 1 << dish as u8;
        self.meals = self.meals.saturating_add(1);
    }

    pub fn recover_previous(garden: &mut Garden) {
        for crop in Crop::ALL {
            if garden.crop_total(crop) > 0 {
                garden.logbook.crop(crop);
            }
        }
        for part in Recipe::ALL {
            if garden.stock(Product::Crafted(part)) > 0 {
                garden.logbook.part(part, 1);
            }
        }
        for dish in Dish::ALL {
            if garden.stock(Product::Prepared(dish)) > 0 {
                garden.logbook.meal(dish);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Path {
    Garden,
    Workshop,
    Kitchen,
    Island,
    Duel,
}

impl Path {
    pub const ALL: [Self; 5] = [
        Self::Garden,
        Self::Workshop,
        Self::Kitchen,
        Self::Island,
        Self::Duel,
    ];

    pub fn title(self) -> &'static str {
        [
            "Gartenbau",
            "Werkstatt",
            "Hofküche",
            "Inselkunde",
            "Duellkunst",
        ][self as usize]
    }

    pub fn rank(self, garden: &Garden) -> u8 {
        garden.logbook.claimed[self as usize].min(3)
    }

    fn target(self, garden: &Garden) -> Option<(u32, u32, &'static str, u32, u32, &'static str)> {
        let step = usize::from(self.rank(garden));
        if step == 3 {
            return None;
        }
        let watches = || {
            Watch::ALL
                .into_iter()
                .filter(|watch| watch.stage(garden) == Stage::Complete)
                .count() as u32
        };
        Some(match self {
            Self::Garden => (
                garden.harvested / 3,
                [3, 12, 30][step],
                "Beete",
                garden.logbook.crops.count_ones(),
                [1, 3, 6][step],
                "Kulturarten",
            ),
            Self::Workshop => (
                garden.logbook.batches,
                [3, 9, 18][step],
                "Materialchargen",
                garden.logbook.parts.count_ones()
                    + garden.crafted.iter().filter(|&&made| made).count() as u32,
                [1, 4, 9][step],
                "Material- oder Möbelarten",
            ),
            Self::Kitchen => (
                garden.logbook.meals,
                [2, 7, 15][step],
                "Zubereitungen",
                garden.logbook.dishes.count_ones(),
                [1, 3, 6][step],
                "Rezepte",
            ),
            Self::Island => (
                Bell::ALL
                    .into_iter()
                    .filter(|bell| bell.ready(garden))
                    .count() as u32,
                [2, 4, 7][step],
                "Wegglocken",
                watches(),
                [1, 2, 5][step],
                "Wegwachten",
            ),
            Self::Duel => (
                garden.rpg.cleared,
                [3, 10, 20][step],
                "Dämmergang-Ebenen",
                watches(),
                [1, 2, 4][step],
                "Wegwachten",
            ),
        })
    }

    pub fn ready(self, garden: &Garden) -> bool {
        self.target(garden)
            .is_some_and(|(have, a, _, other, b, _)| have >= a && other >= b)
    }

    pub fn reward(self, at: usize) -> &'static str {
        match self {
            Self::Garden => [
                "Möhrensamen · Samen kehren häufiger zurück",
                "Erdbeersamen und 3 Dünger",
                "Tomatensamen · höhere Chance auf Saatgut",
            ][at],
            Self::Workshop => [
                "3 Bretter für dein erstes Möbelstück",
                "4 Kupfernieten für ausgebaute Stationen",
                "Eine zusätzliche Diele je Brettercharge",
            ][at],
            Self::Kitchen => [
                "4 Pilze und 2 Äpfel",
                "Eine zweite Portion bei sauberer Zubereitung",
                "5 Haselnüsse und 5 Sonnenblumensamen",
            ][at],
            Self::Island => [
                "12 Münzen und 2 Rübensamen",
                "26 Münzen und 3 Kürbissamen",
                "55 Münzen und 4 Sonnenblumensamen",
            ][at],
            Self::Duel => [
                "Ein Talentpunkt und etwas Proviant",
                "Zwei Talentpunkte",
                "Drei Talentpunkte und 30 Münzen",
            ][at],
        }
    }

    pub fn detail(self, garden: &Garden) -> String {
        self.target(garden).map_or_else(
            || "Meisterstufe erreicht".into(),
            |(h, a, t, k, b, u)| format!("{}/{} {t} · {}/{} {u}", h.min(a), a, k.min(b), b),
        )
    }

    pub fn claim(self, garden: &mut Garden) -> Result<String, String> {
        if !self.ready(garden) {
            return Err(self.detail(garden));
        }
        let step = usize::from(self.rank(garden));
        match (self, step) {
            (Self::Garden, 0) => garden.add_seeds(Crop::Carrot, 5),
            (Self::Garden, 1) => {
                garden.add_seeds(Crop::Strawberry, 5);
                garden.fertilizer = garden.fertilizer.saturating_add(3);
            }
            (Self::Garden, _) => garden.add_seeds(Crop::Tomato, 6),
            (Self::Workshop, 0) => garden.add_stock(Product::Crafted(Recipe::Boards), 3),
            (Self::Workshop, 1) => garden.add_stock(Product::Crafted(Recipe::Rivets), 4),
            (Self::Kitchen, 0) => {
                garden.add_stock(Product::Mushroom, 4);
                garden.add_stock(Product::Apple, 2);
            }
            (Self::Kitchen, 2) => {
                garden.add_stock(Product::Hazel, 5);
                garden.add_seeds(Crop::Sunflower, 5);
            }
            (Self::Island, _) => {
                garden.farm.coins = garden.farm.coins.saturating_add([12, 26, 55][step]);
                garden.add_seeds(
                    [Crop::Turnip, Crop::Pumpkin, Crop::Sunflower][step],
                    2 + step as u32,
                );
            }
            (Self::Duel, _) => {
                garden.rpg.points = garden.rpg.points.saturating_add(1 + step as u8);
                if step == 0 {
                    garden.add_stock(Product::Prepared(Dish::CarrotSoup), 1);
                } else if step == 2 {
                    garden.farm.coins = garden.farm.coins.saturating_add(30);
                }
            }
            (Self::Workshop | Self::Kitchen, _) => {}
        }
        garden.logbook.claimed[self as usize] = (step + 1) as u8;
        Ok(format!(
            "{} · Rang {} · {}",
            self.title(),
            step + 1,
            self.reward(step)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reward_requires_both_practice_and_variety() {
        let mut garden = Garden::default();
        garden.harvested = 90;
        garden.logbook.crop(Crop::Carrot);
        assert!(Path::Garden.claim(&mut garden).is_ok());
        assert_eq!(Path::Garden.rank(&garden), 1);
        assert!(Path::Garden.claim(&mut garden).is_err());
        garden.logbook.crop(Crop::Pumpkin);
        garden.logbook.crop(Crop::Sunflower);
        assert!(Path::Garden.claim(&mut garden).is_ok());
        assert_eq!(garden.fertilizer, 6);
        assert!(Path::Garden.claim(&mut garden).is_err());
        for crop in Crop::ALL {
            garden.logbook.crop(crop);
        }
        assert!(Path::Garden.claim(&mut garden).is_ok());
        assert_eq!(Path::Garden.rank(&garden), 3);
        assert!(Path::Garden.claim(&mut garden).is_err());
    }
}
