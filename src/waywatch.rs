use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    adventure::Target,
    craft::{self, Need, Recipe},
    duel::{Card, Enemy},
    furniture::{self, WATCH_LAMP},
    garden::{Crop, Garden},
    homestead::Product,
    processing::Dish,
    regions::Frontier,
    town::Npc,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum Stage {
    #[default]
    Unheard,
    Surveying,
    Surveyed,
    Guarding,
    Complete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Watch {
    Fern,
    Tide,
    Kiln,
    Wind,
    Fen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trial {
    pub watch: Watch,
    pub rank: u8,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct WatchRecord {
    stages: [Stage; Watch::ALL.len()],
    ranks: [u8; Watch::ALL.len()],
    pub gathered: bool,
}

impl WatchRecord {
    pub fn lights(&self) -> usize {
        self.stages
            .iter()
            .filter(|&&stage| stage >= Stage::Complete)
            .count()
    }
}

impl Watch {
    pub const ALL: [Self; 5] = [Self::Fern, Self::Tide, Self::Kiln, Self::Wind, Self::Fen];

    pub fn name(self) -> &'static str {
        [
            "Farnwacht",
            "Gezeitenwarte",
            "Erzofen",
            "Windwarte",
            "Moorlaterne",
        ][self as usize]
    }

    pub fn model(self) -> &'static str {
        [
            "watch_fern",
            "watch_tide",
            "watch_kiln",
            "watch_wind",
            "watch_fen",
        ][self as usize]
    }

    pub fn point(self) -> Vec2 {
        [
            Vec2::new(-64.85, -18.85),
            Vec2::new(-6.90, 51.85),
            Vec2::new(148., -2.25),
            Vec2::new(44.50, 150.50),
            Vec2::new(-155.55, 7.00),
        ][self as usize]
    }

    pub fn trace(self) -> Vec2 {
        [
            Vec2::new(-55.00, -12.60),
            Vec2::new(-15.45, 50.50),
            Vec2::new(158., 1.80),
            Vec2::new(57.35, 145.),
            Vec2::new(-151., 5.50),
        ][self as usize]
    }

    pub fn stage(self, garden: &Garden) -> Stage {
        garden.watches.stages[self as usize]
    }

    pub fn gate(self) -> Option<Frontier> {
        match self {
            Self::Kiln => Some(Frontier::Copper),
            Self::Wind => Some(Frontier::Wind),
            Self::Fen => Some(Frontier::Moor),
            _ => None,
        }
    }

    pub fn accessible(self, garden: &Garden) -> bool {
        self.gate().is_none_or(|gate| gate.unlocked(garden))
    }

    pub fn route(self, garden: &Garden) -> Option<Target> {
        if let Some(gate) = self.gate().filter(|gate| !gate.unlocked(garden)) {
            return Some(Target::Gate(gate));
        }
        Some(if self.stage(garden) == Stage::Surveying {
            Target::WatchTrace(self)
        } else {
            Target::Watch(self)
        })
    }

    pub fn memory(self) -> &'static str {
        match self {
            Self::Fern => {
                "Auf der Rückseite steht Bos Familienname. Ein Händler hat hier vor jedem Sturm die Vorräte gezählt."
            }
            Self::Tide => {
                "Eine eingeritzte Welle kündigt die Flut an. Das blaugrüne Glas war für Fischer auf dem Rückweg."
            }
            Self::Kiln => {
                "Runa erkennt das Zeichen auf den Schienen. Die alten Hüter prüften hier jede neue Niete."
            }
            Self::Wind => {
                "Neles Großmutter hat die Windrichtung im Gewebe festgehalten. Ein Pfeil im Stoff weist zum Hafen."
            }
            Self::Fen => {
                "Edda kennt den letzten Eintrag. Die Hüter brachten das Licht auch in Nächten zurück, in denen die Glocken schwiegen."
            }
        }
    }

    fn tale(self) -> &'static str {
        match self {
            Self::Fern => {
                "Unter dem Dach liegt das Streckenbuch der Waldwacht. Eine Kerbe weist auf den oberen Waldpfad."
            }
            Self::Tide => {
                "Die Glocke am Strand hat einen Sprung. Daneben ist ein Symbol für eine Flutmarke eingeritzt."
            }
            Self::Kiln => {
                "Am kalten Ofen steht ein Wachzeichen. Im alten Erzgleis fehlt das Gegenstück."
            }
            Self::Wind => {
                "Ein Riss im Segel verstellt den Windmesser. Auf einem Hügel steht noch das vermessene Gegenstück."
            }
            Self::Fen => {
                "Auf dem Stein sitzt ein blinder Glasring. Unter einer Weglaterne am Moorpfad liegt sein alter Bauplan."
            }
        }
    }

    fn supplies(self) -> &'static [Need] {
        use Need::Stock;
        match self {
            Self::Fern => &[Stock(Product::Crafted(Recipe::Boards), 2)],
            Self::Tide => &[
                Stock(Product::Crafted(Recipe::TideGlass), 1),
                Stock(Product::Fish, 1),
            ],
            Self::Kiln => &[
                Stock(Product::Crafted(Recipe::Rivets), 2),
                Stock(Product::Prepared(Dish::CarrotSoup), 1),
            ],
            Self::Wind => &[
                Stock(Product::Crafted(Recipe::Canvas), 2),
                Stock(Product::Prepared(Dish::HazelFlour), 1),
            ],
            Self::Fen => &[
                Stock(Product::Crafted(Recipe::Oil), 1),
                Stock(Product::Regional(crate::regions::Material::MarshHerb), 2),
            ],
        }
    }

    pub fn note(self, garden: &Garden) -> String {
        if !self.accessible(garden) {
            return format!(
                "Zugang über die Pforte zur Region {}.",
                self.gate().unwrap().district().name()
            );
        }
        match self.stage(garden) {
            Stage::Unheard => self.tale().into(),
            Stage::Surveying => {
                "Suche das Wegzeichen in der Nähe. Markiere es hier oder im Journal.".into()
            }
            Stage::Surveyed => {
                let station = match self {
                    Self::Fern | Self::Tide | Self::Fen => "an der Hofwerkbank",
                    Self::Kiln => "am Schmiedetisch",
                    Self::Wind => "am Webrahmen",
                };
                format!(
                    "{}\n{} · Fertigung {}",
                    self.memory(),
                    craft::detail(self.supplies(), garden, 1),
                    station
                )
            }
            Stage::Guarding => format!("Ein Hüter antwortet auf das Licht. {}", self.rule()),
            Stage::Complete => self.trial(garden).map_or_else(
                || self.memory().into(),
                |trial| {
                    format!(
                        "Wacht entzündet · Prüfung {}/{}\n{}",
                        trial.rank + 1,
                        Trial::RANKS,
                        self.rule()
                    )
                },
            ),
        }
    }

    pub fn summary(self, garden: &Garden) -> String {
        if !self.accessible(garden) {
            return "Region noch verschlossen · Weg zum Tor".into();
        }
        match self.stage(garden) {
            Stage::Unheard => "Wacht aufsuchen".into(),
            Stage::Surveying => "Wegzeichen suchen".into(),
            Stage::Surveyed => craft::detail(self.supplies(), garden, 1),
            Stage::Guarding => "Ein Kartenduell steht bevor".into(),
            Stage::Complete if self.trial_enemy().is_none() => {
                "Licht brennt · Wacht wieder in Betrieb".into()
            }
            Stage::Complete => format!(
                "Licht brennt · {} / {} Prüfungen",
                garden.watches.ranks[self as usize],
                Trial::RANKS
            ),
        }
    }

    pub fn action(self, garden: &Garden) -> &'static str {
        match self.stage(garden) {
            Stage::Unheard => "Spur aufnehmen",
            Stage::Surveying => "Wegzeichen markieren",
            Stage::Surveyed => "Wacht herrichten",
            Stage::Guarding | Stage::Complete => "Kartenduell beginnen",
        }
    }

    pub fn ready(self, garden: &Garden) -> bool {
        self.accessible(garden)
            && match self.stage(garden) {
                Stage::Surveyed => craft::ready(self.supplies(), garden, 1),
                Stage::Guarding | Stage::Complete => self.trial(garden).is_some(),
                _ => true,
            }
    }

    pub fn begin(self, garden: &mut Garden) -> bool {
        if !self.accessible(garden) || self.stage(garden) != Stage::Unheard {
            return false;
        }
        garden.watches.stages[self as usize] = Stage::Surveying;
        true
    }

    pub fn inspect(self, garden: &mut Garden) -> Option<String> {
        if !self.accessible(garden) || self.stage(garden) != Stage::Surveying {
            return None;
        }
        garden.watches.stages[self as usize] = Stage::Surveyed;
        Some(format!("Wegzeichen gelesen · {}", self.memory()))
    }

    pub fn restore(self, garden: &mut Garden) -> Result<String, String> {
        if self.stage(garden) != Stage::Surveyed || !self.ready(garden) {
            return Err(self.note(garden));
        }
        craft::take(self.supplies(), garden, 1);
        if self.trial_enemy().is_some() {
            garden.watches.stages[self as usize] = Stage::Guarding;
            return Ok("Die Wacht antwortet. Ihr Hüter wartet auf ein Kartenduell.".into());
        }
        garden.watches.stages[self as usize] = Stage::Complete;
        garden.farm.coins = garden.farm.coins.saturating_add(34);
        let seed = if self == Self::Tide {
            Crop::Turnip
        } else {
            Crop::Sunflower
        };
        garden.add_seeds(seed, 3);
        Ok(format!(
            "{} entzündet · +34 Münzen und Saatgut",
            self.name()
        ))
    }

    fn trial_enemy(self) -> Option<Enemy> {
        match self {
            Self::Fern => Some(Enemy::Mosskin),
            Self::Kiln => Some(Enemy::KilnGuard),
            Self::Fen => Some(Enemy::MistKeeper),
            _ => None,
        }
    }

    pub fn rule(self) -> &'static str {
        match self {
            Self::Fern => "Dornen · Am Zugende treffen Dornen für 2, wenn du keinen Schild hast.",
            Self::Kiln => {
                "Kupferhaut · Der Hüter baut in ungeraden Zügen 3 Rüstung auf, außer beim Aufladen."
            }
            Self::Fen => "Nebel · In jedem dritten Zug steht eine Kraft weniger bereit.",
            _ => "",
        }
    }

    pub fn trial(self, g: &Garden) -> Option<Trial> {
        let rank = g.watches.ranks[self as usize];
        (self.accessible(g)
            && self.trial_enemy().is_some()
            && self.stage(g) >= Stage::Guarding
            && rank < Trial::RANKS)
            .then_some(Trial { watch: self, rank })
    }
}

impl Trial {
    pub const RANKS: u8 = 4;

    pub fn level(self) -> u32 {
        let rank = u32::from(self.rank);
        match self.watch {
            Watch::Fern => 1 + rank * 4,
            Watch::Kiln => 5 + rank * 5,
            Watch::Fen => 9 + rank * 6,
            _ => unreachable!("Only guarded watches offer trials"),
        }
    }

    pub fn enemy(self) -> Enemy {
        self.watch.trial_enemy().expect("guarded watch")
    }

    pub fn reward(self, g: &mut Garden) -> Option<String> {
        if self.watch.trial(g) != Some(self) {
            return None;
        }
        g.watches.stages[self.watch as usize] = Stage::Complete;
        g.watches.ranks[self.watch as usize] += 1;
        let xp = if self.rank == 0 { 1 } else { 2 };
        g.rpg.points = g.rpg.points.saturating_add(xp);
        let coins = 18 + self.level() * 2;
        g.farm.coins = g.farm.coins.saturating_add(coins);
        g.add_stock(Product::Spoils(self.enemy().loot()), 1);
        let card = match (self.watch, self.rank) {
            (Watch::Fern, 0) => Some(Card::ForageTea),
            (Watch::Fern, 2) => Some(Card::MossBloom),
            (Watch::Kiln, 0) => Some(Card::CopperArc),
            (Watch::Kiln, 2) => Some(Card::StoneEcho),
            (Watch::Fen, 0) => Some(Card::LanternWard),
            (Watch::Fen, 3) => Some(Card::MoonSlice),
            _ => None,
        }
        .filter(|&card| g.rpg.learn(card));
        let mut result = format!("Wachtprüfung bestanden · +{coins} Münzen · +{xp} Talentpunkte");
        if let Some(card) = card {
            result.push_str(&format!(" · Karte {}", card.name()));
        }
        Some(result)
    }
}

pub fn finale_route(g: &Garden) -> Option<Target> {
    if g.watches.gathered {
        None
    } else if g.watches.lights() == Watch::ALL.len() {
        Some(Target::Person(Npc::Juna))
    } else {
        Watch::ALL
            .into_iter()
            .find(|site| site.stage(g) != Stage::Complete)
            .and_then(|site| site.route(g))
    }
}

pub fn conclude(g: &mut Garden) -> Result<String, String> {
    if g.watches.gathered || g.watches.lights() != Watch::ALL.len() {
        return Err("Alle fünf Lichter sollen wieder leuchten.".into());
    }
    g.watches.gathered = true;
    furniture::learn(g, WATCH_LAMP);
    g.farm.coins = g.farm.coins.saturating_add(94);
    g.rpg.learn(Card::HarborSong);
    Ok("Die Inselkarte ist vollständig · Bauplan Wegwachtlampe · +94 Münzen".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{craft::Bench, workshop::Project};

    #[test]
    fn five_watches_connect_recipes_trials_a_new_station_and_the_finale() {
        let mut g = Garden::default();
        g.wood = 50;
        g.stone = 40;
        g.copper = 30;
        Project::Restore.build(&mut g).unwrap();
        assert!(!Watch::Wind.begin(&mut g));
        g.regions = serde_json::from_value(serde_json::json!({"gates":[2,2,2]})).unwrap();
        for (watch, recipe) in [
            (Watch::Fern, Recipe::Boards),
            (Watch::Tide, Recipe::TideGlass),
            (Watch::Kiln, Recipe::Rivets),
            (Watch::Wind, Recipe::Canvas),
            (Watch::Fen, Recipe::Oil),
        ] {
            assert!(watch.begin(&mut g));
            assert_eq!(watch.route(&g), Some(Target::WatchTrace(watch)));
            assert!(watch.restore(&mut g).is_err());
            assert!(watch.inspect(&mut g).is_some());
            assert!(watch.inspect(&mut g).is_none());
            g.add_stock(Product::Shell, 2);
            g.add_stock(Product::Fish, 2);
            g.add_stock(Product::Prepared(Dish::CarrotSoup), 1);
            g.add_stock(Product::Regional(crate::regions::Material::MeadowFlax), 2);
            g.add_stock(Product::Wool, 1);
            g.add_stock(Product::Prepared(Dish::HazelFlour), 1);
            g.add_stock(Product::Prepared(Dish::SunflowerMeal), 1);
            g.add_stock(Product::Regional(crate::regions::Material::MarshHerb), 3);
            if watch == Watch::Kiln {
                Project::Forge.build(&mut g).unwrap();
            }
            if watch == Watch::Wind {
                Project::Loom.build(&mut g).unwrap();
            }
            recipe.make(&mut g, recipe.sort(), 1).unwrap();
            watch.restore(&mut g).unwrap();
            assert!(watch.restore(&mut g).is_err());
            if let Some(trial) = watch.trial(&g) {
                assert_eq!(trial.rank, 0);
                assert!(trial.reward(&mut g).is_some());
                assert!(trial.reward(&mut g).is_none());
            }
            assert_eq!(watch.stage(&g), Stage::Complete);
        }
        assert_eq!(g.watches.lights(), 5);
        assert_eq!(finale_route(&g), Some(Target::Person(Npc::Juna)));
        conclude(&mut g).unwrap();
        assert!(g.blueprints[WATCH_LAMP]);
        assert_eq!(finale_route(&g), None);
        assert!(conclude(&mut g).is_err());
        let before = g.wood;
        assert!(Recipe::Boards.make(&mut g, Bench::Forge, 3).is_err());
        assert_eq!(g.wood, before);
        Recipe::Boards.make(&mut g, Bench::Work, 3).unwrap();
        assert_eq!(g.stock(Product::Crafted(Recipe::Boards)), 6);
        assert_eq!(g.wood, before - 9);
    }
}
