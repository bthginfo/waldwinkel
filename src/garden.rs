use crate::{
    day::FarmClock,
    duel,
    furniture::{self, CATALOG},
    homestead::{Homestead, Product},
    onboarding::Onboarding,
    regions::RegionProgress,
    session::Checkpoint,
    story::StoryProgress,
    town::Journal,
    weather::Sky,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const PLOT_COUNT: usize = 12;
pub const FERTILIZER_PRICE: u32 = 4;
const FERTILIZER_PACK: u32 = 2;
const SAVE_VERSION: u8 = 15;
pub const MAX_SAVE_SIZE: usize = 4 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quality {
    Normal,
    Fine,
    Premium,
}

impl Quality {
    pub const ALL: [Self; 3] = [Self::Normal, Self::Fine, Self::Premium];

    pub fn name(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Fine => "Fein",
            Self::Premium => "Prächtig",
        }
    }

    pub fn price(self, base: u32) -> u32 {
        match self {
            Self::Normal => base,
            Self::Fine => base + base.div_ceil(2),
            Self::Premium => base * 2,
        }
    }
}

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
    Fertilize,
    Harvest,
}

impl Action {
    pub fn label(self) -> &'static str {
        match self {
            Self::Till => "Beet anlegen",
            Self::Sow => "Aussäen",
            Self::Water => "Gießen",
            Self::Fertilize => "Düngen",
            Self::Harvest => "Ernten",
        }
    }
}

pub const FURNITURE_COUNT: usize = 14;

#[derive(Resource, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Garden {
    version: u8,
    pub plots: Vec<Plot>,
    pub seeds: u32,
    pub harvested: u32,
    pub day: u32,
    pub clock: FarmClock,
    pub fertilizer: u32,
    pub fertilized: [bool; PLOT_COUNT],
    fine_yields: [[u32; 2]; 6],
    prepared: [u32; crate::processing::Dish::ALL.len()],
    materials: [u32; crate::craft::Recipe::ALL.len()],
    pub watches: crate::waywatch::WatchRecord,
    pub ledger: crate::commissions::Ledger,
    harvest_rng: u64,
    pub extra_seeds: [u32; 2],
    pub yields: [u32; 3],
    pub wood: u32,
    pub stone: u32,
    pub copper: u32,
    pub fish: u32,
    pub felled: Vec<usize>,
    tree_regrowth: Vec<(usize, u32)>,
    pub broken: Vec<usize>,
    pub mined: std::collections::BTreeSet<(u32, u16)>,
    pub deepest: u32,
    pub furniture: Vec<u8>,
    pub crafted: Vec<bool>,
    pub blueprints: Vec<bool>,
    pub challenge: usize,
    pub farm: Homestead,
    pub checkpoint: Option<Checkpoint>,
    pub town: Journal,
    pub regions: RegionProgress,
    pub rpg: duel::Progress,
    pub story: StoryProgress,
    pub seasons: crate::seasons::SeasonProgress,
    pub harbor: crate::harbor::Progress,
    pub onboarding: Onboarding,
    pub large_ui: bool,
    pub sound_muted: bool,
    pub workshop: crate::workshop::Progress,
}

impl Default for Garden {
    fn default() -> Self {
        Self {
            version: SAVE_VERSION,
            plots: vec![Plot::Grass; PLOT_COUNT],
            seeds: 6,
            harvested: 0,
            day: 1,
            clock: FarmClock::default(),
            fertilizer: 3,
            fertilized: [false; PLOT_COUNT],
            fine_yields: [[0; 2]; 6],
            prepared: [0; crate::processing::Dish::ALL.len()],
            materials: [0; crate::craft::Recipe::ALL.len()],
            watches: Default::default(),
            ledger: Default::default(),
            harvest_rng: now_ms(),
            extra_seeds: [4, 4],
            yields: [0; 3],
            wood: 0,
            stone: 0,
            copper: 0,
            fish: 0,
            felled: Vec::new(),
            tree_regrowth: Vec::new(),
            broken: Vec::new(),
            mined: Default::default(),
            deepest: 0,
            furniture: vec![0; FURNITURE_COUNT],
            crafted: vec![false; CATALOG.len()],
            blueprints: vec![false; CATALOG.len()],
            challenge: 0,
            farm: Homestead::default(),
            checkpoint: None,
            town: Journal::default(),
            regions: RegionProgress::default(),
            rpg: duel::Progress::default(),
            story: StoryProgress::default(),
            seasons: Default::default(),
            harbor: Default::default(),
            onboarding: Onboarding::Welcome,
            large_ui: false,
            sound_muted: false,
            workshop: Default::default(),
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
            Product::Crop(crop, Quality::Normal) if (crop as usize) < 3 => {
                &mut self.yields[crop as usize]
            }
            Product::Crop(crop, Quality::Normal) => &mut self.farm.yields[crop as usize - 3],
            Product::Crop(crop, quality) => {
                &mut self.fine_yields[crop as usize][quality as usize - 1]
            }
            Product::Prepared(dish) => &mut self.prepared[dish as usize],
            Product::Crafted(recipe) => &mut self.materials[recipe as usize],
            Product::Regional(material) => &mut self.regions.goods[material as usize],
            Product::Spoils(loot) => &mut self.rpg.trophies[loot as usize],
            Product::Seasonal(season) => &mut self.seasons.stock[season as usize],
            Product::Fish => &mut self.fish,
            Product::Herring => &mut self.harbor.herring,
            Product::Mushroom => &mut self.town.goods[0],
            Product::Hazel => &mut self.town.goods[1],
            Product::Shell => &mut self.town.goods[2],
            _ => &mut self.farm.goods[Self::goods_index(item)],
        }
    }

    pub fn stock(&self, item: Product) -> u32 {
        match item {
            Product::Crop(crop, Quality::Normal) if (crop as usize) < 3 => {
                self.yields[crop as usize]
            }
            Product::Crop(crop, Quality::Normal) => self.farm.yields[crop as usize - 3],
            Product::Crop(crop, quality) => self.fine_yields[crop as usize][quality as usize - 1],
            Product::Prepared(dish) => self.prepared[dish as usize],
            Product::Crafted(recipe) => self.materials[recipe as usize],
            Product::Regional(material) => self.regions.goods[material as usize],
            Product::Spoils(loot) => self.rpg.trophies[loot as usize],
            Product::Seasonal(season) => self.seasons.stock[season as usize],
            Product::Fish => self.fish,
            Product::Herring => self.harbor.herring,
            Product::Mushroom => self.town.goods[0],
            Product::Hazel => self.town.goods[1],
            Product::Shell => self.town.goods[2],
            _ => self.farm.goods[Self::goods_index(item)],
        }
    }

    pub fn crop_total(&self, crop: Crop) -> u32 {
        Quality::ALL.into_iter().fold(0_u32, |total, quality| {
            total.saturating_add(self.stock(Product::Crop(crop, quality)))
        })
    }

    pub fn take_crop(&mut self, crop: Crop, quantity: u32) -> bool {
        if self.crop_total(crop) < quantity {
            return false;
        }
        let mut left = quantity;
        for quality in Quality::ALL {
            let item = Product::Crop(crop, quality);
            let amount = self.stock(item).min(left);
            self.take_stock(item, amount);
            left -= amount;
        }
        true
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
        self.sell_many(item, 1).is_some()
    }

    pub fn sell_many(&mut self, item: Product, quantity: u32) -> Option<u32> {
        let price = item.value().checked_mul(quantity)?;
        let balance = self.farm.coins.checked_add(price)?;
        if quantity == 0 || !self.take_stock(item, quantity) {
            return None;
        }
        self.farm.coins = balance;
        self.farm.trades = self.farm.trades.saturating_add(1);
        Some(price)
    }

    pub fn buy_fertilizer(&mut self) -> bool {
        let Some(balance) = self.farm.coins.checked_sub(FERTILIZER_PRICE) else {
            return false;
        };
        let Some(stock) = self.fertilizer.checked_add(FERTILIZER_PACK) else {
            return false;
        };
        if !self.clock.market_open() {
            return false;
        }
        self.fertilizer = stock;
        self.farm.coins = balance;
        self.farm.trades = self.farm.trades.saturating_add(1);
        true
    }

    pub fn fertilizable(&self, index: usize) -> bool {
        self.plots.get(index).is_some_and(|plot| {
            matches!(plot.stage(), Stage::Soil | Stage::Sprout | Stage::Young)
                && !self.fertilized[index]
        })
    }

    pub fn action(&self, index: usize, crop: Crop) -> Option<Action> {
        if self.clock.night() {
            return None;
        }
        let plot = *self.plots.get(index)?;
        match plot.stage() {
            Stage::Grass => Some(Action::Till),
            Stage::Soil if self.seed_count(crop) > 0 => Some(Action::Sow),
            Stage::Ripe => Some(Action::Harvest),
            _ if matches!(plot, Plot::Sown { wet: false, .. }) => Some(Action::Water),
            _ => None,
        }
    }

    pub fn work(&mut self, index: usize, crop: Crop, action: Action) -> Option<String> {
        let available = match action {
            Action::Fertilize => {
                self.fertilizer > 0 && self.fertilizable(index) && !self.clock.night()
            }
            _ => self.action(index, crop) == Some(action),
        };
        if !available {
            return None;
        }
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
            Action::Fertilize => {
                self.fertilizer -= 1;
                self.fertilized[index] = true;
                (plot, "Gedüngt. Die nächste Ernte wird hochwertiger.".into())
            }
            Action::Harvest => {
                let crop = plot.crop();
                let quality = if !self.fertilized[index] {
                    Quality::Normal
                } else if self.harvest_roll() % 4 == 0 {
                    Quality::Premium
                } else {
                    Quality::Fine
                };
                self.fertilized[index] = false;
                self.harvested = self.harvested.saturating_add(3);
                self.ledger.record(crate::commissions::Activity::Harvest, 1);
                let item = Product::Crop(crop, quality);
                self.add_stock(item, 3);
                let mut message = format!("+3 {}", item.label());
                if self.harvest_roll() % 10 < 3 {
                    self.add_seeds(crop, 1);
                    message.push_str(" · +1 Samen");
                }
                (Plot::Tilled, message)
            }
        };
        self.onboarding
            .farm_action(action, new.remaining().is_some_and(|(_, wet)| wet));
        self.plots[index] = new;
        Some(message)
    }

    fn harvest_roll(&mut self) -> u64 {
        self.harvest_rng = self.harvest_rng.wrapping_add(0x9e3779b97f4a7c15);
        let mut value = self.harvest_rng;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        value ^ (value >> 31)
    }

    pub fn wood_per_tree(&self) -> u32 {
        3 + u32::from(crate::workshop::ToolKind::Axe.level(self))
    }

    pub fn stone_per_rock(&self) -> u32 {
        2 + u32::from(crate::workshop::ToolKind::Pickaxe.level(self))
    }

    pub fn chop_tree(&mut self, index: usize) -> bool {
        if self.felled.contains(&index) {
            return false;
        }
        self.felled.push(index);
        self.ledger.record(crate::commissions::Activity::Fell, 1);
        self.tree_regrowth.push((index, self.day.saturating_add(3)));
        self.wood = self.wood.saturating_add(self.wood_per_tree());
        true
    }

    pub fn sleep(&mut self) {
        self.clock = FarmClock::default();
        self.rpg.rest();
        let rain = Sky::at(self.day) == Sky::Rain;
        self.farm.end_day(self.day);
        self.day = self.day.saturating_add(1);
        let day = self.day;
        self.tree_regrowth.retain(|&(tree, ready)| {
            if ready <= day {
                self.felled.retain(|&index| index != tree);
            }
            ready > day
        });
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
        Self::decode(&read_save(path)?)
    }

    pub(crate) fn decode(bytes: &[u8]) -> io::Result<Self> {
        #[derive(Deserialize)]
        struct Header {
            version: u8,
        }
        if bytes.len() > MAX_SAVE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Die Sicherung ist zu groß.",
            ));
        }
        let header: Header = serde_json::from_slice(bytes)?;
        let mut garden: Self = serde_json::from_slice(bytes)?;
        if !matches!(header.version, 1..=SAVE_VERSION) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unbekannte Spielstandversion",
            ));
        }
        garden.plots.resize(PLOT_COUNT, Plot::Grass);
        garden.furniture.resize(FURNITURE_COUNT, 0);
        garden.workshop.house = garden.workshop.house.min(2);
        garden.crafted.resize(CATALOG.len(), false);
        garden.blueprints.resize(CATALOG.len(), false);
        for (index, &built) in garden.crafted.iter().enumerate() {
            garden.blueprints[index] |= built;
        }
        if garden.version < 7 {
            garden.farm.housing = garden.farm.animals.map(|animal| animal.owned);
            garden.tree_regrowth = garden
                .felled
                .iter()
                .map(|&index| (index, garden.day.saturating_add(3)))
                .collect();
            for plan in garden.town.earned_blueprints() {
                garden.blueprints[plan] = true;
            }
        }
        garden.clock.restore();
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
        if garden.version < 9 {
            crate::progression::restore_quest_cards(&mut garden);
        }
        if garden.version < 12 {
            garden.onboarding = Onboarding::Done;
        }
        if garden.version < 13 {
            garden.workshop.restored = true;
            garden.workshop.barns = garden.farm.housing;
            for plan in 0..3 {
                furniture::learn(&mut garden, plan);
            }
        }
        if garden.version < 15 {
            crate::waywatch::restore_bells(&mut garden);
        }
        if garden.onboarding == Onboarding::Water
            && garden
                .plots
                .iter()
                .any(|plot| plot.remaining().is_some_and(|(_, wet)| wet))
        {
            garden.onboarding = Onboarding::Harbor;
        }
        garden.rpg.normalize();
        garden.version = SAVE_VERSION;
        Ok(garden)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let next = serde_json::to_vec(self)?;
        if let Ok(previous) = read_save(path) {
            if previous == next {
                return Ok(());
            }
            if Self::decode(&previous).is_ok() {
                write_save(&Self::backup_path(path), &previous)?;
            }
        }
        write_save(path, &next)
    }

    pub(crate) fn backup_path(path: &Path) -> PathBuf {
        path.with_extension("backup.json")
    }
}

fn read_save(path: &Path) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(MAX_SAVE_SIZE as u64 + 1)
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn write_save(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    let mut file = fs::File::create(&tmp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(tmp, path)
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
    fn prior_blueprints_and_goods_survive_the_seasonal_update() {
        let mut original = Garden::default();
        original.day = 22;
        original.blueprints[furniture::HARBOR_CHEST] = true;
        original.crafted[furniture::HARBOR_CHEST] = true;
        original.add_stock(Product::Shell, 7);
        let mut old_save = serde_json::to_value(&original).unwrap();
        old_save["version"] = 10.into();
        old_save["blueprints"].as_array_mut().unwrap().pop();
        old_save["crafted"].as_array_mut().unwrap().pop();
        old_save.as_object_mut().unwrap().remove("seasons");
        let path =
            std::env::temp_dir().join(format!("waldwinkel-seasons-{}.json", std::process::id()));
        fs::write(&path, serde_json::to_vec(&old_save).unwrap()).unwrap();
        let mut restored = Garden::load(&path).unwrap();
        assert_eq!(restored.version, SAVE_VERSION);
        assert_eq!(restored.day, 22);
        assert_eq!(restored.stock(Product::Shell), 7);
        assert!(restored.blueprints[furniture::HARBOR_CHEST]);
        assert!(restored.crafted[furniture::HARBOR_CHEST]);
        assert!(!restored.blueprints[furniture::SEASONS_SHELF]);
        assert_eq!(restored.seasons.collection(), 0);
        assert!(crate::seasons::SeasonProgress::pick(&mut restored, 9).is_some());
        restored.save(&path).unwrap();
        let restored = Garden::load(&path).unwrap();
        assert_eq!(restored.seasons.collection(), 1);
        assert_eq!(restored.seasons.stock[3], 1);
        fs::remove_file(path).unwrap();
    }

    fn primary(g: &mut Garden, index: usize, crop: Crop) -> Option<Action> {
        let action = g.action(index, crop)?;
        g.work(index, crop, action)?;
        Some(action)
    }

    #[test]
    fn fertilizer_raises_quality_and_seed_returns_vary() {
        let crop = Crop::Carrot;
        let mut g = Garden {
            harvest_rng: 47,
            ..Garden::default()
        };
        g.plots[0] = Plot::Tilled;
        assert!(g.work(0, crop, Action::Fertilize).is_some());
        assert!(g.work(0, crop, Action::Fertilize).is_none());
        assert_eq!(g.fertilizer, 2);
        let mut seeds_back = 0;
        for _ in 0..32 {
            g.plots[0] = Plot::Sown {
                crop,
                growth: crop.days(),
                wet: false,
            };
            let previous = g.seed_count(crop);
            assert!(g.work(0, crop, Action::Harvest).is_some());
            seeds_back += g.seed_count(crop) - previous;
        }
        assert!((1..32).contains(&seeds_back));
        assert_eq!(g.crop_total(crop), 96);
        assert_eq!(g.stock(Product::Crop(crop, Quality::Normal)), 93);
        assert!(!g.fertilized[0]);
    }

    #[test]
    fn selling_a_stack_only_removes_the_selected_quality() {
        let mut g = Garden::default();
        let fine = Product::Crop(Crop::Carrot, Quality::Fine);
        let regular = Product::Crop(Crop::Carrot, Quality::Normal);
        g.add_stock(fine, 3);
        g.add_stock(regular, 4);
        let initial = g.farm.coins;
        assert_eq!(g.sell_many(fine, 4), None);
        assert_eq!(g.sell_many(fine, 0), None);
        let price = 3 * fine.value();
        assert_eq!(g.sell_many(fine, 3), Some(price));
        assert_eq!((g.stock(fine), g.stock(regular)), (0, 4));
        assert_eq!(g.farm.coins, initial + price);
    }

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
            assert_eq!(primary(&mut g, 0, crop).unwrap(), Action::Till);
            assert_eq!(primary(&mut g, 0, crop).unwrap(), Action::Sow);
            g.sleep();
            assert_eq!(g.plots[0].remaining(), Some((crop.days(), false)));
            for _ in 0..crop.days() {
                if g.plots[0].remaining().is_some_and(|(_, wet)| !wet) {
                    assert_eq!(primary(&mut g, 0, crop).unwrap(), Action::Water);
                }
                assert!(primary(&mut g, 0, crop).is_none());
                g.sleep();
            }
            assert_eq!(primary(&mut g, 0, crop).unwrap(), Action::Harvest);
            assert_eq!(g.stock(Product::Crop(crop, Quality::Normal)), 3);
            assert!(primary(&mut g, PLOT_COUNT, crop).is_none());
        }
    }

    #[test]
    fn rain_waters_crops_until_the_next_day() {
        let mut g = Garden::default();
        g.sleep();
        primary(&mut g, 0, Crop::Tomato);
        primary(&mut g, 0, Crop::Tomato);
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

    #[test]
    fn an_old_save_keeps_built_furniture_and_gives_adopted_animals_a_home() {
        let path =
            std::env::temp_dir().join(format!("waldwinkel-housing-{}.json", std::process::id()));
        fs::write(
            &path,
            r#"{"version":6,"crafted":[true,true,true],"furniture":[2,0,0,0,1,0],"farm":{"animals":[{"owned":true,"pending":2},{}]},"town":{"quests":["Done","Offered","Offered"]}}"#,
        ).unwrap();
        let g = Garden::load(&path).unwrap();
        fs::remove_file(path).unwrap();
        assert_eq!(g.crafted.len(), CATALOG.len());
        assert_eq!(&g.furniture[..6], &[2, 0, 0, 0, 1, 0]);
        assert!(g.workshop.restored);
        assert!(g.workshop.barns[0]);
        assert_eq!(g.furniture.len(), 14);
        assert!(g.crafted[2] && g.farm.animals[0].owned);
        assert_eq!(g.farm.housing, [true, false]);
        assert!(g.blueprints[crate::furniture::ARMCHAIR]);
    }
}
