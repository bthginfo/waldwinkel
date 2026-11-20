use crate::{
    homestead::{Homestead, Product},
    session::Checkpoint,
    town::Journal,
    weather::Sky,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const PLOT_COUNT: usize = 12;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub enum Crop {
    #[default]
    Carrot,
    Turnip,
    Pumpkin,
    Strawberry,
    Tomato,
    Sunflower,
}

impl Crop {
    pub const ALL: [Self; 6] = [
        Self::Carrot,
        Self::Turnip,
        Self::Pumpkin,
        Self::Strawberry,
        Self::Tomato,
        Self::Sunflower,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Carrot => "Möhren",
            Self::Turnip => "Rüben",
            Self::Pumpkin => "Kürbisse",
            Self::Strawberry => "Erdbeeren",
            Self::Tomato => "Tomaten",
            Self::Sunflower => "Sonnenblumen",
        }
    }

    pub fn price(self) -> u32 {
        [5, 6, 8, 7, 8, 9][self as usize]
    }

    pub fn days(self) -> u8 {
        [2, 3, 4, 2, 3, 5][self as usize]
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub enum Plot {
    #[default]
    Grass,
    Tilled,
    Planted {
        at_ms: u64,
    },
    Sown {
        crop: Crop,
        growth: u8,
        wet: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Grass,
    Soil,
    Sprout,
    Young,
    Ripe,
}

impl Plot {
    pub fn stage(self) -> Stage {
        match self {
            Self::Grass => Stage::Grass,
            Self::Tilled => Stage::Soil,
            Self::Planted { at_ms } if now_ms().saturating_sub(at_ms) >= 18_000 => Stage::Ripe,
            Self::Planted { .. } => Stage::Sprout,
            Self::Sown { crop, growth, .. } if growth >= crop.days() => Stage::Ripe,
            Self::Sown { growth: 0, .. } => Stage::Sprout,
            Self::Sown { .. } => Stage::Young,
        }
    }

    pub fn crop(self) -> Crop {
        match self {
            Self::Sown { crop, .. } => crop,
            _ => Crop::Carrot,
        }
    }

    pub fn remaining(self) -> Option<(u8, bool)> {
        match self {
            Self::Sown { crop, growth, wet } => Some((crop.days().saturating_sub(growth), wet)),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Till,
    Sow,
    Water,
    Harvest,
}

impl Action {
    pub fn label(self) -> &'static str {
        match self {
            Self::Till => "Beet anlegen",
            Self::Sow => "Aussäen",
            Self::Water => "Gießen",
            Self::Harvest => "Ernten",
        }
    }
}

pub const FURNITURE_COUNT: usize = 6;

#[derive(Resource, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Garden {
    version: u8,
    pub plots: Vec<Plot>,
    pub seeds: u32,
    pub harvested: u32,
    pub day: u32,
    pub extra_seeds: [u32; 2],
    pub yields: [u32; 3],
    pub wood: u32,
    pub stone: u32,
    pub copper: u32,
    pub fish: u32,
    pub felled: Vec<usize>,
    pub broken: Vec<usize>,
    pub mined: std::collections::BTreeSet<(u32, u16)>,
    pub deepest: u32,
    pub furniture: [u8; FURNITURE_COUNT],
    pub crafted: [bool; 3],
    pub challenge: usize,
    pub farm: Homestead,
    pub checkpoint: Option<Checkpoint>,
    pub town: Journal,
}

impl Default for Garden {
    fn default() -> Self {
        Self {
            version: 5,
            plots: vec![Plot::Grass; PLOT_COUNT],
            seeds: 6,
            harvested: 0,
            day: 1,
            extra_seeds: [4, 4],
            yields: [0; 3],
            wood: 0,
            stone: 0,
            copper: 0,
            fish: 0,
            felled: Vec::new(),
            broken: Vec::new(),
            mined: Default::default(),
            deepest: 0,
            furniture: [0; FURNITURE_COUNT],
            crafted: [true, false, false],
            challenge: 0,
            farm: Homestead::default(),
            checkpoint: None,
            town: Journal::default(),
        }
    }
}

impl Garden {
    fn seed_slot(&mut self, crop: Crop) -> &mut u32 {
        match crop {
            Crop::Carrot => &mut self.seeds,
            Crop::Turnip => &mut self.extra_seeds[0],
            Crop::Pumpkin => &mut self.extra_seeds[1],
            _ => &mut self.farm.seeds[crop as usize - 3],
        }
    }

    pub fn seed_count(&self, crop: Crop) -> u32 {
        let index = crop as usize;
        if index < 3 {
            [self.seeds, self.extra_seeds[0], self.extra_seeds[1]][index]
        } else {
            self.farm.seeds[index - 3]
        }
    }

    pub fn add_seeds(&mut self, crop: Crop, amount: u32) {
        let seeds = self.seed_slot(crop);
        *seeds = seeds.saturating_add(amount);
    }

    fn stock_slot(&mut self, item: Product) -> &mut u32 {
        match item {
            Product::Crop(crop) if (crop as usize) < 3 => &mut self.yields[crop as usize],
            Product::Crop(crop) => &mut self.farm.yields[crop as usize - 3],
            Product::Fish => &mut self.fish,
            Product::Mushroom => &mut self.town.goods[0],
            Product::Hazel => &mut self.town.goods[1],
            Product::Shell => &mut self.town.goods[2],
            _ => &mut self.farm.goods[Self::goods_index(item)],
        }
    }

    pub fn stock(&self, item: Product) -> u32 {
        match item {
            Product::Crop(crop) if (crop as usize) < 3 => self.yields[crop as usize],
            Product::Crop(crop) => self.farm.yields[crop as usize - 3],
            Product::Fish => self.fish,
            Product::Mushroom => self.town.goods[0],
            Product::Hazel => self.town.goods[1],
            Product::Shell => self.town.goods[2],
            _ => self.farm.goods[Self::goods_index(item)],
        }
    }

    fn goods_index(item: Product) -> usize {
        match item {
            Product::Apple => 0,
            Product::Berry => 1,
            Product::Egg => 2,
            Product::Wool => 3,
            Product::Silver => 4,
            Product::Crystal => 5,
            _ => unreachable!(),
        }
    }

    pub fn add_stock(&mut self, item: Product, quantity: u32) {
        let stock = self.stock_slot(item);
        *stock = stock.saturating_add(quantity);
    }

    pub fn take_stock(&mut self, item: Product, quantity: u32) -> bool {
        let stock = self.stock_slot(item);
        let Some(remaining) = stock.checked_sub(quantity) else {
            return false;
        };
        *stock = remaining;
        true
    }

    pub fn sell(&mut self, item: Product) -> bool {
        if !self.take_stock(item, 1) {
            return false;
        }
        self.farm.coins = self.farm.coins.saturating_add(item.value());
        self.farm.trades = self.farm.trades.saturating_add(1);
        true
    }

    pub fn action(&self, index: usize, crop: Crop) -> Option<Action> {
        let plot = *self.plots.get(index)?;
        match plot.stage() {
            Stage::Grass => Some(Action::Till),
            Stage::Soil if self.seed_count(crop) > 0 => Some(Action::Sow),
            Stage::Ripe => Some(Action::Harvest),
            _ if matches!(plot, Plot::Sown { wet: false, .. }) => Some(Action::Water),
            _ => None,
        }
    }

    pub fn work(&mut self, index: usize, crop: Crop) -> Option<(Action, String)> {
        let action = self.action(index, crop)?;
        let plot = self.plots[index];
        let (new, message) = match action {
            Action::Till => (Plot::Tilled, "Beet angelegt.".into()),
            Action::Sow => {
                *self.seed_slot(crop) -= 1;
                (
                    Plot::Sown {
                        crop,
                        growth: 0,
                        wet: Sky::at(self.day) == Sky::Rain,
                    },
                    format!("{} gesät.", crop.name()),
                )
            }
            Action::Water => {
                let Plot::Sown { crop, growth, .. } = plot else {
                    return None;
                };
                (
                    Plot::Sown {
                        crop,
                        growth,
                        wet: true,
                    },
                    "Gegossen. Wächst bis morgen.".into(),
                )
            }
            Action::Harvest => {
                let crop = plot.crop();
                self.harvested = self.harvested.saturating_add(3);
                self.add_stock(Product::Crop(crop), 3);
                self.add_seeds(crop, 1);
                (Plot::Tilled, format!("+3 {} · +1 Saatgut", crop.name()))
            }
        };
        self.plots[index] = new;
        Some((action, message))
    }

    pub fn sleep(&mut self) {
        let rain = Sky::at(self.day) == Sky::Rain;
        self.farm.end_day(self.day);
        self.day = self.day.saturating_add(1);
        let wet_next = Sky::at(self.day) == Sky::Rain;
        for plot in &mut self.plots {
            if let Plot::Sown { crop, growth, wet } = *plot {
                *plot = Plot::Sown {
                    crop,
                    growth: growth
                        .saturating_add(u8::from(wet || rain))
                        .min(crop.days()),
                    wet: wet_next,
                };
            }
        }
    }

    pub fn task(&self) -> (&'static str, bool) {
        match self.challenge {
            0 => ("Ernte eine beliebige Kultur", self.harvested >= 3),
            1 => (
                "Sammle 3 Holz und 2 Stein",
                self.wood >= 3 && self.stone >= 2,
            ),
            2 => ("Fange einen Fisch am Steg", self.fish >= 1),
            3 => (
                "Finde Kupfer und erreiche Tiefe 2",
                self.copper >= 1 && self.deepest >= 2,
            ),
            4 => (
                "Pflücke 4 Früchte auf der Obstwiese",
                self.farm.fruit_picked >= 4,
            ),
            5 => (
                "Kaufe oder verkaufe dreimal am Markt",
                self.farm.trades >= 3,
            ),
            6 => ("Sammle ein Ei oder Wolle ein", self.farm.collected >= 1),
            7 => (
                "Finde einen Kristall in der Tiefenmine",
                self.farm.crystal_mined >= 1,
            ),
            _ => ("Alle Aufgaben abgeschlossen", false),
        }
    }

    pub fn claim(&mut self) -> bool {
        if !self.task().1 {
            return false;
        }
        self.challenge += 1;
        for crop in Crop::ALL {
            self.add_seeds(crop, 2);
        }
        self.farm.coins = self.farm.coins.saturating_add(5);
        true
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        let mut garden: Self = serde_json::from_slice(&fs::read(path)?)?;
        if !matches!(garden.version, 1..=5) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unbekannte Spielstandversion",
            ));
        }
        garden.plots.resize(PLOT_COUNT, Plot::Grass);
        if Sky::at(garden.day) == Sky::Rain {
            for plot in &mut garden.plots {
                if let Plot::Sown { wet, .. } = plot {
                    *wet = true;
                }
            }
        }
        if garden.version <= 4 {
            garden.checkpoint = garden.checkpoint.map(Checkpoint::move_to_mainland);
        }
        garden.version = 5;
        Ok(garden)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, serde_json::to_vec(self)?)?;
        fs::rename(&tmp, path)
    }
}

#[derive(Resource)]
pub struct SavePath(pub PathBuf);

impl SavePath {
    pub fn for_device() -> Self {
        #[cfg(target_os = "android")]
        let directory = bevy::window::ANDROID_APP
            .get()
            .and_then(|app| app.internal_data_path())
            .expect("Android NativeActivity must provide its private files directory");

        #[cfg(not(target_os = "android"))]
        let directory = std::env::var_os("WALDWINKEL_SAVE_DIR")
            .or_else(|| {
                std::env::var_os("XDG_DATA_HOME")
                    .map(|path| PathBuf::from(path).join("waldwinkel").into())
            })
            .or_else(|| {
                std::env::var_os("HOME")
                    .map(|path| PathBuf::from(path).join(".local/share/waldwinkel").into())
            })
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("build/waldwinkel-save"));

        Self(directory.join("garten.json"))
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mainland_update_keeps_the_saved_town_position_and_harvests() {
        let path =
            std::env::temp_dir().join(format!("waldwinkel-mainland-{}.json", std::process::id()));
        fs::write(&path, r#"{"version":4,"day":4,"harvested":8,"checkpoint":{"area":{"District":"Town"},"position":[411.5,2.0]},"town":{"picked":[4,null]}}"#).unwrap();
        let restored = Garden::load(&path).unwrap();
        assert_eq!((restored.day, restored.harvested), (4, 8));
        let checkpoint = restored.checkpoint.unwrap();
        assert_eq!(checkpoint.area, crate::adventure::Area::Island);
        assert_eq!(checkpoint.position, [52.5, 6.]);
        assert!(!restored.town.find_ready(0, 4));
        assert!(restored.town.find_ready(crate::travel::FINDS.len() - 1, 4));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn growth_requires_water_on_each_game_day() {
        for crop in Crop::ALL {
            let mut g = Garden::default();
            assert_eq!(g.work(0, crop).unwrap().0, Action::Till);
            assert_eq!(g.work(0, crop).unwrap().0, Action::Sow);
            g.sleep();
            assert_eq!(g.plots[0].remaining(), Some((crop.days(), false)));
            for _ in 0..crop.days() {
                if g.plots[0].remaining().is_some_and(|(_, wet)| !wet) {
                    assert_eq!(g.work(0, crop).unwrap().0, Action::Water);
                }
                assert!(g.work(0, crop).is_none());
                g.sleep();
            }
            assert_eq!(g.work(0, crop).unwrap().0, Action::Harvest);
            assert_eq!(g.stock(Product::Crop(crop)), 3);
            assert!(g.work(PLOT_COUNT, crop).is_none());
        }
    }

    #[test]
    fn rain_waters_crops_until_the_next_day() {
        let mut g = Garden::default();
        g.sleep();
        g.work(0, Crop::Tomato);
        g.work(0, Crop::Tomato);
        assert_eq!(g.plots[0].remaining(), Some((3, false)));
        g.sleep();
        assert_eq!(Sky::at(g.day), Sky::Rain);
        assert_eq!(g.plots[0].remaining(), Some((3, true)));
        g.sleep();
        assert_eq!(g.plots[0].remaining(), Some((2, false)));
    }

    #[test]
    fn migrating_an_old_save_on_a_rainy_day_waters_the_garden() {
        let path =
            std::env::temp_dir().join(format!("waldwinkel-v2-rain-{}.json", std::process::id()));
        fs::write(
            &path,
            r#"{"version":2,"day":3,"plots":[{"Sown":{"crop":"Turnip","growth":1,"wet":false}}]}"#,
        )
        .unwrap();
        let garden = Garden::load(&path).unwrap();
        fs::remove_file(path).unwrap();
        assert_eq!(garden.plots[0].remaining(), Some((2, true)));
        assert_eq!(garden.farm.coins, 20);
    }

    #[test]
    fn old_save_keeps_its_harvest_and_seeds() {
        let path = std::env::temp_dir().join(format!("waldwinkel-v1-{}.json", std::process::id()));
        fs::write(&path, r#"{"version":1,"plots":["Tilled","Grass","Grass","Grass","Grass","Grass"],"seeds":4,"harvested":9}"#).unwrap();
        let g = Garden::load(&path).unwrap();
        fs::remove_file(path).unwrap();
        assert_eq!(
            (g.plots.len(), g.plots[0], g.seeds, g.harvested, g.day),
            (12, Plot::Tilled, 4, 9, 1)
        );
    }
}
