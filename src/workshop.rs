use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    furniture,
    garden::Garden,
    homestead::{Animal, Product},
};

pub const BENCH: Vec2 = Vec2::new(-0.45, 1.34);
pub const FORGE: Vec2 = Vec2::new(7.05, -0.75);
pub const LOOM: Vec2 = Vec2::new(10.85, -0.65);
pub const ANNEXES: [(Vec2, Vec2); 2] = [
    (Vec2::new(0.51, -2.65), Vec2::new(1.21, 1.01)),
    (Vec2::new(-5.86, -2.96), Vec2::new(1.29, 1.15)),
];
pub const SLOT_COUNTS: [usize; 3] = [6, 10, 14];
pub const ROOMS: [&str; 2] = ["Gartenstube", "Lesekammer"];
pub const ROOM_PRICES: [u32; 2] = [32, 58];

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Progress {
    pub restored: bool,
    pub forge: bool,
    pub loom: bool,
    pub house: u8,
    pub rooms: [bool; 2],
    #[serde(deserialize_with = "crate::expansion::read_array")]
    pub barns: [bool; Animal::ALL.len()],
    tools: [u8; 4],
}

impl Progress {
    pub fn slots(&self) -> usize {
        SLOT_COUNTS[self.house.min(2) as usize]
    }

    pub fn annex_at(&self, point: Vec2) -> bool {
        ANNEXES
            .into_iter()
            .take(self.house as usize)
            .any(|(center, half)| (point - center).abs().cmple(half).all())
    }

    pub fn inside(&self, point: Vec2) -> bool {
        if point.y <= -2.65 || point.y >= 2.93 {
            return false;
        }
        if point.x.abs() < 3.35 {
            return true;
        }
        let room = if point.x > 0. { 1 } else { 2 };
        self.house >= room
            && point.x.abs() <= 7.16
            && (point.x.abs() > 3.82 || (-0.35..1.58).contains(&point.y))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ToolKind {
    Axe,
    Pickaxe,
    Can,
    Hoe,
}

impl ToolKind {
    pub const ALL: [Self; 4] = [Self::Axe, Self::Pickaxe, Self::Can, Self::Hoe];
    pub const MAX_LEVEL: u8 = 3;

    pub fn grade(level: u8) -> &'static str {
        ["Einfach", "Kupfer", "Silber", "Sternstahl"][level.min(Self::MAX_LEVEL) as usize]
    }

    pub(crate) fn for_action(tool: crate::world::Tool) -> Option<Self> {
        match tool {
            crate::world::Tool::Axe => Some(Self::Axe),
            crate::world::Tool::Pickaxe => Some(Self::Pickaxe),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        ["Axt", "Spitzhacke", "Gießkanne", "Hacke"][self as usize]
    }

    pub fn icon(self, garden: &Garden) -> String {
        let kind = ["axe", "pickaxe", "can", "hoe"][self as usize];
        format!("tool_{kind}_{}", self.level(garden))
    }

    pub fn effect(self) -> &'static str {
        match self {
            Self::Axe => "Silber fällt Eisenholz · Sternstahl fällt Riffholz und spart Schläge",
            Self::Pickaxe => "Silber löst Sternenerz und Perlmutt · Sternstahl löst Bernstein und Himmelsquarz",
            Self::Can => "Kupfer gießt 2 Beete · Silber 3 · Sternstahl bis zu 6 in zwei Reihen",
            Self::Hoe => "Kupfer lockert 2 Beete · Silber 3 · Sternstahl bis zu 6 in zwei Reihen",
        }
    }

    pub fn level(self, garden: &Garden) -> u8 {
        let level = garden.workshop.tools[self as usize].min(Self::MAX_LEVEL);
        if self == Self::Pickaxe {
            level.max(u8::from(garden.farm.pickaxe))
        } else {
            level
        }
    }

    pub fn tier(self, garden: &Garden) -> &'static str {
        Self::grade(self.level(garden))
    }

    pub(crate) fn ingredients(self, tier: u8) -> Vec<crate::craft::Need> {
        use crate::{craft::Need, regions::Material};
        if tier < 3 {
            return Self::cost(tier).ingredients().to_vec();
        }
        let (material, amount) = match self {
            Self::Axe => (Material::Ironwood, 5),
            Self::Pickaxe => (Material::GorgeQuartz, 6),
            Self::Can => (Material::MotherPearl, 4),
            Self::Hoe => (Material::GoldResin, 4),
        };
        vec![
            Need::Stock(Product::Regional(Material::StarOre), 8),
            Need::Stock(Product::Regional(material), amount),
            Need::Stock(Product::Silver, 4),
            Need::Coins(68),
        ]
    }

    fn cost(tier: u8) -> Cost {
        if tier == 1 {
            Cost {
                wood: 3,
                copper: 3,
                coins: 8,
                ..Default::default()
            }
        } else {
            Cost {
                wood: 4,
                silver: 3,
                coins: 15,
                ..Default::default()
            }
        }
    }

    pub fn requirements(self, garden: &Garden, at_forge: bool) -> String {
        let level = self.level(garden);
        match level {
            Self::MAX_LEVEL => format!("Sternstahlstufe · {}", self.effect()),
            2 if !crate::expansion::starsteel(garden) => "Sternstahlvorlage · Hilf Sanna in den Himmelstufen".into(),
            1 | 2 if !at_forge => format!("{} am Schmiedetisch", Self::grade(level + 1)),
            _ => crate::craft::detail(&self.ingredients(level + 1), garden, 1),
        }
    }

    pub fn can_upgrade(self, garden: &Garden, at_forge: bool) -> bool {
        let level = self.level(garden);
        garden.workshop.restored
            && level < Self::MAX_LEVEL
            && (level == 0 || at_forge && garden.workshop.forge)
            && (level < 2 || crate::expansion::starsteel(garden))
            && crate::craft::ready(&self.ingredients(level + 1), garden, 1)
    }

    pub fn upgrade(self, garden: &mut Garden, at_forge: bool) -> Result<String, String> {
        if !self.can_upgrade(garden, at_forge) {
            return Err(self.requirements(garden, at_forge));
        }
        let tier = self.level(garden) + 1;
        crate::craft::take(&self.ingredients(tier), garden, 1);
        garden.workshop.tools[self as usize] = tier;
        Ok(format!(
            "{} verbessert · {} · {}",
            self.name(),
            self.tier(garden),
            self.effect()
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Project {
    Restore,
    Forge,
    Loom,
    Field,
    Room(usize),
}

impl Project {
    pub fn available(garden: &Garden) -> impl Iterator<Item = Self> + '_ {
        let state = &garden.workshop;
        (!state.restored)
            .then_some(Self::Restore)
            .into_iter()
            .chain((state.restored && !state.forge).then_some(Self::Forge))
            .chain(
                (state.restored && !state.loom && crate::craft::Recipe::Canvas.learned(garden))
                    .then_some(Self::Loom),
            )
            .chain((state.restored && state.house < 2).then_some(Self::Room(state.house as usize)))
            .chain((state.restored && !garden.expanded_field).then_some(Self::Field))
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Restore => "Werkbank instand setzen",
            Self::Forge => "Schmiedetisch bauen",
            Self::Loom => "Webrahmen bauen",
            Self::Field => "Garten erweitern",
            Self::Room(index) => ROOMS[index],
        }
    }

    fn cost(self) -> Cost {
        match self {
            Self::Restore => Cost {
                wood: 2,
                stone: 1,
                ..Default::default()
            },
            Self::Forge => Cost {
                wood: 5,
                stone: 3,
                copper: 4,
                ..Default::default()
            },
            Self::Loom => Cost {
                wood: 6,
                stone: 2,
                copper: 2,
                ..Default::default()
            },
            Self::Field => Cost { wood: 8, stone: 4, ..Default::default() },
            Self::Room(0) => Cost {
                wood: 12,
                stone: 6,
                copper: 3,
                ..Default::default()
            },
            Self::Room(_) => Cost {
                wood: 16,
                stone: 9,
                silver: 2,
                ..Default::default()
            },
        }
    }

    pub(crate) fn ingredients(self) -> [crate::craft::Need; 5] {
        self.cost().ingredients()
    }

    pub(crate) fn built(self, garden: &Garden) -> bool {
        match self {
            Self::Restore => garden.workshop.restored,
            Self::Forge => garden.workshop.forge,
            Self::Loom => garden.workshop.loom,
            Self::Field => garden.expanded_field,
            Self::Room(index) => usize::from(garden.workshop.house) > index,
        }
    }

    pub fn detail(self, garden: &Garden) -> String {
        if self == Self::Field {
            let requirement = if garden.harbor.phase == crate::harbor::Phase::Open {
                ""
            } else {
                " · Erst den Fischersteg wiedereröffnen"
            };
            return format!("6 neue Beete · {}{requirement}", self.cost().detail(garden));
        }
        if let Self::Room(i) = self
            && !garden.workshop.rooms[i]
        {
            return "Bauplan am Bauplanstand".into();
        }
        self.cost().detail(garden)
    }

    pub fn can_build(self, garden: &Garden) -> bool {
        Self::available(garden).any(|available| available == self)
            && self.cost().ready(garden)
            && match self {
                Self::Room(i) => garden.workshop.rooms[i],
                Self::Field => garden.harbor.phase == crate::harbor::Phase::Open,
                _ => true,
            }
    }

    pub fn build(self, garden: &mut Garden) -> Result<String, String> {
        if !self.can_build(garden) {
            return Err(self.detail(garden));
        }
        self.cost().take(garden);
        Ok(match self {
            Self::Restore => {
                garden.workshop.restored = true;
                for plan in 0..3 {
                    furniture::learn(garden, plan);
                }
                "Werkbank repariert · Drei Möbelbaupläne gelernt".into()
            }
            Self::Forge => {
                garden.workshop.forge = true;
                "Schmiedetisch gebaut.".into()
            }
            Self::Loom => {
                garden.workshop.loom = true;
                "Webrahmen gebaut.".into()
            }
            Self::Field => {
                garden.expanded_field = true;
                "Die südliche Wiese ist erschlossen · Sechs neue Beete".into()
            }
            Self::Room(_) => {
                garden.workshop.house += 1;
                format!(
                    "{} gebaut · Mehr Platz für deine Möbel",
                    self.name()
                )
            }
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BuildingPlan {
    Barn(Animal),
    Room(usize),
}

impl BuildingPlan {
    pub const ALL: [Self; 4] = [
        Self::Barn(Animal::Hen),
        Self::Barn(Animal::Sheep),
        Self::Room(0),
        Self::Room(1),
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Barn(animal) => animal.housing_name(),
            Self::Room(i) => ROOMS[i],
        }
    }

    fn price(self) -> Option<u32> {
        Some(match self {
            Self::Barn(Animal::Hen) => 30,
            Self::Barn(Animal::Sheep) => 52,
            Self::Barn(_) => return None,
            Self::Room(i) => ROOM_PRICES[i],
        })
    }

    pub fn known(self, garden: &Garden) -> bool {
        match self {
            Self::Barn(animal) => garden.workshop.barns[animal as usize],
            Self::Room(i) => garden.workshop.rooms[i],
        }
    }

    pub fn detail(self, garden: &Garden) -> String {
        if self.known(garden) {
            "Bauplan gelernt".into()
        } else {
            let Some(price) = self.price() else {
                return match self {
                    Self::Barn(kind) => kind.housing_hint(garden).0,
                    Self::Room(_) => unreachable!(),
                };
            };
            let requirement = match self {
                Self::Barn(Animal::Hen) if !garden.workshop.restored => " · Erst Werkbank reparieren",
                Self::Barn(Animal::Sheep) if !garden.farm.housing[Animal::Hen as usize] => " · Erst Hühnerstall fertigstellen",
                _ => "",
            };
            format!("Bauplan · {price} Münzen{requirement}")
        }
    }

    pub fn can_buy(self, garden: &Garden) -> bool {
        !self.known(garden) && garden.clock.market_open() && self.price().is_some_and(|price| garden.farm.coins >= price)
            && match self {
                Self::Barn(Animal::Hen) => garden.workshop.restored,
                Self::Barn(Animal::Sheep) => garden.farm.housing[Animal::Hen as usize],
                _ => true,
            }
    }

    pub fn buy(self, garden: &mut Garden) -> Result<String, String> {
        if !self.can_buy(garden) {
            return Err(self.detail(garden));
        }
        garden.farm.coins -= self.price().expect("priced building plan");
        match self {
            Self::Barn(animal) => garden.workshop.barns[animal as usize] = true,
            Self::Room(i) => garden.workshop.rooms[i] = true,
        }
        Ok(format!("Bauplan gelernt · {}", self.name()))
    }
}

#[derive(Default)]
struct Cost {
    wood: u32,
    stone: u32,
    copper: u32,
    silver: u32,
    coins: u32,
}

impl Cost {
    fn ingredients(&self) -> [crate::craft::Need; 5] {
        use crate::craft::Need::*;
        [
            Wood(self.wood),
            Stone(self.stone),
            Copper(self.copper),
            Stock(Product::Silver, self.silver),
            Coins(self.coins),
        ]
    }
    fn materials(&self, g: &Garden) -> [(u32, u32, &'static str); 5] {
        [
            (g.wood, self.wood, "Holz"),
            (g.stone, self.stone, "Stein"),
            (g.copper, self.copper, "Kupfer"),
            (g.stock(Product::Silver), self.silver, "Silber"),
            (g.farm.coins, self.coins, "Münzen"),
        ]
    }

    fn detail(&self, g: &Garden) -> String {
        self.materials(g)
            .into_iter()
            .filter(|&(_, count, _)| count > 0)
            .map(|(stock, count, name)| format!("{}/{count} {name}", stock.min(count)))
            .collect::<Vec<_>>()
            .join(" · ")
    }

    fn ready(&self, g: &Garden) -> bool {
        self.materials(g)
            .into_iter()
            .all(|(stock, count, _)| stock >= count)
    }

    fn take(&self, g: &mut Garden) {
        g.wood -= self.wood;
        g.stone -= self.stone;
        g.copper -= self.copper;
        g.farm.coins -= self.coins;
        g.take_stock(Product::Silver, self.silver);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restoring_the_bench_unlocks_the_first_recipes_once() {
        let mut garden = Garden::default();
        assert!(garden.blueprints.iter().all(|&known| !known));
        assert!(Project::Restore.build(&mut garden).is_err());
        garden.wood = 2;
        garden.stone = 1;
        assert!(Project::Restore.build(&mut garden).is_ok());
        assert_eq!(garden.blueprints.iter().filter(|&&known| known).count(), 3);
        assert!(Project::Restore.build(&mut garden).is_err());
        assert_eq!((garden.wood, garden.stone), (0, 0));
    }

    #[test]
    fn rooms_require_separate_plans_and_open_a_walkable_connection() {
        let mut garden = Garden::default();
        garden.workshop.restored = true;
        garden.farm.coins = 100;
        garden.wood = 28;
        garden.stone = 15;
        garden.copper = 3;
        garden.add_stock(Product::Silver, 2);
        assert!(Project::Room(0).build(&mut garden).is_err());
        BuildingPlan::Room(1).buy(&mut garden).unwrap();
        assert!(BuildingPlan::Room(1).buy(&mut garden).is_err());
        assert!(Project::Room(1).build(&mut garden).is_err());
        BuildingPlan::Room(0).buy(&mut garden).unwrap();
        for room in 0..2 {
            Project::Room(room).build(&mut garden).unwrap();
            let direction = if room == 0 { 1. } else { -1. };
            for i in 33..=70 {
                assert!(
                    garden
                        .workshop
                        .inside(Vec2::new(direction * i as f32 / 10., 0.6))
                );
            }
            assert!(!garden.workshop.inside(Vec2::new(direction * 3.56, -1.)));
            assert_eq!(garden.workshop.slots(), SLOT_COUNTS[room + 1]);
        }
        assert_eq!(garden.farm.coins, 10);
    }

    #[test]
    fn silver_tools_require_the_forge_and_increase_resource_yield() {
        let mut garden = Garden::default();
        garden.workshop.restored = true;
        garden.wood = 12;
        garden.stone = 3;
        garden.copper = 7;
        garden.farm.coins = 23;
        garden.add_stock(Product::Silver, 3);
        ToolKind::Axe.upgrade(&mut garden, false).unwrap();
        assert!(ToolKind::Axe.upgrade(&mut garden, true).is_err());
        Project::Forge.build(&mut garden).unwrap();
        assert!(ToolKind::Axe.upgrade(&mut garden, false).is_err());
        ToolKind::Axe.upgrade(&mut garden, true).unwrap();
        assert_eq!(garden.wood, 0);
        assert!(garden.chop_tree(0));
        assert_eq!(garden.wood, 5);
    }
}
