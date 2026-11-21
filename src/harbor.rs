use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    furniture::{self, HARBOR_LANTERN},
    garden::{Crop, Garden},
    homestead::Product,
};

pub const BOARD: Vec2 = Vec2::new(55.50, 23.79);
pub const BOARD_HALF: Vec2 = Vec2::new(0.87, 0.54);
pub const DRIFTWOOD: Vec2 = Vec2::new(54.07, 26.54);
pub const PIER_X: f32 = 58.0;
pub const DECK_HEIGHT: f32 = 0.189;
pub const DECK_START: f32 = 27.22;
const LANDING_END: f32 = 30.72;
const RAIL_END: f32 = 33.61;
const DECK_END: f32 = 36.78;
const FITTING_COPPER: u32 = 2;
const OFFERING: u32 = 3;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Phase {
    #[default]
    Damaged,
    Landing,
    Rails,
    Open,
}

impl Phase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Damaged => "1/3 · Den Anleger reparieren",
            Self::Landing => "2/3 · Den Steg sichern",
            Self::Rails => "3/3 · Die Wiedereröffnung",
            Self::Open => "Auftrag erfüllt · Der Fischersteg ist offen",
        }
    }

    pub fn model(self) -> &'static str {
        match self {
            Self::Damaged => "pier_damaged",
            Self::Landing => "pier_landing",
            Self::Rails => "pier_rails",
            Self::Open => "pier_open",
        }
    }

    pub fn deck_height(self, point: Vec2) -> Option<f32> {
        let end = match self {
            Self::Damaged => return None,
            Self::Landing => LANDING_END,
            Self::Rails => RAIL_END,
            Self::Open => DECK_END,
        };
        let half_width = if self == Self::Open && point.y > 34.65 {
            1.59
        } else {
            0.91
        };
        (point.y > DECK_START && point.y <= end && (point.x - PIER_X).abs() < half_width)
            .then_some(DECK_HEIGHT)
    }

    pub fn instructions(self) -> &'static str {
        match self {
            Self::Damaged => {
                "Bo braucht Treibholz vom Hafenufer und zwei Holz. Fiete kennt den angeschwemmten Stapel unterhalb des Bautischs."
            }
            Self::Landing => {
                "Bring vier Holz, zwei Stein und zwei Kupfer zum Bautisch. Gefertigte Kupferbeschläge kannst du stattdessen verwenden."
            }
            Self::Rails => {
                "Für die Wiedereröffnung kochen die Nachbarn zusammen. Bring drei Stück Gemüse oder Obst aus deinen Beeten mit."
            }
            Self::Open => {
                "Am Anleger und an der Seeseite kannst du angeln. Den Bauplan für die Hafenlaterne findest du an deiner Werkbank."
            }
        }
    }

    pub fn action(self) -> &'static str {
        match self {
            Self::Damaged => "Anleger reparieren",
            Self::Landing => "Steg sichern",
            Self::Rails => "Steg eröffnen",
            Self::Open => "Eröffnet",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Fishing {
    Landing,
    Seaward,
}

impl Fishing {
    pub const ALL: [Self; 2] = [Self::Landing, Self::Seaward];

    pub fn open(self, phase: Phase) -> bool {
        match self {
            Self::Landing => phase != Phase::Damaged,
            Self::Seaward => phase == Phase::Open,
        }
    }

    pub fn point(self) -> Vec2 {
        match self {
            Self::Landing => Vec2::new(58.53, 29.55),
            Self::Seaward => Vec2::new(58.74, 35.95),
        }
    }

    pub fn bobber(self) -> Vec2 {
        match self {
            Self::Landing => Vec2::new(60.15, 29.86),
            Self::Seaward => Vec2::new(60.87, 37.46),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Landing => "Angeln am Anleger",
            Self::Seaward => "Angeln an der Seeseite",
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Progress {
    pub started: bool,
    pub phase: Phase,
    pub driftwood: bool,
    pub fittings: bool,
    pub opening_day: Option<u32>,
    pub herring: u32,
}

impl Progress {
    pub fn salvage_available(&self) -> bool {
        self.started && self.phase == Phase::Damaged && !self.driftwood
    }

    pub fn celebrating(&self, day: u32) -> bool {
        self.phase == Phase::Open && self.opening_day == Some(day)
    }
}

pub fn salvage(garden: &mut Garden) -> Option<String> {
    if !garden.harbor.salvage_available() {
        return None;
    }
    garden.harbor.driftwood = true;
    Some("Treibholz geborgen · Bring es zum Bautisch am Fischersteg.".into())
}

pub fn begin(garden: &mut Garden) -> Result<String, String> {
    if garden.harbor.started {
        return Err("Der Fischersteg steht bereits im Journal.".into());
    }
    garden.harbor.started = true;
    Ok("Auftrag angenommen · Der alte Fischersteg".into())
}

pub fn fitting_supplies(garden: &Garden) -> String {
    format!(
        "{}/{FITTING_COPPER} Kupfer",
        garden.copper.min(FITTING_COPPER)
    )
}

pub fn can_make_fittings(garden: &Garden) -> bool {
    garden.workshop.restored
        && garden.harbor.phase == Phase::Landing
        && !garden.harbor.fittings
        && garden.copper >= FITTING_COPPER
}

pub fn make_fittings(garden: &mut Garden) -> Result<String, String> {
    if !garden.workshop.restored {
        return Err("Setze zuerst die Werkbank vor deiner Hütte instand.".into());
    }
    if !can_make_fittings(garden) {
        return Err(
            "Für die Beschläge brauchst du zwei Kupfer und einen reparierten Anleger.".into(),
        );
    }
    garden.copper -= FITTING_COPPER;
    garden.harbor.fittings = true;
    Ok("Kupferbeschläge gefertigt · Bring sie zum Bautisch.".into())
}

fn crops(garden: &Garden) -> u32 {
    Crop::ALL.into_iter().fold(0, |total, crop| {
        total.saturating_add(garden.crop_total(crop))
    })
}

pub fn supplies(garden: &Garden) -> String {
    match garden.harbor.phase {
        Phase::Damaged => format!(
            "{}/1 Bündel Treibholz · {}/2 Holz",
            u8::from(garden.harbor.driftwood),
            garden.wood.min(2),
        ),
        Phase::Landing => {
            let metal = if garden.harbor.fittings {
                "Beschläge bereit".into()
            } else {
                format!("{}/2 Kupfer für Beschläge", garden.copper.min(FITTING_COPPER))
            };
            format!("{metal} · {}/4 Holz · {}/2 Stein", garden.wood.min(4), garden.stone.min(2))
        }
        Phase::Rails => format!(
            "{}/{OFFERING} Ernte aus deinen Beeten",
            crops(garden).min(OFFERING)
        ),
        Phase::Open => String::new(),
    }
}

pub fn can_build(garden: &Garden) -> bool {
    garden.harbor.started
        && !garden.clock.night()
        && match garden.harbor.phase {
            Phase::Damaged => garden.harbor.driftwood && garden.wood >= 2,
            Phase::Landing => (garden.harbor.fittings || garden.copper >= FITTING_COPPER)
                && garden.wood >= 4 && garden.stone >= 2,
            Phase::Rails => crops(garden) >= OFFERING,
            Phase::Open => false,
        }
}

pub fn build(garden: &mut Garden) -> Result<String, String> {
    if garden.clock.night() {
        return Err("Morgen früh kannst du am Steg weiterbauen.".into());
    }
    if !can_build(garden) {
        return Err(supplies(garden));
    }
    let (next, coins, message) = match garden.harbor.phase {
        Phase::Damaged => {
            garden.wood -= 2;
            (
                Phase::Landing,
                6,
                "Der Anleger ist repariert. Du kannst hier jetzt angeln.",
            )
        }
        Phase::Landing => {
            garden.wood -= 4;
            garden.stone -= 2;
            if !garden.harbor.fittings {
                garden.copper -= FITTING_COPPER;
            }
            garden.harbor.fittings = false;
            (
                Phase::Rails,
                12,
                "Der Steg ist gesichert. Bo bereitet die Wiedereröffnung vor.",
            )
        }
        Phase::Rails => {
            let mut left = OFFERING;
            for crop in Crop::ALL {
                let amount = garden.crop_total(crop).min(left);
                garden.take_crop(crop, amount);
                left -= amount;
            }
            garden.harbor.opening_day = Some(garden.day);
            furniture::learn(garden, HARBOR_LANTERN);
            (
                Phase::Open,
                18,
                "Der Steg ist eröffnet! Du hast den Bauplan für die Hafenlaterne.",
            )
        }
        Phase::Open => return Err("Der Fischersteg ist bereits eröffnet.".into()),
    };
    garden.harbor.phase = next;
    garden.farm.coins = garden.farm.coins.saturating_add(coins);
    Ok(format!("{message} +{coins} Münzen"))
}

pub fn catch(garden: &mut Garden, spot: Fishing) -> &'static str {
    let (item, message) = match spot {
        Fishing::Landing => (Product::Fish, "+1 Fisch"),
        Fishing::Seaward => (Product::Herring, "+1 Hafenhering"),
    };
    garden.add_stock(item, 1);
    garden.ledger.record(crate::commissions::Activity::Fish, 1);
    if garden.fish_supper.record_catch() {
        match spot {
            Fishing::Landing => "+1 Fisch · Dein erster Fang! Bo besucht dich nach dem Schlafen.",
            Fishing::Seaward => "+1 Hafenhering · Dein erster Fang! Bo besucht dich nach dem Schlafen.",
        }
    } else { message }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_second_stage_accepts_raw_copper_at_the_building_table() {
        let mut garden = Garden::default();
        garden.harbor.started = true;
        garden.harbor.phase = Phase::Landing;
        garden.wood = 4;
        garden.stone = 2;
        garden.copper = 1;
        assert!(build(&mut garden).is_err());
        assert_eq!((garden.wood, garden.stone, garden.copper), (4, 2, 1));
        garden.copper = 2;
        assert!(build(&mut garden).is_ok());
        assert_eq!(garden.harbor.phase, Phase::Rails);
        assert_eq!((garden.wood, garden.stone, garden.copper), (0, 0, 0));
    }
    use crate::{Adventure, adventure::Target, garden::Quality};

    #[test]
    fn fresh_day_one_can_repair_a_landing_and_land_a_fish() {
        let mut garden = Garden::default();
        let manor = Adventure::default();
        assert!(manor.walkable(DRIFTWOOD, &garden));
        assert!(!manor.walkable(Fishing::Landing.point(), &garden));
        assert!(begin(&mut garden).is_ok());
        assert!(salvage(&mut garden).is_some());
        assert!(salvage(&mut garden).is_none());
        assert!(build(&mut garden).is_err());
        assert!(garden.chop_tree(0));
        assert!(build(&mut garden).is_ok());
        assert_eq!((garden.day, garden.wood), (1, 1));
        assert!(Fishing::Landing.open(garden.harbor.phase));
        assert!(!Fishing::Seaward.open(garden.harbor.phase));
        assert!(manor.walkable(Fishing::Landing.point(), &garden));
        assert_eq!(
            manor.position(Target::HarborFishing(Fishing::Landing), &garden),
            Fishing::Landing.point()
        );
        catch(&mut garden, Fishing::Landing);
        assert_eq!(garden.stock(Product::Fish), 1);
    }

    #[test]
    fn reopening_spends_ingredients_once_and_saves_the_lantern_blueprint() {
        let mut garden = Garden::default();
        begin(&mut garden).unwrap();
        salvage(&mut garden).unwrap();
        garden.wood = 6;
        garden.copper = 2;
        garden.stone = 2;
        build(&mut garden).unwrap();
        assert!(make_fittings(&mut garden).is_err());
        garden.workshop.restored = true;
        make_fittings(&mut garden).unwrap();
        assert!(make_fittings(&mut garden).is_err());
        build(&mut garden).unwrap();
        garden.add_stock(Product::Crop(Crop::Carrot, Quality::Fine), 2);
        garden.add_stock(Product::Crop(Crop::Turnip, Quality::Normal), 1);
        build(&mut garden).unwrap();
        assert_eq!(crops(&garden), 0);
        assert!(garden.harbor.celebrating(garden.day));
        assert!(garden.blueprints[HARBOR_LANTERN]);
        assert!(Adventure::default().walkable(Fishing::Seaward.point(), &garden));
        let coins = garden.farm.coins;
        assert!(build(&mut garden).is_err());
        assert_eq!(garden.farm.coins, coins);
        catch(&mut garden, Fishing::Seaward);
        let saved = serde_json::to_vec(&garden).unwrap();
        let restored: Garden = serde_json::from_slice(&saved).unwrap();
        assert_eq!(restored.harbor.phase, Phase::Open);
        assert_eq!(restored.stock(Product::Herring), 1);
    }
}
