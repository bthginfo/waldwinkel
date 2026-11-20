use crate::{
    Controls, GardenFocus, Notice,
    duel::{Battle, CAMP_INTERVAL, Chapter, Enemy, LAST_LEVEL},
    farm_ui::{FarmMenu, FarmPage},
    forest,
    furniture::CATALOG,
    garden::{Crop, Garden, SavePath, Stage},
    harbor::{self, Fishing, Phase},
    harvest_fx::Break,
    homestead::{Animal, ORCHARD, Ore, Product, Village},
    onboarding::Onboarding,
    regions::{Bell, Frontier},
    seasons::{self, MEETING_POINT, SITES, SeasonProgress},
    session::Checkpoint,
    story::{Guide, GuideTab, Resonator},
    town::{Dialog, Npc},
    travel::{self, District, FINDS},
    waywatch::{Stage as WatchStage, Watch},
    weather::Season,
    workshop,
    world::{self, Gardener, IslandCamera, Reach, Tool},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const HOME: Vec2 = Vec2::new(220., 0.);
const MINE: Vec2 = Vec2::new(400., 0.);
pub const DUSK: Vec2 = Vec2::new(560., 0.);
pub const DOOR: Vec2 = Vec2::new(-2.08, 1.32);
pub const PORTAL: Vec2 = Vec2::new(9., -8.90);
pub const PIER: Vec2 = Vec2::new(5.8, 17.25);
pub const SALE_CHEST: Vec2 = Vec2::new(0.66, 3.58);
const KITCHEN: Vec2 = Vec2::new(9.12, 1.15);
const PATTERN_STALL: Vec2 = Vec2::new(18.36, 5.49);
const DUSK_GATE: Vec2 = Vec2::new(-111., -35.);
const DUSK_ARRIVAL: Vec2 = Vec2::new(-111., -33.54);
const BINDER: Vec2 = Vec2::new(221.02, -2.23);
const MARKER: &str = "selection";
const TILE: f32 = 2.4;
const GRID: i16 = 9;
const START: u16 = 40;
const STUMP_RADIUS: f32 = 0.42;
const TREE_RADIUS: f32 = 0.46;
const ROCK_RADIUS: f32 = 0.78;
const ORE_RADIUS: f32 = 0.71;
const BED_HALF_SIZE: Vec2 = Vec2::new(0.78, 1.04);
const BENCH_HALF_SIZE: Vec2 = Vec2::new(0.71, 0.56);
const TABLE_HALF_SIZE: Vec2 = Vec2::new(0.89, 0.53);
const NPC_RADIUS: f32 = 0.43;
const TREES: [Vec2; 12] = [
    Vec2::new(-16.88, 6.9),
    Vec2::new(-16.60, 10.18),
    Vec2::new(-15., 3.),
    Vec2::new(-10., -2.),
    Vec2::new(-13., -6.),
    Vec2::new(-5., -8.),
    Vec2::new(2., -8.),
    Vec2::new(15., -7.),
    Vec2::new(12.38, 6.41),
    Vec2::new(15.2, 11.4),
    Vec2::new(-5.8, 5.05),
    Vec2::new(8.24, -2.31),
];
const ROCKS: [Vec2; 8] = [
    Vec2::new(-11., 4.),
    Vec2::new(-12., -5.),
    Vec2::new(-17., -1.),
    Vec2::new(10., -6.),
    Vec2::new(12., 12.),
    Vec2::new(10., 9.),
    Vec2::new(-4., 10.),
    Vec2::new(-3.23, 6.20),
];
const SLOTS: [Vec2; 14] = [
    Vec2::new(1.59, 0.77),
    Vec2::new(2.45, -1.61),
    Vec2::new(-0.02, -1.93),
    Vec2::new(-1.66, 1.49),
    Vec2::new(1.85, 1.98),
    Vec2::new(-2.62, 0.38),
    Vec2::new(4.90, -1.70),
    Vec2::new(6.46, -1.55),
    Vec2::new(6.45, 0.29),
    Vec2::new(5.30, 1.60),
    Vec2::new(-4.95, -1.62),
    Vec2::new(-6.40, -1.55),
    Vec2::new(-6.45, 0.31),
    Vec2::new(-5.35, 1.60),
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum Area {
    #[default]
    Island,
    Home,
    Mine(u32),
    District(District),
    Expedition(u32),
}

impl Area {
    pub fn origin(self) -> Vec2 {
        match self {
            Self::Island => Vec2::ZERO,
            Self::Home => HOME,
            Self::Mine(_) => MINE,
            Self::District(district) => district.origin(),
            Self::Expedition(_) => DUSK,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Island => "Waldwinkel",
            Self::Home => "Holzhütte",
            Self::Mine(_) => "Mine",
            Self::District(district) => district.name(),
            Self::Expedition(_) => "Dämmergänge",
        }
    }

    pub fn outdoors(self) -> bool {
        matches!(self, Self::Island | Self::District(_))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Door,
    Portal,
    Pier,
    HarborBoard,
    HarborDriftwood,
    HarborFishing(Fishing),
    SaleChest,
    Kitchen,
    PatternStall,
    FarmAnimal(Animal),
    Gate(Frontier),
    Bell(Bell),
    Tree(usize),
    Rock(usize),
    MineRock(u16),
    Down,
    Up,
    Bed,
    Exit,
    Slot(usize),
    Bench,
    Forge,
    Loom,
    Watch(Watch),
    WatchTrace(Watch),
    Village(Village),
    Travel(District),
    Find(usize),
    SeasonFind(usize),
    SeasonMeeting,
    Person(Npc),
    Binder,
    DuskGate,
    DuskExit,
    DuskDown,
    DuskFoe,
    DuskCamp,
    Resonator(Resonator),
}

impl Target {
    fn restriction(self, garden: &Garden) -> Option<&'static str> {
        match self {
            Self::PatternStall if !garden.clock.market_open() => {
                Some("Der Bauplanstand öffnet morgens um 06:00.")
            }
            Self::DuskGate if !dusk_open(garden) => {
                Some("Eddas Auftrag „Das alte Glockengewölbe“ öffnet den Zugang.")
            }
            Self::Village(Village::Animal(kind)) if garden.farm.animals[kind as usize].owned => {
                Some("Dein Tier wartet in seinem Stall auf deinem Hof.")
            }
            Self::Village(village) if village.closed(garden) => {
                Some("Der Stand öffnet morgens um 06:00.")
            }
            Self::Tree(_)
            | Self::Rock(_)
            | Self::MineRock(_)
            | Self::Portal
            | Self::Down
            | Self::Pier
            | Self::HarborFishing(_)
                if garden.clock.night() =>
            {
                Some("Für heute ist Feierabend. Schlaf in deiner Hütte.")
            }
            _ => None,
        }
    }

    fn tool(self) -> Tool {
        match self {
            Self::Tree(_) => Tool::Axe,
            Self::Rock(_) | Self::MineRock(_) => Tool::Pickaxe,
            Self::Pier | Self::HarborFishing(_) => Tool::Rod,
            _ => Tool::Hoe,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Door | Self::Exit => "Holzhütte",
            Self::Portal => "Minenstollen",
            Self::Pier => "Angelsteg",
            Self::HarborBoard => "Alter Fischersteg",
            Self::HarborDriftwood => "Treibholzbündel",
            Self::HarborFishing(spot) => spot.name(),
            Self::SaleChest => "Verkaufstruhe",
            Self::Kitchen => "Hofküche",
            Self::PatternStall => "Bauplanstand",
            Self::FarmAnimal(kind) => kind.housing_name(),
            Self::Gate(gate) => gate.district().name(),
            Self::Bell(bell) => bell.name(),
            Self::Tree(i) => forest::node(i).map_or("Kiefer", forest::Node::name),
            Self::Rock(i) => forest::node(i).map_or("Steinblock", forest::Node::name),
            Self::MineRock(_) => "Kupferader",
            Self::Down | Self::Up => "Minenleiter",
            Self::Bed => "Bett",
            Self::Slot(_) => "Möbelplatz",
            Self::Bench => "Hofwerkbank",
            Self::Forge => "Schmiedetisch",
            Self::Loom => "Webrahmen",
            Self::Watch(watch) => watch.name(),
            Self::WatchTrace(_) => "Wegzeichen",
            Self::Village(target) => target.name(),
            Self::Travel(district) => district.name(),
            Self::Find(i) => FINDS[i].name(),
            Self::SeasonFind(i) => SITES[i].season.find_name(),
            Self::SeasonMeeting => "Dorftisch",
            Self::Person(person) => person.name(),
            Self::Binder => "Kartenbinder",
            Self::DuskGate | Self::DuskExit => "Dämmergänge",
            Self::DuskDown => "Abstieg",
            Self::DuskFoe => "Begegnung",
            Self::DuskCamp => "Glockenrast",
            Self::Resonator(site) => site.name(),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Door => "Eintreten",
            Self::Exit => "Hinausgehen",
            Self::Portal => "Mine betreten",
            Self::Pier | Self::HarborFishing(_) => "Auswerfen",
            Self::HarborBoard => "Bautisch ansehen",
            Self::HarborDriftwood => "Treibholz bergen",
            Self::SaleChest => "Truhe öffnen",
            Self::Kitchen => "Küche öffnen",
            Self::PatternStall => "Baupläne ansehen",
            Self::FarmAnimal(_) => "Versorgen",
            Self::Gate(_) => "Weg öffnen",
            Self::Bell(_) => "Wegglocke",
            Self::Tree(_) => "Baum fällen",
            Self::Rock(_) => "Stein abbauen",
            Self::MineRock(_) => "Kupfer abbauen",
            Self::Down => "Tiefer steigen",
            Self::Up => "Aufsteigen",
            Self::Bed => "Schlafen",
            Self::Slot(_) => "Einrichten",
            Self::Bench => "Werkbank öffnen",
            Self::Forge => "Schmieden",
            Self::Loom => "Weben",
            Self::Watch(_) => "Wacht ansehen",
            Self::WatchTrace(_) => "Zeichen lesen",
            Self::Village(_) => "Benutzen",
            Self::Travel(_) => "Angekommen",
            Self::Find(_) | Self::SeasonFind(_) => "Sammeln",
            Self::SeasonMeeting => "Treffen ansehen",
            Self::Person(_) => "Ansprechen",
            Self::Binder => "Öffnen",
            Self::DuskGate => "Aufbrechen",
            Self::DuskExit => "Ins Nebelmoor",
            Self::DuskDown => "Tiefer steigen",
            Self::DuskFoe => "Kämpfen",
            Self::DuskCamp => "Ausruhen",
            Self::Resonator(_) => "Stimmen",
        }
    }

    fn height(self) -> f32 {
        match self {
            Self::Door => 0.86,
            Self::Portal => 0.95,
            Self::SaleChest => 0.53,
            Self::HarborBoard => 0.95,
            Self::HarborDriftwood => 0.22,
            Self::HarborFishing(_) => 0.34,
            Self::Kitchen | Self::PatternStall => 0.88,
            Self::FarmAnimal(_) => 0.51,
            Self::Gate(_) | Self::Bell(_) => 0.93,
            Self::Watch(_) => 1.5,
            Self::WatchTrace(_) => 0.42,
            Self::Tree(i) => forest::node(i).map_or(1.50, |node| node.top() * 0.4),
            Self::Rock(i) => forest::node(i).map_or(0.45, |node| node.top() * 0.5),
            Self::Down | Self::Up => 0.65,
            Self::Village(target) => target.height(),
            Self::Travel(_) => 0.2,
            Self::Find(i) => FINDS[i].height(),
            Self::SeasonFind(_) => 0.23,
            Self::SeasonMeeting => 0.94,
            Self::Person(_) => 1.02,
            Self::Binder | Self::DuskFoe => 0.86,
            Self::DuskGate | Self::DuskExit => 1.05,
            Self::DuskDown | Self::DuskCamp => 0.42,
            Self::Resonator(_) => 1.14,
            _ => 0.23,
        }
    }
}

pub const LANDMARKS: [(&str, Target); 9] = [
    ("Minenstollen", Target::Portal),
    ("Hof", Target::Door),
    ("Farnwald", Target::Travel(District::Forest)),
    ("Muschelbucht", Target::Travel(District::Beach)),
    ("Eichenhafen", Target::Travel(District::Town)),
    ("Kupferklamm", Target::Travel(District::Copper)),
    ("Windweiden", Target::Travel(District::Wind)),
    ("Nebelmoor", Target::Travel(District::Moor)),
    ("Dämmergänge", Target::DuskGate),
];

pub fn landmark_center(
    camera: &Camera,
    view: &GlobalTransform,
    p: Vec2,
    scale: f32,
    offset: f32,
) -> Option<Vec2> {
    camera
        .world_to_viewport(view, Vec3::new(p.x, 0.85, p.y))
        .ok()
        .map(|at| at + Vec2::Y * offset * scale)
}

pub fn landmark_offset(target: Target) -> f32 {
    if target == Target::Portal { -24. } else { 23. }
}

pub fn tracked_pin(
    adventure: &Adventure,
    garden: &Garden,
    camera: &Camera,
    view: &GlobalTransform,
    scale: f32,
) -> Option<(Target, Vec2)> {
    if !adventure.area.outdoors() {
        return None;
    }
    let target = adventure.tracked_target(garden)?;
    let point = adventure.position(target, garden);
    let at = if adventure.map {
        landmark_center(camera, view, point, scale, -36.)?
    } else {
        if adventure.focus == Some(target) {
            return None;
        }
        let height = match target {
            Target::Watch(_) => 3.36,
            Target::WatchTrace(_) => 1.12,
            Target::Person(_) => 2.18,
            Target::Bench | Target::Forge | Target::Loom | Target::HarborBoard => 1.72,
            Target::Resonator(_) => 2.54,
            _ => return None,
        };
        camera
            .world_to_viewport(
                view,
                Vec3::new(
                    point.x,
                    world::surface_height(point, garden) + height,
                    point.y,
                ),
            )
            .ok()?
    };
    let viewport = camera.logical_viewport_size()?;
    (at.cmpge(Vec2::new(53., 14.) * scale).all()
        && at.cmple(viewport - Vec2::new(53., 14.) * scale).all())
    .then_some((target, at))
}

#[derive(Resource, Default)]
pub struct Adventure {
    pub area: Area,
    pub focus: Option<Target>,
    pub crop: Crop,
    pub furniture: usize,
    pub tracked: Option<Guide>,
    pub map: bool,
    pub zoom: f32,
    waiting: Option<(Target, f32, f32)>,
    queued: Option<Target>,
    strike: Option<(Tool, f32)>,
    pub layout: MineFloor,
    sale: usize,
}

#[derive(Clone, Debug, Default)]
pub struct MineFloor {
    cells: Vec<u16>,
    ores: Vec<u16>,
    down: u16,
    encounter: u16,
}

fn neighbors(cell: u16) -> impl Iterator<Item = u16> {
    let (x, z) = (cell as i16 % GRID, cell as i16 / GRID);
    [(x - 1, z), (x + 1, z), (x, z - 1), (x, z + 1)]
        .into_iter()
        .filter(|&(x, z)| (0..GRID).contains(&x) && (0..GRID).contains(&z))
        .map(|(x, z)| (z * GRID + x) as u16)
}

fn cell_position(cell: u16) -> Vec2 {
    Vec2::new(
        f32::from(cell % GRID as u16) - 4.,
        f32::from(cell / GRID as u16) - 4.,
    ) * TILE
        + MINE
}

impl MineFloor {
    fn expedition(level: u32) -> Self {
        let mut floor = Self::generate(900 + level);
        let chapter = Chapter::at(level);
        // Keep every original tile, foe and stair usable by saves made before the redesign.
        // A connections rule adds loops and small side chambers without splitting the route.
        let extra = match chapter {
            Chapter::Moss => 2,
            Chapter::Clay | Chapter::Mist => 4,
            Chapter::Root | Chapter::Bell => 6,
        };
        for step in 0..extra {
            let cell = (0..(GRID * GRID) as u16)
                .filter(|cell| !floor.cells.contains(cell))
                .filter(|&cell| {
                    neighbors(cell)
                        .filter(|next| floor.cells.contains(next))
                        .count()
                        >= 2
                })
                .max_by_key(|&cell| (u32::from(cell) * 37 + level * 23 + step * 53) % 101);
            if let Some(cell) = cell {
                floor.cells.push(cell);
            }
        }
        floor
    }

    pub fn generate(depth: u32) -> Self {
        let mut state = 0xa8cc_113du32 ^ depth.wrapping_mul(0x9e37_79b9);
        let mut roll = |upper: usize| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as usize % upper
        };
        let mut cells = vec![START];
        for (left, back) in [(true, true), (false, true), (true, false), (false, false)] {
            let x = if left { 1 + roll(2) } else { 6 + roll(2) };
            let z = if back { 1 + roll(2) } else { 6 + roll(2) };
            let room = IVec2::new(x as i32, z as i32);
            let mut cursor = IVec2::splat(4);
            let mut carve = |point: IVec2| {
                let cell = (point.y * 9 + point.x) as u16;
                if !cells.contains(&cell) {
                    cells.push(cell);
                }
            };
            let x_first = roll(2) == 0;
            while cursor != room {
                let diff = room - cursor;
                if diff.x != 0 && (x_first || diff.y == 0) {
                    cursor.x += diff.x.signum();
                } else {
                    cursor.y += diff.y.signum();
                }
                carve(cursor);
            }
            let dx = if left { -1 } else { 1 };
            let dz = if back { -1 } else { 1 };
            for shift in [IVec2::new(dx, 0), IVec2::new(0, dz), IVec2::new(dx, dz)] {
                carve(room + shift);
            }
        }
        let mut distances = [u8::MAX; 81];
        let mut queue = std::collections::VecDeque::from([START]);
        distances[START as usize] = 0;
        while let Some(cell) = queue.pop_front() {
            for next in neighbors(cell) {
                if cells.contains(&next) && distances[next as usize] == u8::MAX {
                    distances[next as usize] = distances[cell as usize] + 1;
                    queue.push_back(next);
                }
            }
        }
        let down = *cells
            .iter()
            .max_by_key(|&&c| distances[c as usize])
            .unwrap();
        let encounter = *cells
            .iter()
            .filter(|&&cell| cell != down)
            .max_by_key(|&&cell| {
                let opposite_side = cell_position(cell).distance_squared(cell_position(down));
                u32::from(distances[cell as usize]) * 20 + opposite_side as u32
            })
            .unwrap();
        let mut candidates: Vec<_> = cells
            .iter()
            .copied()
            .filter(|&c| c != START && c != down && distances[c as usize] >= 2)
            .collect();
        let ores = (0..7)
            .map(|_| {
                let index = roll(candidates.len());
                candidates.swap_remove(index)
            })
            .collect();
        Self {
            cells,
            ores,
            down,
            encounter,
        }
    }

    fn contains(&self, point: Vec2, origin: Vec2) -> bool {
        let cell = (point - origin) / TILE + Vec2::splat(4.);
        let center = cell.round();
        if center.min_element() < 0. || center.max_element() >= f32::from(GRID) {
            return false;
        }
        self.cells
            .contains(&((center.y as u16) * GRID as u16 + center.x as u16))
    }

    fn walls(&self, cell: u16) -> impl Iterator<Item = (Vec2, f32)> + '_ {
        let (x, z) = (cell as i16 % GRID, cell as i16 / GRID);
        let turn = std::f32::consts::FRAC_PI_2;
        [(0, -1, 0.), (1, 0, turn), (0, 1, 0.), (-1, 0, turn)]
            .into_iter()
            .filter_map(move |(dx, dz, angle)| {
                let (nx, nz) = (x + dx, z + dz);
                let open = (0..GRID).contains(&nx)
                    && (0..GRID).contains(&nz)
                    && self.cells.contains(&((nz * GRID + nx) as u16));
                (!open).then_some((Vec2::new(dx as f32, dz as f32) * TILE * 0.49, angle))
            })
    }
}

pub fn dusk_open(garden: &Garden) -> bool {
    Frontier::Moor.unlocked(garden) && garden.regions.story_steps(Frontier::Moor) >= 2
}

fn dusk_cell(cell: u16) -> Vec2 {
    cell_position(cell) + (DUSK - MINE)
}

impl Adventure {
    pub fn tracked_target(&self, garden: &Garden) -> Option<Target> {
        self.tracked
            .and_then(|guide| guide.target(garden))
            .or_else(|| Guide::Campaign.target(garden))
    }

    pub fn offer(&self, garden: &Garden) -> Option<Product> {
        Product::at(self.sale, garden).or_else(|| Product::at(0, garden))
    }

    pub fn held_tool(&self) -> Tool {
        self.focus.map_or(Tool::Hoe, Target::tool)
    }

    pub fn swing(&self, now: f32) -> Option<(Tool, f32)> {
        self.strike
            .map(|(tool, start)| (tool, now - start))
            .filter(|(_, elapsed)| *elapsed < 0.52)
    }

    pub fn walkable(&self, point: Vec2, garden: &Garden) -> bool {
        let within = |center: Vec2, radius: f32| point.distance_squared(center) < radius * radius;
        let inside = |center: Vec2, half: Vec2| {
            let offset = (point - center).abs();
            offset.x < half.x && offset.y < half.y
        };
        match self.area {
            Area::Island | Area::District(_) => {
                (world::walkable(point) || garden.harbor.phase.deck_height(point).is_some())
                    && garden.regions.passage(point)
                    && !Frontier::ALL.into_iter().any(|gate| {
                        if gate.unlocked(garden) {
                            return false;
                        }
                        let half = if gate == Frontier::Wind {
                            Vec2::new(1.53, 0.26)
                        } else {
                            Vec2::new(0.26, 1.53)
                        };
                        inside(gate.position(), half)
                    })
                    && !Bell::ALL
                        .into_iter()
                        .any(|bell| within(bell.position(), 0.59))
                    && !Resonator::ALL
                        .into_iter()
                        .any(|site| within(site.position(), 0.63))
                    && !within(SALE_CHEST, 0.70)
                    && !garden.workshop.annex_at(point)
                    && !inside(workshop::BENCH, BENCH_HALF_SIZE)
                    && !(garden.workshop.loom && inside(workshop::LOOM, BENCH_HALF_SIZE))
                    && !Watch::ALL.into_iter().any(|site| {
                        inside(site.point(), Vec2::new(2.08, 1.27)) || within(site.trace(), 0.52)
                    })
                    && !(garden.workshop.forge && inside(workshop::FORGE, BENCH_HALF_SIZE))
                    && !inside(KITCHEN, Vec2::new(1.15, 0.64))
                    && !inside(PATTERN_STALL, Vec2::new(0.85, 0.51))
                    && !inside(MEETING_POINT, TABLE_HALF_SIZE)
                    && !inside(harbor::BOARD, harbor::BOARD_HALF)
                    && !Npc::ALL
                        .into_iter()
                        .any(|npc| within(npc.position(garden), NPC_RADIUS))
                    && !Animal::ALL
                        .into_iter()
                        .any(|kind| inside(kind.site(), Vec2::new(1.57, 1.22)))
                    && !forest::blocks(point, garden)
                    && !TREES.iter().enumerate().any(|(i, &tree)| {
                        let radius = if garden.felled.contains(&i) {
                            STUMP_RADIUS
                        } else {
                            TREE_RADIUS
                        };
                        within(tree, radius)
                    })
                    && !ROCKS
                        .iter()
                        .enumerate()
                        .any(|(i, &rock)| within(rock, ROCK_RADIUS) && !garden.broken.contains(&i))
            }
            Area::Home => {
                let p = point - HOME;
                garden.workshop.inside(p)
                    && !inside(self.position(Target::Bed, garden), BED_HALF_SIZE)
                    && !inside(BINDER, Vec2::new(0.52, 0.35))
                    && !inside(HOME + Vec2::new(-3.04, -2.34), Vec2::splat(0.34))
                    && !inside(HOME + Vec2::new(3.02, 2.37), Vec2::splat(0.34))
                    && !garden.furniture.iter().enumerate().any(|(i, &kind)| {
                        kind.checked_sub(1)
                            .and_then(|kind| CATALOG.get(kind as usize))
                            .is_some_and(|design| inside(HOME + SLOTS[i], design.half))
                    })
            }
            Area::Mine(depth) => {
                self.layout.contains(point, MINE)
                    && !self.layout.ores.iter().any(|&cell| {
                        within(self.position(Target::MineRock(cell), &garden), ORE_RADIUS)
                            && !garden.mined.contains(&(depth, cell))
                    })
            }
            Area::Expedition(level) => {
                [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y]
                    .into_iter()
                    .all(|offset| self.layout.contains(point + offset * 0.26, DUSK))
                    && if garden.rpg.cleared < level {
                        !within(self.position(Target::DuskFoe, &garden), 0.62)
                    } else {
                        level % CAMP_INTERVAL != 0
                            || !within(self.position(Target::DuskCamp, &garden), 0.63)
                    }
            }
        }
    }

    pub fn step_away_from_resident(&self, point: Vec2, garden: &Garden) -> Vec2 {
        if !self.area.outdoors() {
            return point;
        }
        let Some(center) = Npc::ALL
            .into_iter()
            .map(|npc| npc.position(garden))
            .find(|center| center.distance_squared(point) < NPC_RADIUS * NPC_RADIUS)
        else {
            return point;
        };
        let offset = (point - center).try_normalize().unwrap_or(Vec2::Y);
        [0., 0.8, -0.8, 1.6, -1.6, std::f32::consts::PI]
            .into_iter()
            .map(|turn| center + offset.rotate(Vec2::from_angle(turn)) * (NPC_RADIUS + 0.04))
            .find(|&p| self.walkable(p, garden))
            .unwrap_or(point)
    }

    fn reach(&self, target: Target, garden: &Garden) -> Reach {
        if target == Target::Door {
            return Reach::target(Vec2::new(DOOR.x, -0.11), 0., 0.68);
        }
        let clearance = match target {
            Target::Tree(i) => forest::node(i).map_or(TREE_RADIUS, forest::Node::radius) + 0.06,
            Target::Rock(i) => forest::node(i).map_or(ROCK_RADIUS, forest::Node::radius) + 0.06,
            Target::MineRock(_) => ORE_RADIUS + 0.06,
            Target::SaleChest => 0.74,
            Target::Kitchen => 1.18,
            Target::PatternStall => 0.94,
            Target::Gate(_) => 0.67,
            Target::Bell(_) => 0.66,
            Target::Resonator(_) => 0.69,
            Target::SeasonMeeting => 0.95,
            Target::Person(_) => NPC_RADIUS + 0.07,
            Target::HarborBoard => 0.95,
            Target::Bed => 1.10,
            Target::Bench | Target::Forge | Target::Loom => 0.91,
            Target::Watch(_) => 2.08,
            Target::WatchTrace(_) => 0.58,
            Target::Binder | Target::DuskFoe | Target::DuskCamp => 0.70,
            Target::Slot(i) => {
                let kind = garden.furniture[i]
                    .checked_sub(1)
                    .map_or(self.furniture, usize::from);
                CATALOG
                    .get(kind)
                    .map_or(0., |design| design.half.length() + 0.03)
            }
            Target::Village(Village::Orchard(i)) => {
                if i < 4 {
                    0.46
                } else {
                    0.76
                }
            }
            Target::Village(Village::Feed | Village::Animal(_)) => 0.78,
            Target::Village(Village::Seeds | Village::Sell | Village::Board | Village::Smith) => {
                1.04
            }
            _ => 0.,
        };
        let limit = if matches!(target, Target::Watch(_)) {
            2.95
        } else {
            match target.tool() {
                Tool::Axe => 1.08,
                Tool::Pickaxe => 1.20,
                Tool::Hoe | Tool::WateringCan | Tool::Rod | Tool::Weapon(_) => 1.49,
            }
        };
        Reach::target(self.position(target, &garden), clearance, limit)
    }

    fn can_reach(&self, target: Target, player: Vec2, garden: &Garden) -> bool {
        self.reach(target, garden)
            .reached(player, &|p| self.walkable(p, garden))
    }

    pub fn position(&self, target: Target, garden: &Garden) -> Vec2 {
        match target {
            Target::Door => DOOR,
            Target::Portal => PORTAL,
            Target::Pier => PIER,
            Target::HarborBoard => harbor::BOARD,
            Target::HarborDriftwood => harbor::DRIFTWOOD,
            Target::HarborFishing(spot) => spot.point(),
            Target::SaleChest => SALE_CHEST,
            Target::Kitchen => KITCHEN,
            Target::PatternStall => PATTERN_STALL,
            Target::Gate(gate) => gate.position(),
            Target::Bell(bell) => bell.position(),
            Target::FarmAnimal(kind) => kind.site() + Vec2::Y * 1.66,
            Target::Tree(i) => forest::node(i).map_or_else(|| TREES[i], |node| node.point),
            Target::Rock(i) => forest::node(i).map_or_else(|| ROCKS[i], |node| node.point),
            Target::Bed => HOME + Vec2::new(-2.03, -1.74),
            Target::Exit => HOME + Vec2::new(0., 2.73),
            Target::Bench => workshop::BENCH,
            Target::Forge => workshop::FORGE,
            Target::Loom => workshop::LOOM,
            Target::Watch(site) => site.point(),
            Target::WatchTrace(site) => site.trace(),
            Target::Slot(i) => HOME + SLOTS[i],
            Target::Up => cell_position(START) - Vec2::new(0.55, 0.57),
            Target::Down => cell_position(self.layout.down),
            Target::MineRock(cell) => cell_position(cell) + Vec2::new(0.41, -0.34),
            Target::Village(location) => location.position(),
            Target::Travel(district) => district.waypoint(),
            Target::Find(i) => FINDS[i].position(),
            Target::SeasonFind(i) => SITES[i].point,
            Target::SeasonMeeting => MEETING_POINT,
            Target::Person(person) => person.position(garden),
            Target::Binder => BINDER,
            Target::DuskGate => DUSK_GATE,
            Target::DuskExit => DUSK + Vec2::new(-0.14, -0.77),
            Target::DuskDown => dusk_cell(self.layout.down),
            Target::DuskFoe | Target::DuskCamp => dusk_cell(self.layout.encounter),
            Target::Resonator(site) => site.position(),
        }
    }

    fn targets(&self, garden: &Garden, near: Vec2) -> Vec<Target> {
        match self.area {
            Area::Island | Area::District(_) => [
                Target::Door,
                Target::Portal,
                Target::Pier,
                Target::HarborBoard,
                Target::SaleChest,
                Target::Bench,
                Target::Kitchen,
                Target::PatternStall,
                Target::SeasonMeeting,
            ]
            .into_iter()
            .chain(
                garden
                    .harbor
                    .salvage_available()
                    .then_some(Target::HarborDriftwood),
            )
            .chain(
                Fishing::ALL
                    .into_iter()
                    .filter(|spot| spot.open(garden.harbor.phase))
                    .map(Target::HarborFishing),
            )
            .chain(garden.workshop.forge.then_some(Target::Forge))
            .chain(garden.workshop.loom.then_some(Target::Loom))
            .chain(
                Watch::ALL
                    .into_iter()
                    .filter(|site| site.accessible(garden))
                    .map(Target::Watch),
            )
            .chain(
                Watch::ALL
                    .into_iter()
                    .filter(|site| site.stage(garden) == WatchStage::Surveying)
                    .map(Target::WatchTrace),
            )
            .chain(Animal::ALL.into_iter().map(Target::FarmAnimal))
            .chain(Frontier::Moor.unlocked(garden).then_some(Target::DuskGate))
            .chain(
                Frontier::ALL
                    .into_iter()
                    .filter(|&gate| !gate.unlocked(garden))
                    .map(Target::Gate),
            )
            .chain(Bell::ALL.into_iter().map(Target::Bell))
            .chain(
                Resonator::ALL
                    .into_iter()
                    .filter(|site| site.frontier().is_none_or(|region| region.unlocked(garden)))
                    .map(Target::Resonator),
            )
            .chain(
                District::ALL
                    .into_iter()
                    .filter(|&district| {
                        Frontier::for_district(district)
                            .is_none_or(|frontier| frontier.unlocked(garden))
                    })
                    .map(Target::Travel),
            )
            .chain(Village::all().map(Target::Village))
            .chain(
                Npc::ALL
                    .into_iter()
                    .filter(|npc| {
                        npc.frontier()
                            .is_none_or(|frontier| frontier.unlocked(garden))
                    })
                    .map(Target::Person),
            )
            .chain(
                FINDS
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &find)| find.available(garden, i).then_some(Target::Find(i))),
            )
            .chain((0..SITES.len()).filter_map(|i| {
                garden
                    .seasons
                    .available(i, garden.day)
                    .then_some(Target::SeasonFind(i))
            }))
            .chain(
                (0..TREES.len())
                    .filter(|i| !garden.felled.contains(i))
                    .map(Target::Tree),
            )
            .chain(
                (0..ROCKS.len())
                    .filter(|i| !garden.broken.contains(i))
                    .map(Target::Rock),
            )
            .chain(
                forest::near(near, if self.map { 0. } else { 46. }).filter_map(|node| {
                    if node.is_tree() {
                        (!garden.felled.contains(&node.id)).then_some(Target::Tree(node.id))
                    } else {
                        (!garden.broken.contains(&node.id)).then_some(Target::Rock(node.id))
                    }
                }),
            )
            .collect(),
            Area::Home => [Target::Bed, Target::Exit, Target::Binder]
                .into_iter()
                .chain((0..garden.workshop.slots()).map(Target::Slot))
                .collect(),
            Area::Mine(depth) => [Target::Up, Target::Down]
                .into_iter()
                .chain(
                    self.layout
                        .ores
                        .iter()
                        .filter(|&&i| !garden.mined.contains(&(depth, i)))
                        .copied()
                        .map(Target::MineRock),
                )
                .collect(),
            Area::Expedition(level) => [Target::DuskExit, Target::DuskDown]
                .into_iter()
                .chain(if garden.rpg.cleared < level {
                    Some(Target::DuskFoe)
                } else if level % CAMP_INTERVAL == 0 {
                    Some(Target::DuskCamp)
                } else {
                    None
                })
                .collect(),
        }
    }

    pub fn context(
        &self,
        garden: &Garden,
        player: Vec2,
        now: f32,
    ) -> (String, String, &'static str, bool) {
        if let Some(target) = self.focus {
            let walking = self.queued == Some(target);
            if let Some(reason) = target.restriction(garden) {
                return (target.name().into(), reason.into(), "Geschlossen", false);
            }
            if target == Target::Village(Village::Seeds) {
                return (
                    target.name().into(),
                    if walking {
                        "Auf dem Weg …".into()
                    } else {
                        "Saatgut für sechs Kulturen und Dünger.".into()
                    },
                    "Einkaufen",
                    true,
                );
            }
            if target == Target::Village(Village::Board) {
                return (
                    "Auftragsbrett".into(),
                    "Hafenaufträge annehmen oder den Hofauftrag im Journal ansehen.".into(),
                    "Brett ansehen",
                    true,
                );
            }
            let regional_action = match target {
                Target::Gate(gate) => Some(gate.describe(garden)),
                Target::Bell(bell) => Some(bell.describe(garden)),
                Target::Resonator(site) => Some(site.describe(garden)),
                Target::DuskDown => match self.area {
                    Area::Expedition(level) if garden.rpg.cleared < level => Some((
                        "Besiege zuerst den Hüter dieser Ebene.".into(),
                        "Versiegelt",
                        false,
                    )),
                    Area::Expedition(LAST_LEVEL) => Some((
                        "Du hast alle 25 Ebenen gemeistert. Ruhe dich an der Glocke aus.".into(),
                        "Erkundet",
                        false,
                    )),
                    _ => None,
                },
                _ => None,
            };
            if let Some((hint, action, enabled)) = regional_action {
                return (
                    target.name().into(),
                    if walking {
                        "Auf dem Weg …".into()
                    } else {
                        hint
                    },
                    action,
                    enabled,
                );
            }
            if let Target::FarmAnimal(kind) = target {
                let (hint, action, enabled) = if garden.farm.housing[kind as usize] {
                    Village::Animal(kind).describe(garden, self.crop, None)
                } else {
                    let (hint, available) = kind.housing_hint(garden);
                    (hint, "Stall bauen", available)
                };
                return (
                    target.name().into(),
                    if walking {
                        "Auf dem Weg …".into()
                    } else {
                        hint
                    },
                    action,
                    enabled,
                );
            }
            if let Target::Village(location) = target {
                let (detail, action, enabled) =
                    location.describe(garden, self.crop, self.offer(garden));
                return (
                    target.name().into(),
                    if walking {
                        "Auf dem Weg …".into()
                    } else {
                        detail
                    },
                    action,
                    enabled,
                );
            }
            if let (Target::MineRock(cell), Area::Mine(depth)) = (target, self.area) {
                let ore = Ore::at(depth, cell);
                let amount = if garden.farm.pickaxe { 2 } else { 1 };
                return (
                    ore.title().into(),
                    if walking {
                        "Auf dem Weg zur Ader …".into()
                    } else {
                        format!("Spitzhacke · +{amount} {} und 1 Stein", ore.name())
                    },
                    "Erz abbauen",
                    true,
                );
            }
            let (hint, action) = match target {
                Target::Slot(i) => {
                    let old = garden.furniture[i]
                        .checked_sub(1)
                        .and_then(|i| CATALOG.get(i as usize));
                    (
                        format!(
                            "{} · {}",
                            old.map_or("Freier Platz", |design| design.name),
                            CATALOG[self.furniture].name
                        ),
                        if old.is_some() {
                            "Einpacken"
                        } else {
                            "Aufstellen"
                        },
                    )
                }
                Target::Bench if !garden.workshop.restored => (
                    "Diese Werkbank braucht zwei Holz und einen Stein.".into(),
                    "Werkbank ansehen",
                ),
                Target::Bench => (
                    "Möbel, Bauprojekte und Kupferwerkzeuge.".into(),
                    "Werkbank öffnen",
                ),
                Target::Forge => (
                    "Hier verbesserst du deine Werkzeuge auf Silber.".into(),
                    "Schmieden",
                ),
                Target::Kitchen => ("Kochen, Einmachen und Mahlen.".into(), "Küche öffnen"),
                Target::Watch(site) => (site.summary(garden), target.label()),
                Target::WatchTrace(_) => {
                    ("Eine eingeritzte Spur der Wegwacht".into(), target.label())
                }
                Target::Loom => (
                    "Flachs und Wolle zu Segeltuch weben.".into(),
                    target.label(),
                ),
                Target::PatternStall => (
                    "Hier kaufst du Baupläne für deine Werkbank.".into(),
                    "Baupläne",
                ),
                Target::SeasonMeeting => (seasons::meeting_status(garden), target.label()),
                Target::SeasonFind(_) => (
                    "Saisonfund · Sammlung und Dorftreffen".into(),
                    target.label(),
                ),
                Target::HarborBoard => (garden.harbor.phase.label().into(), target.label()),
                Target::HarborDriftwood => ("Bretter für Bos Fischersteg".into(), target.label()),
                Target::Pier | Target::HarborFishing(_) => match self.waiting {
                    Some((spot, bite, end)) if spot == target && now >= bite && now <= end => {
                        ("Ein Fisch beißt an!".into(), "Einholen!")
                    }
                    Some((spot, _, end)) if spot == target && now <= end => {
                        ("Warte auf den Biss …".into(), "Einholen")
                    }
                    _ => ("Beim Biss die rechte Taste antippen.".into(), "Auswerfen"),
                },
                Target::Bed => (
                    "Ein neuer Tag beginnt. Gegossene Pflanzen wachsen.".into(),
                    "Schlafen",
                ),
                Target::Door => ("Über die Veranda gelangst du hinein.".into(), "Eintreten"),
                Target::SaleChest => (
                    "Waren auswählen und sofort verkaufen.".into(),
                    "Truhe öffnen",
                ),
                Target::Tree(_) => (
                    format!("Axt · +{} Holz", garden.wood_per_tree()),
                    "Baum fällen",
                ),
                Target::Rock(_) => (
                    format!("Spitzhacke · +{} Stein", garden.stone_per_rock()),
                    "Stein abbauen",
                ),
                Target::Travel(district) => {
                    let near = self.can_reach(target, player, garden);
                    (
                        district.hint().into(),
                        if near { "Angekommen" } else { "Hingehen" },
                    )
                }
                Target::Person(person) => (person.whereabouts(garden).into(), target.label()),
                Target::Find(i) => (FINDS[i].reward(), target.label()),
                Target::Binder => ("Sammlung, Kampfdeck und Talente.".into(), "Binder öffnen"),
                Target::DuskGate => ("Wähle, wo deine Erkundung beginnt.".into(), target.label()),
                Target::DuskFoe => match self.area {
                    Area::Expedition(level) => (
                        format!("Rundenkampf · {}", Enemy::at(level).name()),
                        target.label(),
                    ),
                    _ => (String::new(), target.label()),
                },
                Target::DuskCamp => (
                    format!(
                        "{} / {} Leben · Erreichbar vom Moorbogen",
                        garden.rpg.hp,
                        garden.rpg.max_hp()
                    ),
                    target.label(),
                ),
                Target::DuskExit => (
                    "Kehre zum Steinbogen im Nebelmoor zurück.".into(),
                    target.label(),
                ),
                _ => (String::new(), target.label()),
            };
            return (
                target.name().into(),
                if walking {
                    "Auf dem Weg …".into()
                } else {
                    hint
                },
                action,
                action != "Angekommen",
            );
        }
        if self.area == Area::Island
            && player.length() < 15.
            && garden.onboarding != Onboarding::Done
        {
            return (
                "Dein erster Morgen".into(),
                garden.onboarding.hint().into(),
                "Ort wählen",
                true,
            );
        }
        match self.area {
            Area::Island | Area::District(_) => District::at(player).map_or_else(
                || {
                    (
                        "Waldwinkel".into(),
                        "Tippe auf einen Ort oder folge den Wegen.".into(),
                        "Ort wählen",
                        true,
                    )
                },
                |district| {
                    (
                        district.name().into(),
                        district.hint().into(),
                        "Ort wählen",
                        true,
                    )
                },
            ),
            Area::Home => (
                "Deine Hütte".into(),
                "Tippe auf ein Möbel, das Bett oder den Kartenbinder. Die Werkbank steht auf dem Hof.".into(),
                "Ort wählen",
                true,
            ),
            Area::Mine(depth) => (
                format!("Mine · Tiefe {depth}"),
                "Tippe auf Erz oder eine Leiter.".into(),
                "Ort wählen",
                true,
            ),
            Area::Expedition(level) => (
                format!("{} · Ebene {level}", Chapter::at(level).name()),
                if garden.rpg.cleared < level {
                    "Suche den Hüter. Die Karte zeigt die Gänge.".into()
                } else {
                    "Der Abstieg ist frei. Erkunden oder zum Steinbogen zurückkehren.".into()
                },
                "Ort wählen",
                true,
            ),
        }
    }

    pub(crate) fn enter(
        &mut self,
        area: Area,
        player: &mut Gardener,
        transform: &mut Transform,
        from: Vec2,
    ) {
        self.area = area;
        self.focus = None;
        self.queued = None;
        self.strike = None;
        self.waiting = None;
        self.map = false;
        match area {
            Area::Mine(depth) => self.layout = MineFloor::generate(depth),
            Area::Expedition(level) => self.layout = MineFloor::expedition(level),
            _ => {}
        }
        player.teleport(from);
        transform.translation = Vec3::new(from.x, 0., from.y);
    }

    pub fn enter_duel(
        &mut self,
        level: u32,
        at_camp: bool,
        garden: &mut Garden,
        player: &mut Gardener,
        transform: &mut Transform,
    ) {
        let area = Area::Expedition(level);
        self.enter(area, player, transform, DUSK + Vec2::new(0.42, 0.38));
        if at_camp {
            garden.rpg.rest();
            let position = self.position(Target::DuskCamp, &garden) + Vec2::Y * 0.79;
            player.teleport(position);
            transform.translation = Vec3::new(position.x, 0., position.y);
        }
    }

    pub fn escape_duel(
        &mut self,
        garden: &mut Garden,
        player: &mut Gardener,
        transform: &mut Transform,
    ) -> (Area, Vec2) {
        let camp = garden.rpg.camp();
        if camp == 0 {
            garden.rpg.rest();
            self.enter(Area::Island, player, transform, DUSK_ARRIVAL);
        } else {
            self.enter_duel(camp, true, garden, player, transform);
        }
        (self.area, player.position)
    }

    pub fn restore(
        &mut self,
        checkpoint: Option<Checkpoint>,
        player: &mut Gardener,
        transform: &mut Transform,
        garden: &Garden,
    ) {
        *self = Self::default();
        if let Some(checkpoint) = checkpoint.filter(|point| match point.area {
            Area::Mine(0) => false,
            Area::Expedition(level) => {
                dusk_open(garden) && (1..=LAST_LEVEL.min(garden.rpg.cleared + 1)).contains(&level)
            }
            _ => true,
        }) {
            let position = Vec2::from_array(checkpoint.position);
            if position.is_finite() {
                self.enter(checkpoint.area, player, transform, position);
                if self.walkable(position, garden) {
                    return;
                }
                for radius in [0.35, 0.6, 1.0, 1.6, 2.4, 3.2] {
                    if let Some(nearby) = (0..16)
                        .map(|i| {
                            position
                                + Vec2::from_angle(i as f32 * std::f32::consts::TAU / 16.) * radius
                        })
                        .find(|&p| self.walkable(p, garden))
                    {
                        player.teleport(nearby);
                        let y = if self.area.outdoors() {
                            world::exterior_ground(nearby)
                        } else {
                            0.
                        };
                        transform.translation = Vec3::new(nearby.x, y, nearby.y);
                        return;
                    }
                }
                let landing = match self.area {
                    Area::Home => HOME + Vec2::new(0., 2.23),
                    Area::Mine(_) => MINE + Vec2::new(0.42, 0.38),
                    Area::Expedition(_) => DUSK + Vec2::new(0.42, 0.38),
                    Area::District(district) => district.waypoint(),
                    Area::Island => world::SPAWN,
                };
                self.enter(checkpoint.area, player, transform, landing);
                return;
            }
        }
        self.enter(Area::Island, player, transform, world::SPAWN);
    }
}

fn ray_box(ray: Ray3d, min: Vec3, max: Vec3) -> Option<f32> {
    let mut enter = 0.0_f32;
    let mut exit = f32::INFINITY;
    for axis in 0..3 {
        let speed = ray.direction[axis];
        let origin = ray.origin[axis];
        if speed.abs() < 0.00001 {
            if origin < min[axis] || origin > max[axis] {
                return None;
            }
        } else {
            let a = (min[axis] - origin) / speed;
            let b = (max[axis] - origin) / speed;
            enter = enter.max(a.min(b));
            exit = exit.min(a.max(b));
        }
    }
    (enter <= exit).then_some(enter)
}

fn hit_plot(ray: Ray3d, garden: &Garden) -> Option<(usize, f32)> {
    world::PLOTS
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(i, point)| {
            let center = Vec3::new(point.x, 0., point.y);
            let frame = ray_box(
                ray,
                center + Vec3::new(-0.67, 0., -0.67),
                center + Vec3::new(0.67, 0.25, 0.67),
            );
            let plant = match garden.plots[i].stage() {
                Stage::Sprout => 0.29,
                Stage::Young => 0.50,
                Stage::Ripe if garden.plots[i].crop() == Crop::Sunflower => 1.36,
                Stage::Ripe => 0.88,
                _ => 0.,
            };
            let canopy = (plant > 0.)
                .then(|| {
                    ray_box(
                        ray,
                        center + Vec3::new(-0.51, 0.19, -0.51),
                        center + Vec3::new(0.51, plant, 0.51),
                    )
                })
                .flatten();
            frame
                .into_iter()
                .chain(canopy)
                .min_by(f32::total_cmp)
                .map(|depth| (i, depth))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
}

fn hit_volume(ray: Ray3d, target: Target, base: Vec3, garden: &Garden) -> Option<f32> {
    let hit = |half: Vec2, bottom, top| {
        ray_box(
            ray,
            base + Vec3::new(-half.x, bottom, -half.y),
            base + Vec3::new(half.x, top, half.y),
        )
    };
    match target {
        Target::Door => [
            ray_box(
                ray,
                Vec3::new(-5.20, 0., -4.36),
                Vec3::new(-0.10, 3.51, 0.78),
            ),
            ray_box(
                ray,
                Vec3::new(-4.44, 3.51, -4.38),
                Vec3::new(-0.88, 4.17, -0.46),
            ),
            ray_box(
                ray,
                Vec3::new(-3.73, 4.17, -4.27),
                Vec3::new(-1.59, 4.76, -0.54),
            ),
            ray_box(
                ray,
                Vec3::new(-4.30, 3.40, -3.43),
                Vec3::new(-3.56, 5.27, -2.61),
            ),
        ]
        .into_iter()
        .chain(
            workshop::ANNEXES
                .into_iter()
                .take(garden.workshop.house as usize)
                .map(|(p, half)| {
                    ray_box(
                        ray,
                        Vec3::new(p.x - half.x, 0., p.y - half.y),
                        Vec3::new(p.x + half.x, 3.2, p.y + half.y),
                    )
                }),
        )
        .flatten()
        .min_by(f32::total_cmp),
        Target::Tree(i) => {
            let (scale, top) = forest::node(i).map_or((1., 4.36), |node| (node.scale, node.top()));
            hit(Vec2::splat(1.11 * scale), 1.00 * scale, top)
                .or_else(|| hit(Vec2::splat(0.38 * scale), 0., 1.04 * scale))
        }
        Target::Rock(i) => {
            let (scale, top) = forest::node(i).map_or((1., 0.97), |node| (node.scale, node.top()));
            hit(Vec2::new(1.0, 0.86) * scale, 0., top)
        }
        Target::MineRock(_) => hit(Vec2::splat(0.76), 0., 0.97),
        Target::SaleChest => hit(Vec2::new(0.64, 0.46), 0., 0.84),
        Target::Gate(gate) => hit(
            if gate == Frontier::Wind {
                Vec2::new(1.66, 0.42)
            } else {
                Vec2::new(0.42, 1.66)
            },
            0.,
            1.8,
        ),
        Target::Bell(_) => hit(Vec2::splat(0.67), 0., 1.95),
        Target::Resonator(_) => hit(Vec2::splat(0.53), 0., 1.62),
        Target::SeasonMeeting => hit(TABLE_HALF_SIZE, 0., 1.85),
        Target::HarborBoard => hit(harbor::BOARD_HALF, 0., 1.71),
        Target::HarborDriftwood => hit(Vec2::new(0.63, 0.34), 0., 0.38),
        Target::HarborFishing(_) => hit(Vec2::new(0.55, 0.49), 0., 0.70),
        Target::Person(_) => hit(Vec2::splat(0.42), 0., 1.62),
        Target::SeasonFind(_) => hit(Vec2::splat(0.36), 0., 0.32),
        Target::Kitchen | Target::PatternStall => hit(Vec2::new(0.96, 0.61), 0., 1.82),
        Target::FarmAnimal(_) => ray_box(
            ray,
            base + Vec3::new(-1.50, 0., -3.15),
            base + Vec3::new(1.50, 1.98, 0.28),
        ),
        Target::Bed => hit(BED_HALF_SIZE, 0., 1.03),
        Target::Bench | Target::Forge | Target::Loom => hit(BENCH_HALF_SIZE, 0., 1.16),
        Target::Watch(Watch::Fern) => hit(Vec2::new(2.0, 1.35), 0., 3.66),
        Target::Watch(_) => hit(Vec2::new(2.0, 1.35), 0., 2.87),
        Target::WatchTrace(_) => hit(Vec2::splat(0.47), 0., 0.65),
        Target::Binder => hit(Vec2::new(0.52, 0.35), 0., 1.03),
        Target::DuskExit | Target::DuskGate => hit(Vec2::new(1.42, 0.52), 0., 2.15),
        Target::DuskFoe => hit(Vec2::splat(0.60), 0., 1.43),
        Target::DuskCamp => hit(Vec2::splat(0.85), 0., 0.85),
        Target::DuskDown => hit(Vec2::new(0.81, 0.92), 0., 0.61),
        Target::Slot(i) => {
            let kind = garden.furniture[i].checked_sub(1)? as usize;
            let design = CATALOG.get(kind)?;
            hit(design.half.max(Vec2::splat(0.47)), 0., design.height)
        }
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PadTarget {
    Plot(usize),
    Place(Target),
}

pub fn select(
    mut controls: ResMut<Controls>,
    layout: Res<crate::ui::HudLayout>,
    mut adventure: ResMut<Adventure>,
    garden: Res<Garden>,
    mut focus: ResMut<GardenFocus>,
    camera: Single<(&Camera, &GlobalTransform), With<IslandCamera>>,
    mut player: Single<&mut Gardener>,
) {
    if controls.cancel {
        adventure.map = false;
        adventure.focus = None;
        adventure.queued = None;
        adventure.waiting = None;
        *focus = GardenFocus::default();
        player.stop();
        return;
    }
    let auto = controls.pad_action && focus.selected.is_none() && adventure.focus.is_none();
    if controls.pad_cycle != 0 || auto {
        let point = player.position;
        let mut targets: Vec<_> = adventure
            .targets(&garden, point)
            .into_iter()
            .filter(|target| !matches!(target, Target::Travel(_)))
            .map(PadTarget::Place)
            .chain(
                (0..world::PLOTS.len())
                    .filter(|_| adventure.area == Area::Island)
                    .map(PadTarget::Plot),
            )
            .map(|entry| {
                let at = match entry {
                    PadTarget::Plot(i) => world::PLOTS[i],
                    PadTarget::Place(target) => adventure.position(target, &garden),
                };
                (point.distance_squared(at), entry)
            })
            .filter(|(distance, _)| *distance < 6.4 * 6.4)
            .collect();
        targets.sort_by(|a, b| a.0.total_cmp(&b.0));
        let current = focus
            .selected
            .map(PadTarget::Plot)
            .or(adventure.focus.map(PadTarget::Place));
        let chosen = targets
            .iter()
            .position(|(_, entry)| Some(*entry) == current)
            .filter(|_| !auto)
            .map_or_else(
                || {
                    if controls.pad_cycle < 0 {
                        targets.len().saturating_sub(1)
                    } else {
                        0
                    }
                },
                |i| {
                    (i as isize + controls.pad_cycle as isize).rem_euclid(targets.len() as isize)
                        as usize
                },
            );
        let chosen = if auto {
            let tutorial_plot = match garden.onboarding {
                Onboarding::Till => Some(crate::garden::Action::Till),
                Onboarding::Sow => Some(crate::garden::Action::Sow),
                Onboarding::Water => Some(crate::garden::Action::Water),
                _ => None,
            };
            targets
                .iter()
                .position(|(_, entry)| {
                    matches!(entry, PadTarget::Plot(i) if tutorial_plot.is_some()
                    && garden.action(*i, adventure.crop) == tutorial_plot)
                })
                .or_else(|| {
                    adventure.tracked_target(&garden).and_then(|goal| {
                        targets
                            .iter()
                            .position(|(_, entry)| *entry == PadTarget::Place(goal))
                    })
                })
                .unwrap_or(chosen)
        } else {
            chosen
        };
        if let Some(&(_, target)) = targets.get(chosen) {
            adventure.map = false;
            adventure.queued = None;
            adventure.waiting = None;
            focus.pending = None;
            player.stop();
            match target {
                PadTarget::Plot(i) => {
                    adventure.focus = None;
                    focus.selected = Some(i);
                }
                PadTarget::Place(place) => {
                    adventure.focus = Some(place);
                    focus.selected = None;
                }
            }
        }
        controls.act = false;
        return;
    }
    let (camera, view) = *camera;
    let Some(at) = controls.pick_at else {
        if controls.cycle && adventure.area.outdoors() {
            adventure.focus = None;
            adventure.queued = None;
        }
        return;
    };
    adventure.queued = None;
    adventure.waiting = None;
    let ray = camera.viewport_to_world(view, at).ok();
    let mut volume = None;
    let mut best = (f32::MAX, None);
    for target in adventure.targets(&garden, player.position) {
        let point = adventure.position(target, &garden);
        let node = match target {
            Target::Tree(id) | Target::Rock(id) => forest::node(id),
            _ => None,
        };
        let y = node.map_or_else(
            || {
                if adventure.area.outdoors() {
                    world::surface_height(point, &garden)
                } else {
                    0.
                }
            },
            |node| node.height,
        );
        let base = Vec3::new(point.x, y, point.y);
        if let Some(depth) = ray.and_then(|ray| hit_volume(ray, target, base, &garden)) {
            if volume.is_none_or(|(closest, _)| depth < closest) {
                volume = Some((depth, target));
            }
        }
        let world = base + Vec3::Y * target.height();
        let Ok(projected) = camera.world_to_viewport(view, world) else {
            continue;
        };
        let radius = camera
            .world_to_viewport(view, world + Vec3::X * 0.82)
            .map_or(18., |p| p.distance(projected))
            .max(12.);
        let dist = projected.distance(at) / radius;
        if dist <= 1. && dist < best.0 {
            best = (dist, Some(target));
        }
    }
    let landmark = tracked_pin(&adventure, &garden, camera, view, layout.scale)
        .filter(|(_, p)| {
            (*p - at)
                .abs()
                .cmple(Vec2::new(53., 14.) * layout.scale)
                .all()
        })
        .map(|(target, _)| target)
        .or_else(|| {
            if !adventure.map || !adventure.area.outdoors() {
                return None;
            }
            LANDMARKS
                .into_iter()
                .filter_map(|(_, target)| {
                    let center = landmark_center(
                        camera,
                        view,
                        adventure.position(target, &garden),
                        layout.scale,
                        landmark_offset(target),
                    )?;
                    let delta = (at - center).abs() / layout.scale;
                    let destination = if let Target::Travel(district) = target {
                        Frontier::for_district(district)
                            .filter(|gate| !gate.unlocked(&garden))
                            .map_or(target, Target::Gate)
                    } else {
                        target
                    };
                    (delta.x <= 53. && delta.y <= 12.)
                        .then_some((destination, delta.length_squared()))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(target, _)| target)
        });
    if landmark.is_none()
        && !adventure.map
        && adventure.area == Area::Island
        && let Some((index, depth)) = ray.and_then(|ray| hit_plot(ray, &garden))
        && volume.is_none_or(|(other, _)| depth < other)
    {
        focus.selected = Some(index);
        focus.pending = None;
        adventure.focus = None;
        player.stop();
        return;
    }
    adventure.focus = landmark.or(volume.map(|(_, target)| target)).or(best.1);
    adventure.sale = 0;
    player.stop();
    if let Some(target) = adventure.focus {
        if landmark.is_some() || matches!(target, Target::Travel(_)) {
            player.navigate(adventure.reach(target, &garden), &adventure, &garden);
        }
    } else if let Some(point) =
        ray.and_then(|ray| world::ground_point(ray, adventure.area, &garden))
    {
        if let Some(gate) = Frontier::ALL.into_iter().find(|&gate| {
            !gate.unlocked(&garden)
                && gate.boundary(point)
                && player.position.distance(gate.position()) < 18.
        }) {
            let target = Target::Gate(gate);
            adventure.focus = Some(target);
            player.navigate(adventure.reach(target, &garden), &adventure, &garden);
        } else {
            player.navigate(Reach::walk(point), &adventure, &garden);
        }
    }
    adventure.map = false;
    focus.selected = None;
    focus.pending = None;
}

pub fn interact(
    controls: Res<Controls>,
    time: Res<Time>,
    mut adventure: ResMut<Adventure>,
    mut player: Single<(&mut Gardener, &mut Transform)>,
    mut garden: ResMut<Garden>,
    path: Res<SavePath>,
    mut focus: ResMut<GardenFocus>,
    mut notice: ResMut<Notice>,
    mut dialog: ResMut<Dialog>,
    mut farm_menu: ResMut<FarmMenu>,
    mut breaks: EventWriter<Break>,
) {
    if controls.zoom != 0. {
        adventure.map = false;
        adventure.zoom = (adventure.zoom + controls.zoom).clamp(-0.7, 1.25);
    }
    if controls.map {
        adventure.map = !adventure.map;
    }
    if controls.inventory {
        farm_menu.show(FarmPage::Inventory);
        return;
    }
    if controls.choice {
        if adventure.area == Area::Home {
            farm_menu.show(FarmPage::Furnish);
            return;
        } else if adventure.focus == Some(Target::Village(Village::Sell)) {
            let next = adventure.sale + 1;
            adventure.sale = if Product::at(next, &garden).is_some() {
                next
            } else {
                0
            };
        } else {
            if focus.pending.take().is_some() {
                player.0.stop();
            }
            farm_menu.show(FarmPage::Seeds);
            return;
        }
    }
    let now = time.elapsed_secs();
    if controls.task {
        adventure.queued = None;
        player.0.stop();
        farm_menu.show(FarmPage::Journal(GuideTab::Story));
        return;
    }
    if controls.move_axis != Vec2::ZERO || focus.selected.is_some() {
        adventure.queued = None;
        adventure.waiting = None;
        return;
    }
    if !controls.act && adventure.queued.is_none() {
        return;
    }
    let (player, transform) = &mut *player;
    let Some(target) = adventure.focus else {
        adventure.focus = adventure
            .targets(&garden, player.position)
            .into_iter()
            .min_by(|&a, &b| {
                player
                    .position
                    .distance_squared(adventure.position(a, &garden))
                    .total_cmp(
                        &player
                            .position
                            .distance_squared(adventure.position(b, &garden)),
                    )
            });
        adventure.queued = None;
        return;
    };
    if let Some(reason) = target.restriction(&garden) {
        adventure.queued = None;
        player.stop();
        *notice = Notice {
            message: reason.into(),
            until: now + Notice::DURATION,
            ..default()
        };
        return;
    }
    if !adventure.can_reach(target, player.position, &garden) {
        if adventure.queued == Some(target) && player.approaching() {
            return;
        }
        adventure.map = false;
        let follow_resident =
            matches!(target, Target::Person(_)) && adventure.queued == Some(target);
        if (controls.act || follow_resident)
            && player.navigate(adventure.reach(target, &garden), &adventure, &garden)
        {
            adventure.queued = Some(target);
        } else {
            adventure.queued = None;
            *notice = Notice {
                message: "Hier komme ich gerade nicht hin.".into(),
                until: now + Notice::DURATION,
                ..default()
            };
        }
        return;
    }
    adventure.queued = None;
    let position = player.position;
    let target_point = adventure.position(target, &garden);
    let area = adventure.area;
    let hit = |model: &'static str, heading: f32, tree: bool| Break {
        area,
        point: target_point,
        away: target_point - position,
        heading,
        model,
        tree,
        floor: None,
        scale: 1.,
        snow_cap: tree,
    };
    player.stop();
    let tool = target.tool();
    if matches!(tool, Tool::Axe | Tool::Pickaxe | Tool::Rod) {
        let direction = adventure.position(target, &garden) - position;
        transform.rotation = Quat::from_rotation_y(direction.x.atan2(direction.y));
        if tool != Tool::Rod {
            adventure.strike = Some((tool, now));
        }
    }
    let mut persistent = true;
    let message = match target {
        Target::Travel(_) => return,
        Target::Bell(bell) if bell.ready(&garden) => {
            farm_menu.show(FarmPage::Travel(bell));
            return;
        }
        Target::Gate(_) | Target::Bell(_) => {
            let action = match target {
                Target::Gate(gate) => gate.work(&mut garden),
                Target::Bell(bell) => bell.work(&mut garden),
                _ => unreachable!(),
            };
            match action {
                Ok(message) => {
                    if let Target::Gate(gate) = target
                        && gate.unlocked(&garden)
                        && player.navigate(Reach::walk(gate.entrance()), &adventure, &garden)
                    {
                        adventure.focus = None;
                    }
                    message
                }
                Err(message) => {
                    persistent = false;
                    message
                }
            }
        }
        Target::Resonator(site) => match site.work(&mut garden) {
            Ok(message) => {
                if site == Resonator::Harbor {
                    farm_menu.show(FarmPage::Cinematic(crate::cinematic::Clip::Festival));
                }
                message
            }
            Err(message) => {
                persistent = false;
                message
            }
        },
        Target::SaleChest
        | Target::Village(Village::Seeds)
        | Target::Binder
        | Target::DuskGate
        | Target::Bench
        | Target::Forge
        | Target::Loom
        | Target::Watch(_)
        | Target::Village(Village::Board)
        | Target::PatternStall
        | Target::Kitchen
        | Target::HarborBoard
        | Target::SeasonMeeting => {
            farm_menu.show(match target {
                Target::SaleChest => FarmPage::Sale,
                Target::Bench if garden.workshop.restored => FarmPage::Workbench,
                Target::Bench => FarmPage::Projects,
                Target::Forge => FarmPage::Tools { at_forge: true },
                Target::Loom => FarmPage::Materials(crate::craft::Bench::Loom),
                Target::Watch(site) => FarmPage::Watch(site),
                Target::Village(Village::Board) => FarmPage::Commissions,
                Target::PatternStall => FarmPage::BlueprintShop,
                Target::Kitchen => FarmPage::Kitchen,
                Target::Binder => FarmPage::Binder,
                Target::DuskGate => FarmPage::Expedition,
                Target::SeasonMeeting => FarmPage::SeasonMeeting,
                Target::HarborBoard => FarmPage::Harbor { at_site: true },
                _ => FarmPage::SeedShop,
            });
            if target == Target::Binder {
                farm_menu.load_deck(&garden);
            }
            return;
        }
        Target::DuskExit => {
            adventure.enter(Area::Island, player, transform, DUSK_ARRIVAL);
            "Zurück im Nebelmoor.".into()
        }
        Target::DuskDown => {
            let Area::Expedition(level) = adventure.area else {
                return;
            };
            if garden.rpg.cleared < level || level >= LAST_LEVEL {
                return;
            }
            adventure.enter_duel(level + 1, false, &mut garden, player, transform);
            format!("Dämmergänge · Ebene {}", level + 1)
        }
        Target::DuskFoe => {
            let Area::Expedition(level) = adventure.area else {
                return;
            };
            if garden.rpg.cleared >= level {
                return;
            }
            let direction = adventure.position(target, &garden) - player.position;
            transform.rotation = Quat::from_rotation_y(direction.x.atan2(direction.y));
            adventure.map = false;
            garden.checkpoint = Some(Checkpoint::new(adventure.area, player.position));
            farm_menu.show_battle(Battle::new(level, &garden.rpg));
            return;
        }
        Target::DuskCamp => {
            garden.rpg.rest();
            "Ausgeruht. Diese Glockenrast erreichst du vom Moorbogen.".into()
        }
        Target::FarmAnimal(kind) => {
            let action = if garden.farm.housing[kind as usize] {
                Village::Animal(kind).work(&mut garden, adventure.crop, None)
            } else {
                kind.build_housing(&mut garden)
            };
            match action {
                Ok(message) => message,
                Err(message) => {
                    persistent = false;
                    message
                }
            }
        }
        Target::Find(i) => {
            let Some(message) = travel::collect(&mut garden, i) else {
                return;
            };
            adventure.focus = None;
            message
        }
        Target::SeasonFind(i) => {
            adventure.focus = None;
            let Some(message) = SeasonProgress::pick(&mut garden, i) else {
                return;
            };
            message
        }
        Target::WatchTrace(site) => {
            let Some(message) = site.inspect(&mut garden) else {
                return;
            };
            adventure.focus = None;
            adventure.tracked = Some(Guide::for_watch(site, &garden));
            message
        }
        Target::HarborDriftwood => {
            adventure.focus = None;
            let Some(message) = harbor::salvage(&mut garden) else {
                return;
            };
            message
        }
        Target::Person(person) => {
            let position = player.position;
            player.teleport(position);
            dialog.talk(person);
            if Onboarding::greet(&mut garden, person) {
                commit(
                    &garden,
                    &path,
                    &mut notice,
                    now,
                    "Bo zeigt dir die Farnwacht · 5 Holz, 1 Stein und 3 Rübensamen".into(),
                    None,
                    None,
                );
            }
            return;
        }
        Target::Village(location) => {
            let item = adventure.offer(&garden);
            match location.work(&mut garden, adventure.crop, item) {
                Ok(message) => message,
                Err(detail) => {
                    persistent = false;
                    detail
                }
            }
        }

        Target::Door => {
            adventure.enter(Area::Home, player, transform, HOME + Vec2::new(0., 2.23));
            persistent = false;
            "Willkommen zu Hause.".into()
        }
        Target::Exit => {
            adventure.enter(Area::Island, player, transform, DOOR + Vec2::Y * 0.38);
            persistent = false;
            "Zurück auf der Insel.".into()
        }
        Target::Portal => {
            adventure.enter(
                Area::Mine(1),
                player,
                transform,
                MINE + Vec2::new(0.42, 0.38),
            );
            garden.deepest = garden.deepest.max(1);
            "Tiefe 1".into()
        }
        Target::Up | Target::Down => {
            let Area::Mine(depth) = adventure.area else {
                return;
            };
            if target == Target::Up && depth == 1 {
                adventure.enter(Area::Island, player, transform, PORTAL + Vec2::Y * 0.52);
                persistent = false;
                "Zurück im Wald.".into()
            } else {
                let next = if target == Target::Up {
                    depth - 1
                } else {
                    depth.saturating_add(1)
                };
                adventure.enter(
                    Area::Mine(next),
                    player,
                    transform,
                    MINE + Vec2::new(0.42, 0.38),
                );
                garden.deepest = garden.deepest.max(next);
                format!("Tiefe {next}")
            }
        }
        Target::Bed => {
            garden.sleep();
            format!("Guten Morgen · Tag {}", garden.day)
        }
        Target::Tree(i) => {
            if !garden.chop_tree(i) {
                return;
            }
            adventure.focus = None;
            let impact = if let Some(node) = forest::node(i) {
                Break {
                    floor: Some(node.height),
                    scale: node.scale,
                    snow_cap: false,
                    ..hit(node.model(Season::at(garden.day)), node.heading, true)
                }
            } else {
                hit("wood_tree", i as f32 * 0.62, true)
            };
            breaks.write(impact);
            format!("+{} Holz", garden.wood_per_tree())
        }
        Target::Rock(i) => {
            if garden.broken.contains(&i) {
                return;
            }
            garden.broken.push(i);
            let quantity = garden.stone_per_rock();
            garden.stone = garden.stone.saturating_add(quantity);
            adventure.focus = None;
            let impact = if let Some(node) = forest::node(i) {
                Break {
                    floor: Some(node.height),
                    scale: node.scale,
                    ..hit(node.model(Season::at(garden.day)), node.heading, false)
                }
            } else {
                hit("resource_rock", i as f32 * 0.67, false)
            };
            breaks.write(impact);
            format!("+{quantity} Stein")
        }
        Target::MineRock(i) => {
            let Area::Mine(depth) = adventure.area else {
                return;
            };
            if !garden.mined.insert((depth, i)) {
                return;
            }
            let ore = Ore::at(depth, i);
            let quantity = ore.mine(&mut garden);
            adventure.focus = None;
            breaks.write(hit(ore.model(), 0., false));
            format!("+{quantity} {} · +1 Stein", ore.name())
        }
        Target::Pier | Target::HarborFishing(_) => match adventure.waiting.take() {
            Some((spot, bite, end)) if spot == target && (bite..=end).contains(&now) => {
                match target {
                    Target::HarborFishing(spot) => harbor::catch(&mut garden, spot).into(),
                    _ => {
                        garden.add_stock(Product::Fish, 1);
                        garden.ledger.record(crate::commissions::Activity::Fish, 1);
                        "+1 Fisch".into()
                    }
                }
            }
            Some((spot, bite, _)) if spot == target && now < bite => {
                persistent = false;
                "Zu früh. Wirf erneut aus.".into()
            }
            _ => {
                let bite = now + 2.3 + (garden.fish % 3) as f32 * 0.5;
                adventure.waiting = Some((target, bite, bite + 2.));
                persistent = false;
                "Angel ausgeworfen.".into()
            }
        },
        Target::Slot(i) => {
            if garden.furniture[i] > 0 {
                garden.furniture[i] = 0;
                "Möbel eingepackt.".into()
            } else if !garden.crafted[adventure.furniture] {
                persistent = false;
                "Erst an der Werkbank herstellen.".into()
            } else if garden
                .furniture
                .contains(&((adventure.furniture + 1) as u8))
            {
                persistent = false;
                "Dieses Möbel steht bereits im Raum.".into()
            } else {
                garden.furniture[i] = (adventure.furniture + 1) as u8;
                format!("{} aufgestellt.", CATALOG[adventure.furniture].name)
            }
        }
    };
    focus.selected = None;
    focus.pending = None;
    if persistent {
        commit(&garden, &path, &mut notice, now, message, None, None);
    } else {
        *notice = Notice {
            message,
            until: now + Notice::DURATION,
            ..default()
        };
    }
}

pub fn commit(
    garden: &Garden,
    path: &SavePath,
    notice: &mut Notice,
    now: f32,
    message: String,
    index: Option<usize>,
    action: Option<crate::garden::Action>,
) {
    let message = match garden.save(&path.0) {
        Ok(()) => message,
        Err(error) => {
            warn!("Speichern fehlgeschlagen: {error}");
            "Fortschritt konnte nicht gespeichert werden.".into()
        }
    };
    *notice = Notice {
        index,
        message,
        action,
        until: now + Notice::DURATION,
    };
}

#[derive(PartialEq, Eq)]
enum Stamp {
    Island {
        felled: Vec<usize>,
        broken: Vec<usize>,
        fruit: [bool; 7],
        animals: [bool; 2],
        housing: [bool; 2],
        gates: [bool; 3],
        bells: [bool; 7],
        finds: [bool; FINDS.len()],
        resonance: [bool; Resonator::ALL.len()],
        season: Season,
        seasonal_finds: [bool; SITES.len()],
        pier: Phase,
        salvage: bool,
        opening: bool,
        workshop: (bool, bool, bool, u8),
        watches: [bool; Watch::ALL.len()],
    },
    Home {
        furniture: Vec<u8>,
        rooms: u8,
    },
    Mine(u32, Vec<u16>),
    Dusk(u32, bool),
}

#[derive(Default)]
pub struct AreaDrawing {
    stamp: Option<Stamp>,
    root: Option<Entity>,
    marker: Option<Entity>,
    bobber: Option<Entity>,
}

pub fn draw(
    mut commands: Commands,
    assets: Res<AssetServer>,
    garden: Res<Garden>,
    adventure: Res<Adventure>,
    time: Res<Time>,
    mut clear: ResMut<ClearColor>,
    mut drawing: Local<AreaDrawing>,
) {
    let stamp = match adventure.area {
        Area::Island | Area::District(_) => Stamp::Island {
            felled: garden
                .felled
                .iter()
                .copied()
                .filter(|&i| i < TREES.len())
                .collect(),
            broken: garden
                .broken
                .iter()
                .copied()
                .filter(|&i| i < ROCKS.len())
                .collect(),
            fruit: std::array::from_fn(|i| garden.farm.fruit_ready(i, garden.day)),
            animals: garden.farm.animals.map(|animal| animal.owned),
            housing: garden.farm.housing,
            gates: Frontier::ALL.map(|gate| gate.unlocked(&garden)),
            bells: Bell::ALL.map(|bell| bell.ready(&garden)),
            finds: std::array::from_fn(|i| FINDS[i].available(&garden, i)),
            resonance: Resonator::ALL.map(|site| site.active(&garden)),
            season: Season::at(garden.day),
            seasonal_finds: std::array::from_fn(|i| garden.seasons.available(i, garden.day)),
            pier: garden.harbor.phase,
            salvage: garden.harbor.salvage_available(),
            opening: garden.harbor.celebrating(garden.day),
            workshop: (
                garden.workshop.restored,
                garden.workshop.forge,
                garden.workshop.loom,
                garden.workshop.house,
            ),
            watches: Watch::ALL.map(|site| site.stage(&garden) >= WatchStage::Guarding),
        },
        Area::Home => Stamp::Home {
            furniture: garden.furniture.clone(),
            rooms: garden.workshop.house,
        },
        Area::Expedition(level) => Stamp::Dusk(level, garden.rpg.cleared >= level),
        Area::Mine(depth) => Stamp::Mine(
            depth,
            garden
                .mined
                .iter()
                .filter(|(d, _)| *d == depth)
                .map(|(_, i)| *i)
                .collect(),
        ),
    };
    if drawing.stamp.as_ref() != Some(&stamp) {
        if let Some(old) = drawing.root.take() {
            commands.entity(old).despawn();
        }
        let root = commands
            .spawn((Transform::default(), Visibility::Inherited))
            .id();
        let load = |name: &str| assets.load::<Scene>(format!("models/{name}.glb#Scene0"));
        let mut spawn = |asset: &str, point: Vec2, y: f32, angle: f32| {
            let child = commands
                .spawn((
                    SceneRoot(load(asset)),
                    Transform::from_xyz(point.x, y, point.y)
                        .with_rotation(Quat::from_rotation_y(angle)),
                ))
                .id();
            commands.entity(root).add_child(child);
            child
        };
        clear.0 = match &stamp {
            Stamp::Island {
                felled,
                broken,
                fruit,
                animals,
                housing,
                gates,
                bells,
                finds,
                resonance,
                season,
                seasonal_finds,
                pier,
                salvage,
                opening,
                workshop: (restored, forge, loom, house),
                watches,
            } => {
                spawn(
                    if *restored {
                        "furniture_bench"
                    } else {
                        "workbench_old"
                    },
                    workshop::BENCH,
                    world::exterior_ground(workshop::BENCH),
                    0.,
                );
                if *forge {
                    spawn(
                        "workshop_forge",
                        workshop::FORGE,
                        world::exterior_ground(workshop::FORGE),
                        0.,
                    );
                }
                if *loom {
                    spawn(
                        "workshop_loom",
                        workshop::LOOM,
                        world::exterior_ground(workshop::LOOM),
                        0.,
                    );
                }
                for site in Watch::ALL {
                    let point = site.point();
                    let ground = world::exterior_ground(point);
                    spawn(site.model(), point, ground, 0.);
                    let green = matches!(site, Watch::Tide | Watch::Fen);
                    let model = match (green, watches[site as usize]) {
                        (true, true) => "watch_green_lit",
                        (true, false) => "watch_green_dark",
                        (false, true) => "watch_warm_lit",
                        (false, false) => "watch_warm_dark",
                    };
                    spawn(model, point + Vec2::new(1.43, 0.72), ground + 0.94, 0.);
                    spawn(
                        "watch_trace",
                        site.trace(),
                        world::exterior_ground(site.trace()),
                        0.,
                    );
                }
                for model in ["cabin_annex_0", "cabin_annex_1"]
                    .into_iter()
                    .take(*house as usize)
                {
                    spawn(model, Vec2::ZERO, 0., 0.);
                }
                let mount = Vec2::new(harbor::PIER_X, harbor::DECK_START);
                spawn(pier.model(), mount, 0., 0.);
                spawn(
                    "pier_board",
                    harbor::BOARD,
                    world::exterior_ground(harbor::BOARD),
                    0.,
                );
                if *salvage {
                    spawn(
                        "pier_salvage",
                        harbor::DRIFTWOOD,
                        world::exterior_ground(harbor::DRIFTWOOD),
                        0.,
                    );
                }
                if *opening {
                    spawn("pier_feast", mount, 0., 0.);
                }
                spawn(
                    "sale_crate",
                    SALE_CHEST,
                    world::exterior_ground(SALE_CHEST),
                    0.,
                );
                spawn("farm_kitchen", KITCHEN, world::exterior_ground(KITCHEN), 0.);
                spawn(
                    "pattern_stall",
                    PATTERN_STALL,
                    world::exterior_ground(PATTERN_STALL),
                    0.,
                );
                for kind in Animal::ALL {
                    let site = kind.site();
                    spawn(
                        if housing[kind as usize] {
                            kind.housing_model()
                        } else {
                            "building_plot"
                        },
                        site,
                        world::exterior_ground(site),
                        0.,
                    );
                }
                for gate in Frontier::ALL {
                    let p = gate.position();
                    spawn(
                        if gates[gate as usize] {
                            "frontier_open"
                        } else {
                            "frontier_gate"
                        },
                        p,
                        world::exterior_ground(p),
                        gate.angle(),
                    );
                }
                for bell in Bell::ALL {
                    let p = bell.position();
                    spawn(
                        if bells[bell as usize] {
                            "waybell_lit"
                        } else {
                            "waybell"
                        },
                        p,
                        world::exterior_ground(p),
                        0.,
                    );
                }
                for site in Resonator::ALL {
                    let p = site.position();
                    spawn(
                        site.model(resonance[site as usize]),
                        p,
                        world::exterior_ground(p),
                        0.,
                    );
                }
                for (i, find) in FINDS.into_iter().enumerate() {
                    if finds[i] {
                        let point = find.position();
                        spawn(
                            find.model(),
                            point,
                            world::exterior_ground(point) + 0.04,
                            i as f32 * 0.31,
                        );
                    }
                }
                spawn(
                    season.table_model(),
                    MEETING_POINT,
                    world::exterior_ground(MEETING_POINT),
                    0.,
                );
                for (i, site) in SITES.into_iter().enumerate() {
                    if seasonal_finds[i] {
                        spawn(
                            site.season.find_model(),
                            site.point,
                            world::exterior_ground(site.point) + 0.035,
                            i as f32 * 0.63,
                        );
                    }
                }
                for (i, point) in TREES.into_iter().enumerate() {
                    let angle = i as f32 * 0.62;
                    spawn(
                        if felled.contains(&i) {
                            "wood_stump"
                        } else {
                            "wood_tree"
                        },
                        point,
                        0.,
                        angle,
                    );
                    if *season == Season::Winter && !felled.contains(&i) {
                        spawn("wood_snow", point, 0., angle);
                    }
                }
                for (i, point) in ROCKS.into_iter().enumerate() {
                    if !broken.contains(&i) {
                        spawn("resource_rock", point, 0., i as f32 * 0.67);
                    }
                }
                for (i, point) in ORCHARD.into_iter().enumerate() {
                    if fruit[i] {
                        spawn(
                            if i < 4 { "apple_fruit" } else { "berry_fruit" },
                            point,
                            0.,
                            0.,
                        );
                    }
                }
                let critters = Animal::ALL
                    .into_iter()
                    .filter(|kind| animals[*kind as usize])
                    .map(|kind| {
                        let base = kind.site() + Vec2::new(-0.17, 0.66);
                        let entity = spawn(
                            if kind == Animal::Hen {
                                "farm_hen"
                            } else {
                                "farm_sheep"
                            },
                            base,
                            0.,
                            0.,
                        );
                        (entity, Critter { origin: base, kind })
                    })
                    .collect::<Vec<_>>();
                // Release the spawn closure before attaching an animated component.
                for (entity, critter) in critters {
                    commands.entity(entity).insert(critter);
                }
                Color::srgb_u8(132, 204, 201)
            }
            Stamp::Home { furniture, rooms } => {
                spawn("interior", HOME, 0., 0.);
                spawn(
                    "bedroom_bed",
                    adventure.position(Target::Bed, &garden),
                    0.,
                    0.,
                );
                if *rooms == 0 {
                    spawn("interior_annex_wall", HOME, 0., 0.);
                }
                for model in ["interior_annex_0", "interior_annex_1"]
                    .into_iter()
                    .take(*rooms as usize)
                {
                    spawn(model, HOME, 0., 0.);
                }
                spawn("card_binder", BINDER, 0., 0.);
                for (i, kind) in furniture.iter().take(garden.workshop.slots()).enumerate() {
                    spawn("selection", HOME + SLOTS[i], 0.055, 0.);
                    if let Some(name) = kind.checked_sub(1).and_then(|v| CATALOG.get(v as usize)) {
                        spawn(name.model, HOME + SLOTS[i], 0.08, 0.);
                    }
                }
                spawn(
                    "selection",
                    adventure.position(Target::Exit, &garden),
                    0.05,
                    0.,
                );
                Color::srgb_u8(184, 139, 115)
            }
            Stamp::Mine(depth, removed) => {
                for &cell in &adventure.layout.cells {
                    let p = cell_position(cell);
                    spawn(
                        if *depth >= 3 {
                            "mine_deep_floor"
                        } else {
                            "mine_floor"
                        },
                        p,
                        0.,
                        f32::from(cell % 4) * std::f32::consts::FRAC_PI_2,
                    );
                    for (offset, angle) in adventure.layout.walls(cell) {
                        spawn(
                            if *depth >= 3 {
                                "mine_deep_wall"
                            } else {
                                "mine_wall"
                            },
                            p + offset,
                            -0.05,
                            angle,
                        );
                    }
                }
                for &cell in &adventure.layout.cells {
                    if cell % 4 == 1 && cell < 72 && adventure.layout.cells.contains(&(cell + 9)) {
                        spawn(
                            "mine_support",
                            cell_position(cell) + Vec2::Y * TILE * 0.48,
                            0.,
                            0.,
                        );
                    } else if cell % 5 == 2
                        && cell % 9 < 8
                        && adventure.layout.cells.contains(&(cell + 1))
                    {
                        spawn(
                            "mine_support",
                            cell_position(cell) + Vec2::X * TILE * 0.48,
                            0.,
                            std::f32::consts::FRAC_PI_2,
                        );
                    }
                }
                for &cell in &adventure.layout.ores {
                    if !removed.contains(&cell) {
                        spawn(
                            Ore::at(*depth, cell).model(),
                            adventure.position(Target::MineRock(cell), &garden),
                            0.,
                            f32::from(cell) * 0.43,
                        );
                    }
                }
                spawn(
                    "mine_ladder",
                    adventure.position(Target::Up, &garden),
                    0.,
                    0.,
                );
                spawn(
                    "mine_ladder",
                    adventure.position(Target::Down, &garden),
                    0.,
                    0.25,
                );
                if *depth >= 3 {
                    Color::srgb_u8(36, 39, 60)
                } else {
                    Color::srgb_u8(57, 42, 62)
                }
            }
            Stamp::Dusk(level, won) => {
                let twist = std::f32::consts::FRAC_PI_2;
                let chapter = Chapter::at(*level);
                for &cell in &adventure.layout.cells {
                    let p = dusk_cell(cell);
                    spawn(
                        chapter.floor((u32::from(cell) + level) as usize),
                        p,
                        0.,
                        f32::from(cell % 4) * twist,
                    );
                    for (edge, (offset, angle)) in adventure.layout.walls(cell).enumerate() {
                        let decoration = (usize::from(cell) * 3 + edge + *level as usize) % 6;
                        spawn(
                            match decoration {
                                0 => "dusk_lantern_wall",
                                1 => chapter.wall_feature(),
                                _ => "dusk_wall",
                            },
                            p + offset,
                            -0.02,
                            angle,
                        );
                    }
                }
                spawn(
                    "dusk_arch",
                    adventure.position(Target::DuskExit, &garden),
                    0.,
                    0.,
                );
                spawn(
                    if *won { "dusk_open" } else { "dusk_seal" },
                    adventure.position(Target::DuskDown, &garden),
                    0.,
                    0.,
                );
                if !*won {
                    spawn(
                        "dusk_rosette",
                        adventure.position(Target::DuskFoe, &garden),
                        0.,
                        0.,
                    );
                    let foe = spawn(
                        Enemy::at(*level).model(),
                        adventure.position(Target::DuskFoe, &garden),
                        0.,
                        -0.26,
                    );
                    commands
                        .entity(foe)
                        .insert(crate::battle_ui::GuardianMotion(0.));
                } else if level % CAMP_INTERVAL == 0 {
                    spawn(
                        "dusk_camp",
                        adventure.position(Target::DuskCamp, &garden),
                        0.,
                        0.,
                    );
                }
                match chapter {
                    Chapter::Moss => Color::srgb_u8(41, 45, 53),
                    Chapter::Clay => Color::srgb_u8(54, 40, 52),
                    Chapter::Mist => Color::srgb_u8(40, 49, 66),
                    Chapter::Root => Color::srgb_u8(46, 49, 42),
                    Chapter::Bell => Color::srgb_u8(55, 47, 62),
                }
            }
        };
        drawing.root = Some(root);
        drawing.stamp = Some(stamp);
    }
    if let Some(target) = adventure.focus {
        let p = adventure.position(target, &garden);
        let floor = if adventure.area.outdoors() {
            world::surface_height(p, &garden)
        } else {
            0.
        };
        let transform = Transform::from_xyz(p.x, floor + 0.07, p.y);
        if let Some(entity) = drawing.marker {
            commands.entity(entity).insert(transform);
        } else {
            drawing.marker = Some(
                commands
                    .spawn((
                        SceneRoot(assets.load(format!("models/{MARKER}.glb#Scene0"))),
                        transform,
                    ))
                    .id(),
            );
        }
    } else if let Some(entity) = drawing.marker.take() {
        commands.entity(entity).despawn();
    }
    if let Some((spot, bite, _)) = adventure
        .waiting
        .filter(|(_, _, end)| time.elapsed_secs() <= *end)
    {
        let now = time.elapsed_secs();
        let y = -0.755 + (now * 8.).sin() * if now >= bite { 0.08 } else { 0.021 };
        let p = match spot {
            Target::HarborFishing(fishing) => fishing.bobber(),
            _ => Vec2::new(6.52, 19.13),
        };
        let transform = Transform::from_xyz(p.x, y, p.y);
        if let Some(entity) = drawing.bobber {
            commands.entity(entity).insert(transform);
        } else {
            drawing.bobber = Some(
                commands
                    .spawn((
                        SceneRoot(assets.load("models/bobber.glb#Scene0")),
                        transform,
                    ))
                    .id(),
            );
        }
    } else if let Some(entity) = drawing.bobber.take() {
        commands.entity(entity).despawn();
    }
}

#[derive(Component)]
pub struct Critter {
    origin: Vec2,
    kind: Animal,
}

pub fn animate_animals(time: Res<Time>, mut animals: Query<(&Critter, &mut Transform)>) {
    for (animal, mut pose) in &mut animals {
        let speed = if animal.kind == Animal::Hen {
            1.15
        } else {
            0.69
        };
        let elapsed = time.elapsed_secs() * speed;
        let x = elapsed.sin() * 0.53;
        let z = (elapsed * 0.77).cos() * 0.38;
        let bob = (elapsed * 6.).sin().abs() * 0.018;
        pose.translation = Vec3::new(animal.origin.x + x, bob, animal.origin.y + z);
        pose.rotation =
            Quat::from_rotation_y((elapsed.cos() * 0.53).atan2(-(elapsed * 0.77).sin() * 0.29));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_five_watches_and_survey_marks_have_normal_walking_routes() {
        let mut garden = Garden::default();
        garden.regions = serde_json::from_value(serde_json::json!({"gates":[2,2,2]})).unwrap();
        let a = Adventure::default();
        for (site, district) in Watch::ALL.into_iter().zip([
            District::Forest,
            District::Beach,
            District::Copper,
            District::Wind,
            District::Moor,
        ]) {
            let mut player = Gardener::new(district.waypoint());
            assert!(
                player.navigate(a.reach(Target::Watch(site), &garden), &a, &garden),
                "Wacht {site:?}"
            );
            assert!(
                player.navigate(a.reach(Target::WatchTrace(site), &garden), &a, &garden),
                "Zeichen {site:?}"
            );
        }
    }

    #[test]
    fn a_tap_on_a_tall_plant_selects_its_own_bed() {
        let mut garden = Garden::default();
        let index = 1;
        garden.plots[index] = crate::garden::Plot::Sown {
            crop: Crop::Sunflower,
            growth: Crop::Sunflower.days(),
            wet: true,
        };
        let center = world::PLOTS[index];
        let tip = Vec3::new(center.x, 1.17, center.y);
        let offset = world::camera_offset(Area::Island);
        let ray = Ray3d::new(tip + offset, Dir3::new(-offset).unwrap());
        assert_eq!(hit_plot(ray, &garden).map(|(index, _)| index), Some(index));
    }

    #[test]
    fn seasonal_targets_are_reachable_and_the_dorftisch_blocks_movement() {
        let mut garden = Garden::default();
        let adventure = Adventure::default();
        assert!(!adventure.walkable(MEETING_POINT, &garden));
        assert!(adventure.can_reach(Target::SeasonMeeting, MEETING_POINT + Vec2::Y, &garden));
        for (i, site) in SITES.into_iter().enumerate() {
            garden.day = 1 + site.season as u32 * Season::LENGTH;
            assert!(
                adventure
                    .targets(&garden, site.point)
                    .contains(&Target::SeasonFind(i))
            );
            assert!(adventure.walkable(site.point, &garden), "Fundort {}", i + 1);
        }
        for npc in [Npc::Lina, Npc::Miro, Npc::Fiete, Npc::Juna] {
            assert!(!adventure.walkable(npc.position(&garden), &garden));
            assert!((0..8).any(|i| {
                let point = npc.position(&garden)
                    + Vec2::from_angle(i as f32 * std::f32::consts::FRAC_PI_4);
                adventure.walkable(point, &garden)
                    && adventure.can_reach(Target::Person(npc), point, &garden)
            }));
        }
    }

    #[test]
    fn resources_block_the_player_until_collected_and_remain_reachable() {
        let mut garden = Garden::default();
        let mut adventure = Adventure::default();
        let tree = Target::Tree(10);
        let rock = Target::Rock(7);
        for target in [tree, rock] {
            let center = adventure.position(target, &garden);
            assert!(!adventure.walkable(center, &garden));
            assert!(adventure.can_reach(target, center + Vec2::Y, &garden));
        }
        garden.felled.push(10);
        garden.broken.push(7);
        assert!(!adventure.walkable(adventure.position(tree, &garden), &garden));
        assert!(adventure.walkable(adventure.position(rock, &garden), &garden));

        adventure.area = Area::Mine(1);
        adventure.layout = MineFloor::generate(1);
        let cell = adventure.layout.ores[0];
        let point = adventure.position(Target::MineRock(cell), &garden);
        assert!(!adventure.walkable(point, &garden));
        garden.mined.insert((1, cell));
        assert!(adventure.walkable(point, &garden));
    }

    #[test]
    fn furniture_collides_and_the_entry_stays_clear() {
        let mut garden = Garden::default();
        let adventure = Adventure {
            area: Area::Home,
            ..default()
        };
        assert!(!adventure.walkable(adventure.position(Target::Bed, &garden), &garden));

        let slot = adventure.position(Target::Slot(0), &garden);
        assert!(adventure.walkable(slot, &garden));
        garden.furniture[0] = 1;
        assert!(!adventure.walkable(slot, &garden));
        assert!(adventure.walkable(HOME + Vec2::new(0., 2.23), &garden));
        assert!(adventure.can_reach(Target::Exit, HOME + Vec2::new(0., 2.23), &garden));
    }

    #[test]
    fn selecting_the_roof_picks_the_door() {
        let garden = Garden::default();
        let camera = Vec3::new(-0.62, 0.89, 3.47) + world::camera_offset(Area::Island);
        let roof = Vec3::new(-3.0, 4.36, -2.44);
        let ray = Ray3d::new(camera, Dir3::new(roof - camera).unwrap());
        assert!(hit_volume(ray, Target::Door, Vec3::ZERO, &garden).is_some());
        let outside = Vec3::new(3.0, 0.0, 1.4);
        let ray = Ray3d::new(camera, Dir3::new(outside - camera).unwrap());
        assert!(hit_volume(ray, Target::Door, Vec3::ZERO, &garden).is_none());
    }

    #[test]
    fn layers_have_reachable_exit_and_distinct_deterministic_layouts() {
        let first = MineFloor::generate(1);
        assert_eq!(first.cells, MineFloor::generate(1).cells);
        assert_ne!(first.cells, MineFloor::generate(2).cells);
        for depth in 1..80 {
            let mine = MineFloor::generate(depth);
            let mut seen = vec![START];
            let mut i = 0;
            while let Some(&cell) = seen.get(i) {
                for next in neighbors(cell) {
                    if mine.cells.contains(&next) && !seen.contains(&next) {
                        seen.push(next);
                    }
                }
                i += 1;
            }
            assert_eq!(seen.len(), mine.cells.len());
            assert!(seen.contains(&mine.down) && mine.down != START);
            assert_eq!(mine.ores.len(), 7);
        }
    }

    #[test]
    fn gate_binder_and_all_dusk_floors_have_a_playable_route() {
        let mut garden = Garden::default();
        let mut adventure = Adventure::default();
        let mut player = Gardener::new(DUSK_ARRIVAL);
        let mut transform = Transform::default();
        assert!(world::walkable(DUSK_ARRIVAL));
        assert!(world::walkable(DUSK_GATE));
        adventure.enter(
            Area::Home,
            &mut player,
            &mut transform,
            HOME + Vec2::new(0., 2.23),
        );
        assert!(player.navigate(
            adventure.reach(Target::Binder, &garden),
            &adventure,
            &garden
        ));
        let mut layouts = std::collections::HashSet::new();
        for level in 1..=LAST_LEVEL {
            garden.rpg.cleared = level - 1;
            adventure.enter_duel(level, false, &mut garden, &mut player, &mut transform);
            let entrance = player.position;
            assert!(
                adventure.walkable(entrance, &garden),
                "first tile on {level}"
            );
            let mut cells = adventure.layout.cells.clone();
            let old = MineFloor::generate(900 + level);
            assert!(old.cells.iter().all(|cell| cells.contains(cell)));
            assert_eq!(
                (old.down, old.encounter),
                (adventure.layout.down, adventure.layout.encounter)
            );
            cells.sort_unstable();
            assert!(layouts.insert(cells), "duplicate layout at {level}");
            assert!(
                player.navigate(
                    adventure.reach(Target::DuskFoe, &garden),
                    &adventure,
                    &garden
                ),
                "foe on {level}"
            );
            garden.rpg.cleared = level;
            player.teleport(entrance);
            assert!(
                player.navigate(
                    adventure.reach(Target::DuskDown, &garden),
                    &adventure,
                    &garden
                ),
                "stairs on {level}"
            );
            if level % CAMP_INTERVAL == 0 {
                garden.rpg.hp = 1;
                let (area, at) = adventure.escape_duel(&mut garden, &mut player, &mut transform);
                assert_eq!(area, Area::Expedition(level));
                assert_eq!(garden.rpg.hp, garden.rpg.max_hp());
                assert!(adventure.walkable(at, &garden), "camp on {level}");
                assert!(
                    player.navigate(
                        adventure.reach(Target::DuskExit, &garden),
                        &adventure,
                        &garden
                    ),
                    "return from camp {level}"
                );
            }
        }
    }

    #[test]
    fn story_landmarks_are_reachable_from_their_regions() {
        let mut garden = Garden::default();
        garden.regions = serde_json::from_str(r#"{"gates":[2,2,2]}"#).unwrap();
        let adventure = Adventure::default();
        for site in Resonator::ALL {
            let start = site
                .frontier()
                .map_or(District::Town, Frontier::district)
                .waypoint();
            let mut player = Gardener::new(start);
            assert!(
                player.navigate(
                    adventure.reach(Target::Resonator(site), &garden),
                    &adventure,
                    &garden
                ),
                "{site:?}"
            );
        }
    }
}
