use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    adventure::{Area, Target},
    duel::Loot,
    garden::{Crop, Garden, Quality},
    processing::Dish,
    regions::Material,
    workshop::ToolKind,
};

pub const SEED_SHOP: Vec2 = Vec2::new(16.15, 3.02);
pub const BUYER: Vec2 = Vec2::new(20.05, 3.21);
pub const BOARD: Vec2 = Vec2::new(16.05, 6.24);
pub const SMITH: Vec2 = Vec2::new(20.02, 6.58);
pub const FEED: Vec2 = Vec2::new(-10.35, -8.37);
pub const HEN: Vec2 = Vec2::new(-12.56, -9.17);
pub const SHEEP: Vec2 = Vec2::new(-7.63, -9.41);
pub const ORCHARD: [Vec2; 7] = [
    Vec2::new(-12.35, 7.80),
    Vec2::new(-11.05, 5.30),
    Vec2::new(-8.10, 8.90),
    Vec2::new(-14.40, 9.80),
    Vec2::new(-13.30, 11.50),
    Vec2::new(-9.70, 10.80),
    Vec2::new(-15.0, 6.50),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Village {
    Seeds,
    Sell,
    Board,
    Smith,
    Feed,
    Animal(Animal),
    Orchard(usize),
}

impl Village {
    pub fn closed(self, garden: &Garden) -> bool {
        matches!(self, Self::Seeds | Self::Sell | Self::Smith) && !garden.clock.market_open()
    }

    pub fn all() -> impl Iterator<Item = Self> {
        [
            Self::Seeds,
            Self::Sell,
            Self::Board,
            Self::Smith,
            Self::Feed,
            Self::Animal(Animal::Hen),
            Self::Animal(Animal::Sheep),
        ]
        .into_iter()
        .chain((0..ORCHARD.len()).map(Self::Orchard))
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Seeds => "Saatgutstand",
            Self::Sell => "Hafenankauf",
            Self::Board => "Schwarzes Brett",
            Self::Smith => "Hafenschmiede",
            Self::Feed => "Futterkiste",
            Self::Animal(Animal::Hen) => "Hühnerstall",
            Self::Animal(Animal::Sheep) => "Schafweide",
            Self::Orchard(i) if i < 4 => "Apfelbaum",
            Self::Orchard(_) => "Beerenstrauch",
        }
    }

    pub fn position(self) -> Vec2 {
        match self {
            Self::Seeds => SEED_SHOP,
            Self::Sell => BUYER,
            Self::Board => BOARD,
            Self::Smith => SMITH,
            Self::Feed => FEED,
            Self::Animal(Animal::Hen) => HEN,
            Self::Animal(Animal::Sheep) => SHEEP,
            Self::Orchard(i) => ORCHARD[i],
        }
    }

    pub fn height(self) -> f32 {
        match self {
            Self::Seeds | Self::Sell | Self::Board | Self::Smith => 0.90,
            Self::Orchard(i) if i < 4 => 1.44,
            Self::Animal(_) => 0.70,
            _ => 0.38,
        }
    }

    pub fn describe(
        self,
        g: &Garden,
        crop: Crop,
        item: Option<Product>,
    ) -> (String, &'static str, bool) {
        let farm = &g.farm;
        let money = farm.coins;
        if self.closed(g) {
            return ("Wieder geöffnet ab 06:00.".into(), "Geschlossen", false);
        }
        match self {
            Self::Seeds => (
                format!("3 {}-Samen · {} Münzen", crop.name(), crop.price()),
                "Saat kaufen",
                money >= crop.price(),
            ),
            Self::Sell => item.map_or_else(
                || {
                    (
                        "Ernte, sammle oder fange zuerst etwas.".into(),
                        "Verkaufen",
                        false,
                    )
                },
                |item| {
                    (
                        format!(
                            "{} ×{} · +{} Münzen",
                            item.label(),
                            g.stock(item),
                            item.value()
                        ),
                        "Verkaufen",
                        true,
                    )
                },
            ),
            Self::Board => (
                g.task().0.into(),
                if g.task().1 {
                    "Belohnung"
                } else {
                    "Aufgabe lesen"
                },
                true,
            ),
            Self::Feed => (
                format!("4 Futter für 5 Münzen · Vorrat {}", farm.feed),
                "Futter kaufen",
                money >= 5,
            ),
            Self::Smith if ToolKind::Pickaxe.level(g) > 0 => (
                format!(
                    "Spitzhacke · {} · +{} Erz je Ader",
                    ToolKind::Pickaxe.tier(g),
                    ToolKind::Pickaxe.level(g)
                ),
                "Verbessert",
                false,
            ),
            Self::Smith => (
                "Kupferspitzhacke · 2 Kupfer, 2 Holz, 12 Münzen".into(),
                "Verbessern",
                money >= 12 && g.copper >= 2 && g.wood >= 2,
            ),
            Self::Animal(kind) => {
                let state = farm.animals[kind as usize];
                if !farm.housing[kind as usize] {
                    (
                        format!(
                            "Baue auf deinem Hof zuerst das Gebäude „{}“.",
                            kind.housing_name()
                        ),
                        "Stall am Hof",
                        false,
                    )
                } else if !state.owned {
                    (
                        format!(
                            "{} bekommt ein Zuhause · {} Münzen",
                            kind.name(),
                            kind.price()
                        ),
                        "Adoptieren",
                        money >= kind.price(),
                    )
                } else if state.pending > 0 {
                    (
                        format!(
                            "{} liegt bereit · {} Futter",
                            kind.product().name(),
                            farm.feed
                        ),
                        "Einsammeln",
                        true,
                    )
                } else if state.fed_day == Some(g.day) && state.petted_day != Some(g.day) {
                    (
                        format!("Gefüttert · Zutrauen {} / 3", state.affection),
                        "Streicheln",
                        true,
                    )
                } else if state.fed_day == Some(g.day) {
                    (
                        "Heute gefüttert · morgen nachsehen".into(),
                        "Versorgt",
                        false,
                    )
                } else {
                    (
                        format!("Einmal täglich füttern · {} Gras · {} Futter", farm.grass, farm.feed),
                        "Füttern",
                        farm.grass > 0 || farm.feed > 0,
                    )
                }
            }
            Self::Orchard(_)
                if crate::weather::Season::at(g.day) == crate::weather::Season::Winter =>
            {
                (
                    "Die Obstwiese ruht bis zum Frühling.".into(),
                    "Winterruhe",
                    false,
                )
            }
            Self::Orchard(i) if farm.fruit_ready(i, g.day) => (
                "Früchte wachsen nach zwei Tagen nach.".into(),
                "Pflücken",
                true,
            ),
            Self::Orchard(i) => (
                format!(
                    "Wächst bis Tag {}",
                    farm.fruit_days[i].unwrap_or(g.day).saturating_add(2)
                ),
                "Wächst nach",
                false,
            ),
        }
    }

    pub fn work(self, g: &mut Garden, crop: Crop, item: Option<Product>) -> Result<String, String> {
        let (detail, _, enabled) = self.describe(g, crop, item);
        if !enabled {
            return Err(detail);
        }
        match self {
            Self::Seeds => {
                g.farm.coins -= crop.price();
                g.add_seeds(crop, 3);
                g.farm.trades = g.farm.trades.saturating_add(1);
                Ok(format!("+3 Saatgut · {}", crop.name()))
            }
            Self::Sell => {
                let item = item.ok_or_else(|| "Keine Waren in der Tasche.".to_owned())?;
                if !g.sell(item) {
                    return Err("Ware aufgebraucht.".into());
                }
                Ok(format!(
                    "{} verkauft · +{} Münzen",
                    item.name(),
                    item.value()
                ))
            }
            Self::Board => {
                if g.claim() {
                    Ok("Aufgabe erledigt · Münzen und Saatgut".into())
                } else {
                    Err(detail)
                }
            }
            Self::Smith => {
                g.farm.coins -= 12;
                g.copper -= 2;
                g.wood -= 2;
                g.farm.pickaxe = true;
                Ok("Kupferspitzhacke geschmiedet.".into())
            }
            Self::Feed => {
                g.farm.coins -= 5;
                g.farm.feed = g.farm.feed.saturating_add(4);
                Ok("+4 Futter".into())
            }
            Self::Animal(kind) => {
                let state = &mut g.farm.animals[kind as usize];
                if !state.owned {
                    g.farm.coins -= kind.price();
                    state.owned = true;
                    return Ok(format!("{} ist eingezogen.", kind.name()));
                }
                if state.pending > 0 {
                    let quantity = std::mem::take(&mut state.pending);
                    g.farm.collected = g.farm.collected.saturating_add(quantity);
                    g.add_stock(kind.product(), quantity);
                    if kind == Animal::Sheep {
                        g.add_stock(Product::Milk, quantity);
                        return Ok(format!("+{quantity} Wolle · +{quantity} Schafmilch"));
                    }
                    return Ok(format!("+{quantity} {}", kind.product().name()));
                }
                if state.fed_day == Some(g.day) {
                    state.petted_day = Some(g.day);
                    state.affection = state.affection.saturating_add(1).min(3);
                    return Ok(format!(
                        "{} freut sich · Zutrauen {}/3",
                        kind.name(),
                        state.affection
                    ));
                }
                state.fed_day = Some(g.day);
                let food = if g.farm.grass > 0 {
                    g.farm.grass -= 1;
                    "Gras"
                } else {
                    g.farm.feed -= 1;
                    "Futter"
                };
                Ok(format!("{} hat {} gefressen. Schau morgen wieder nach.", kind.name(), food))
            }
            Self::Orchard(i) => {
                g.farm.fruit_days[i] = Some(g.day);
                g.farm.fruit_picked = g.farm.fruit_picked.saturating_add(2);
                let fruit = if i < 4 {
                    Product::Apple
                } else {
                    Product::Berry
                };
                g.add_stock(fruit, 2);
                Ok(format!("+2 {}", fruit.name()))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Animal {
    Hen,
    Sheep,
}

impl Animal {
    pub const ALL: [Self; 2] = [Self::Hen, Self::Sheep];
    pub fn name(self) -> &'static str {
        match self {
            Self::Hen => "Huhn",
            Self::Sheep => "Schaf",
        }
    }
    pub fn price(self) -> u32 {
        match self {
            Self::Hen => 9,
            Self::Sheep => 17,
        }
    }
    pub fn product(self) -> Product {
        match self {
            Self::Hen => Product::Egg,
            Self::Sheep => Product::Wool,
        }
    }

    pub fn site(self) -> Vec2 {
        match self {
            Self::Hen => Vec2::new(-6.4, 9.4),
            Self::Sheep => Vec2::new(13.05, 8.83),
        }
    }

    pub fn housing_name(self) -> &'static str {
        match self {
            Self::Hen => "Hühnerstall",
            Self::Sheep => "Schafscheune",
        }
    }

    pub fn housing_model(self) -> &'static str {
        match self {
            Self::Hen => "farm_coop",
            Self::Sheep => "farm_barn",
        }
    }

    fn housing_cost(self) -> (u32, u32) {
        match self {
            Self::Hen => (5, 3),
            Self::Sheep => (9, 6),
        }
    }

    pub fn housing_hint(self, garden: &Garden) -> (String, bool) {
        if !garden.workshop.barns[self as usize] {
            return ("Kaufe den Bauplan am Bauplanstand.".into(), false);
        }
        let (wood, stone) = self.housing_cost();
        (
            format!(
                "{}/{} Holz · {}/{} Stein",
                garden.wood.min(wood),
                wood,
                garden.stone.min(stone),
                stone
            ),
            garden.wood >= wood && garden.stone >= stone,
        )
    }

    pub fn build_housing(self, garden: &mut Garden) -> Result<String, String> {
        let (hint, enough) = self.housing_hint(garden);
        if garden.farm.housing[self as usize] {
            return Err("Das Gebäude steht bereits.".into());
        }
        if !enough {
            return Err(hint);
        }
        let (wood, stone) = self.housing_cost();
        garden.wood -= wood;
        garden.stone -= stone;
        garden.farm.housing[self as usize] = true;
        Ok(format!(
            "{} steht. Ein {} kann einziehen.",
            self.housing_name(),
            self.name()
        ))
    }

    fn cycle(self) -> u8 {
        match self {
            Self::Hen => 1,
            Self::Sheep => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct AnimalState {
    pub owned: bool,
    fed_day: Option<u32>,
    petted_day: Option<u32>,
    affection: u8,
    care: u8,
    pending: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Product {
    Crop(Crop, Quality),
    Prepared(Dish),
    Crafted(crate::craft::Recipe),
    Regional(Material),
    Spoils(Loot),
    Seasonal(crate::weather::Season),
    Fish,
    Herring,
    Apple,
    Berry,
    Egg,
    Wool,
    Silver,
    Crystal,
    Mushroom,
    Hazel,
    Shell,
    Grass,
    Milk,
}

impl Product {
    const GATHERED: [Self; 13] = [
        Self::Fish,
        Self::Herring,
        Self::Apple,
        Self::Berry,
        Self::Egg,
        Self::Wool,
        Self::Silver,
        Self::Crystal,
        Self::Mushroom,
        Self::Hazel,
        Self::Shell,
        Self::Grass,
        Self::Milk,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Crop(crop, _) => crop.name(),
            Self::Prepared(dish) => dish.name(),
            Self::Crafted(recipe) => recipe.name(),
            Self::Regional(material) => material.name(),
            Self::Spoils(loot) => loot.name(),
            Self::Seasonal(season) => season.find_name(),
            Self::Fish => "Fisch",
            Self::Herring => "Hafenhering",
            Self::Apple => "Äpfel",
            Self::Berry => "Waldbeeren",
            Self::Egg => "Eier",
            Self::Wool => "Wolle",
            Self::Silver => "Silber",
            Self::Crystal => "Kristalle",
            Self::Mushroom => "Pilze",
            Self::Hazel => "Haselnüsse",
            Self::Shell => "Muscheln",
            Self::Grass => "Wildgras",
            Self::Milk => "Schafmilch",
        }
    }
    pub fn quality(self) -> Option<Quality> {
        match self {
            Self::Crop(_, quality) => Some(quality),
            _ => None,
        }
    }

    pub fn label(self) -> String {
        match self.quality() {
            Some(quality) => format!("{} · {}", self.name(), quality.name()),
            None => self.name().into(),
        }
    }

    pub fn value(self) -> u32 {
        match self {
            Self::Crop(crop, quality) => quality.price(crop.price().div_ceil(2)),
            Self::Prepared(dish) => dish.value(),
            Self::Crafted(recipe) => recipe.value(),
            Self::Regional(material) => material.value(),
            Self::Spoils(loot) => loot.value(),
            Self::Seasonal(_) => 7,
            Self::Fish | Self::Egg => 5,
            Self::Herring => 9,
            Self::Apple | Self::Berry => 2,
            Self::Wool => 8,
            Self::Silver => 9,
            Self::Crystal => 13,
            Self::Mushroom | Self::Hazel => 3,
            Self::Shell => 4,
            Self::Grass => 1,
            Self::Milk => 6,
        }
    }

    pub fn stocked(garden: &Garden) -> impl Iterator<Item = Self> + '_ {
        Crop::ALL
            .into_iter()
            .flat_map(|crop| Quality::ALL.map(move |quality| Self::Crop(crop, quality)))
            .chain(Self::GATHERED)
            .chain(Dish::ALL.map(Self::Prepared))
            .chain(crate::craft::Recipe::ALL.map(Self::Crafted))
            .chain(Material::ALL.map(Self::Regional))
            .chain(Loot::ALL.map(Self::Spoils))
            .chain(crate::weather::Season::ALL.map(Self::Seasonal))
            .filter(|&item| garden.stock(item) > 0)
    }

    pub fn at(index: usize, garden: &Garden) -> Option<Self> {
        Self::stocked(garden).nth(index)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Homestead {
    pub coins: u32,
    pub feed: u32,
    pub grass: u32,
    pub milk: u32,
    pub animals: [AnimalState; 2],
    pub housing: [bool; 2],
    pub fruit_days: [Option<u32>; 7],
    pub goods: [u32; 6],
    pub seeds: [u32; 3],
    pub yields: [u32; 3],
    pub trades: u32,
    pub fruit_picked: u32,
    pub collected: u32,
    pub crystal_mined: u32,
    pub pickaxe: bool,
}

impl Default for Homestead {
    fn default() -> Self {
        Self {
            coins: 20,
            feed: 3,
            grass: 0,
            milk: 0,
            animals: [AnimalState::default(); 2],
            housing: [false; 2],
            fruit_days: [None; 7],
            goods: [0; 6],
            seeds: [3; 3],
            yields: [0; 3],
            trades: 0,
            fruit_picked: 0,
            collected: 0,
            crystal_mined: 0,
            pickaxe: false,
        }
    }
}

impl Homestead {
    pub fn fruit_ready(&self, i: usize, day: u32) -> bool {
        crate::weather::Season::at(day) != crate::weather::Season::Winter
            && self.fruit_days[i].is_none_or(|last| day.saturating_sub(last) >= 2)
    }

    pub fn end_day(&mut self, day: u32) {
        for animal in Animal::ALL {
            let state = &mut self.animals[animal as usize];
            if state.owned && state.fed_day == Some(day) {
                state.care = state.care.saturating_add(1);
                if state.care >= animal.cycle() {
                    state.care = 0;
                    state.pending = state
                        .pending
                        .saturating_add(1 + u32::from(state.affection == 3));
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum Ore {
    Copper,
    Silver,
    Crystal,
}

impl Ore {
    pub fn tool_level(self) -> u8 {
        match self {
            Self::Copper => 0,
            Self::Silver => 1,
            Self::Crystal => 2,
        }
    }

    pub fn at(depth: u32, cell: u16) -> Self {
        if depth >= 3 {
            return match u32::from(cell).wrapping_add(depth) % 5 {
                0 | 1 => Self::Crystal,
                2 | 3 => Self::Silver,
                _ => Self::Copper,
            };
        }
        if depth == 2 && cell % 3 == 1 {
            Self::Silver
        } else {
            Self::Copper
        }
    }
    pub fn model(self) -> &'static str {
        match self {
            Self::Copper => "resource_copper",
            Self::Silver => "resource_silver",
            Self::Crystal => "resource_crystal",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Copper => "Kupferader",
            Self::Silver => "Silberader",
            Self::Crystal => "Kristallnest",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Copper => "Kupfer",
            Self::Silver => "Silber",
            Self::Crystal => "Kristalle",
        }
    }
    pub fn mine(self, g: &mut Garden) -> u32 {
        g.ledger.record(crate::commissions::Activity::Mine, 1);
        let quantity = 1 + u32::from(crate::workshop::ToolKind::Pickaxe.level(g));
        match self {
            Self::Copper => g.copper = g.copper.saturating_add(quantity),
            Self::Silver => g.add_stock(Product::Silver, quantity),
            Self::Crystal => {
                g.add_stock(Product::Crystal, quantity);
                g.farm.crystal_mined = g.farm.crystal_mined.saturating_add(quantity);
            }
        }
        g.stone = g.stone.saturating_add(1);
        quantity
    }
}

pub fn choice_label(
    area: Area,
    focus: Option<Target>,
    item: Option<Product>,
) -> Option<&'static str> {
    if area == Area::Island && focus == Some(Target::Village(Village::Sell)) {
        return Some(item.map_or("Verkauf", Product::name));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wild_grass_feeds_a_sheep_and_care_produces_milk() {
        let mut garden = Garden::default();
        garden.farm.housing[Animal::Sheep as usize] = true;
        garden.farm.animals[Animal::Sheep as usize].owned = true;
        garden.add_stock(Product::Grass, 2);
        let feed = Village::Animal(Animal::Sheep);
        let before = garden.farm.feed;
        for _ in 0..2 {
            feed.work(&mut garden, Crop::Carrot, None).unwrap();
            garden.sleep();
        }
        assert_eq!(garden.stock(Product::Grass), 0);
        assert_eq!(garden.farm.feed, before);
        feed.work(&mut garden, Crop::Carrot, None).unwrap();
        assert_eq!(garden.stock(Product::Milk), 1);
        assert_eq!(garden.stock(Product::Wool), 1);
    }

    #[test]
    fn fruit_returns_and_can_fund_older_game_saves() {
        let mut g = Garden::default();
        assert!(Village::Orchard(0).work(&mut g, Crop::Carrot, None).is_ok());
        assert_eq!(g.stock(Product::Apple), 2);
        assert!(
            Village::Orchard(0)
                .work(&mut g, Crop::Carrot, None)
                .is_err()
        );
        let coins = g.farm.coins;
        assert!(
            Village::Sell
                .work(&mut g, Crop::Carrot, Some(Product::Apple))
                .is_ok()
        );
        assert_eq!((g.stock(Product::Apple), g.farm.coins), (1, coins + 2));
        g.sleep();
        assert!(!g.farm.fruit_ready(0, g.day));
        g.sleep();
        assert!(g.farm.fruit_ready(0, g.day));
    }

    #[test]
    fn animals_only_produce_after_fed_days() {
        let mut g = Garden::default();
        g.farm.housing[Animal::Hen as usize] = true;
        let hen = Village::Animal(Animal::Hen);
        hen.work(&mut g, Crop::Carrot, None).unwrap();
        g.sleep();
        assert!(hen.describe(&g, Crop::Carrot, None).0.contains("füttern"));
        hen.work(&mut g, Crop::Carrot, None).unwrap();
        assert!(
            hen.work(&mut g, Crop::Carrot, None)
                .unwrap()
                .contains("Zutrauen")
        );
        assert!(hen.work(&mut g, Crop::Carrot, None).is_err());
        g.sleep();
        hen.work(&mut g, Crop::Carrot, None).unwrap();
        assert_eq!(g.stock(Product::Egg), 1);
    }

    #[test]
    fn adopting_an_animal_requires_its_own_building() {
        let mut g = Garden::default();
        let sheep = Village::Animal(Animal::Sheep);
        assert!(sheep.work(&mut g, Crop::Carrot, None).is_err());
        g.wood = 9;
        g.stone = 6;
        assert!(Animal::Sheep.build_housing(&mut g).is_err());
        assert_eq!((g.wood, g.stone), (9, 6));
        g.farm.coins = 45;
        crate::workshop::BuildingPlan::Barn(Animal::Sheep)
            .buy(&mut g)
            .unwrap();
        assert_eq!(g.farm.coins, 18);
        assert!(Animal::Sheep.build_housing(&mut g).is_ok());
        assert_eq!((g.wood, g.stone), (0, 0));
        assert!(sheep.work(&mut g, Crop::Carrot, None).is_ok());
        assert!(g.farm.animals[Animal::Sheep as usize].owned);
        assert!(!g.farm.housing[Animal::Hen as usize]);
    }

    #[test]
    fn wool_needs_two_fed_days_and_can_be_sold() {
        let mut g = Garden::default();
        g.farm.housing[Animal::Sheep as usize] = true;
        let pen = Village::Animal(Animal::Sheep);
        pen.work(&mut g, Crop::Carrot, None).unwrap();
        let coins = g.farm.coins;
        pen.work(&mut g, Crop::Carrot, None).unwrap();
        g.sleep();
        assert_eq!(g.farm.animals[Animal::Sheep as usize].pending, 0);
        pen.work(&mut g, Crop::Carrot, None).unwrap();
        g.sleep();
        pen.work(&mut g, Crop::Carrot, None).unwrap();
        assert_eq!(g.stock(Product::Wool), 1);
        assert!(g.sell(Product::Wool));
        assert_eq!(g.farm.coins, coins + 8);
    }

    #[test]
    fn upgrade_spends_exact_materials_and_increases_ore() {
        let mut g = Garden::default();
        g.copper = 2;
        g.wood = 2;
        Village::Smith.work(&mut g, Crop::Carrot, None).unwrap();
        assert_eq!((g.copper, g.wood, g.farm.coins), (0, 0, 8));
        assert_eq!(Ore::Crystal.mine(&mut g), 2);
        assert_eq!(g.stock(Product::Crystal), 2);
    }
}
