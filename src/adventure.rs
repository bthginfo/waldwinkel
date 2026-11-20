use crate::{
    Controls, GardenFocus, Notice,
    garden::{Crop, Garden, SavePath},
    homestead::{Animal, ORCHARD, Ore, Product, Village},
    session::Checkpoint,
    town::{Dialog, Npc},
    travel::{self, District, FINDS},
    world::{self, Gardener, IslandCamera},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const HOME: Vec2 = Vec2::new(220., 0.);
const MINE: Vec2 = Vec2::new(400., 0.);
pub const DOOR: Vec2 = Vec2::new(-2.08, 1.32);
pub const PORTAL: Vec2 = Vec2::new(9., -8.90);
pub const PIER: Vec2 = Vec2::new(5.8, 17.25);
const MARKER: &str = "selection";
const TILE: f32 = 2.4;
const GRID: i16 = 9;
const START: u16 = 40;

const TREES: [Vec2; 10] = [
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
];
const ROCKS: [Vec2; 7] = [
    Vec2::new(-11., 4.),
    Vec2::new(-12., -5.),
    Vec2::new(-17., -1.),
    Vec2::new(10., -6.),
    Vec2::new(12., 12.),
    Vec2::new(10., 9.),
    Vec2::new(-4., 10.),
];
const SLOTS: [Vec2; 6] = [
    Vec2::new(1.59, 0.77),
    Vec2::new(2.45, -1.61),
    Vec2::new(-0.02, -1.93),
    Vec2::new(-1.66, 1.49),
    Vec2::new(1.85, 1.98),
    Vec2::new(-2.62, 0.38),
];
const FURNITURE: [&str; 3] = ["Tisch", "Regal", "Ofen"];
const FURNITURE_MODELS: [&str; 3] = ["furniture_table", "furniture_shelf", "furniture_stove"];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum Area {
    #[default]
    Island,
    Home,
    Mine(u32),
    District(District),
}

impl Area {
    pub fn origin(self) -> Vec2 {
        match self {
            Self::Island => Vec2::ZERO,
            Self::Home => HOME,
            Self::Mine(_) => MINE,
            Self::District(district) => district.origin(),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Island => "Waldwinkel",
            Self::Home => "Holzhütte",
            Self::Mine(_) => "Mine",
            Self::District(district) => district.name(),
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
    Tree(usize),
    Rock(usize),
    MineRock(u16),
    Down,
    Up,
    Bed,
    Exit,
    Slot(usize),
    Bench,
    Village(Village),
    Travel(District),
    Find(usize),
    Person(Npc),
}

impl Target {
    fn name(self) -> &'static str {
        match self {
            Self::Door | Self::Exit => "Holzhütte",
            Self::Portal => "Minenstollen",
            Self::Pier => "Angelsteg",
            Self::Tree(_) => "Kiefer",
            Self::Rock(_) => "Steinblock",
            Self::MineRock(_) => "Kupferader",
            Self::Down | Self::Up => "Minenleiter",
            Self::Bed => "Bett",
            Self::Slot(_) => "Möbelplatz",
            Self::Bench => "Werkbank",
            Self::Village(target) => target.name(),
            Self::Travel(district) => district.name(),
            Self::Find(i) => FINDS[i].name(),
            Self::Person(person) => person.name(),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Door => "Eintreten",
            Self::Exit => "Hinausgehen",
            Self::Portal => "Mine betreten",
            Self::Pier => "Auswerfen",
            Self::Tree(_) => "Baum fällen",
            Self::Rock(_) => "Stein abbauen",
            Self::MineRock(_) => "Kupfer abbauen",
            Self::Down => "Tiefer steigen",
            Self::Up => "Aufsteigen",
            Self::Bed => "Schlafen",
            Self::Slot(_) => "Einrichten",
            Self::Bench => "Herstellen",
            Self::Village(_) => "Benutzen",
            Self::Travel(_) => "Angekommen",
            Self::Find(_) => "Sammeln",
            Self::Person(_) => "Ansprechen",
        }
    }

    fn height(self) -> f32 {
        match self {
            Self::Door => 0.86,
            Self::Portal => 0.95,
            Self::Tree(_) => 1.50,
            Self::Down | Self::Up => 0.65,
            Self::Village(target) => target.height(),
            Self::Travel(_) => 0.2,
            Self::Find(i) => FINDS[i].height(),
            Self::Person(_) => 1.02,
            _ => 0.23,
        }
    }
}

pub const LANDMARKS: [(&str, Target); 7] = [
    ("Hafenmarkt", Target::Village(Village::Board)),
    ("Tierweide", Target::Village(Village::Feed)),
    ("Minenstollen", Target::Portal),
    ("Hof", Target::Door),
    ("Farnwald", Target::Travel(District::Forest)),
    ("Muschelbucht", Target::Travel(District::Beach)),
    ("Eichenhafen", Target::Travel(District::Town)),
];

pub fn landmark_center(
    camera: &Camera,
    view: &GlobalTransform,
    p: Vec2,
    scale: f32,
) -> Option<Vec2> {
    camera
        .world_to_viewport(view, Vec3::new(p.x, 0.85, p.y))
        .ok()
        .map(|at| at + Vec2::Y * 23.0 * scale)
}

#[derive(Resource, Default)]
pub struct Adventure {
    pub area: Area,
    pub focus: Option<Target>,
    pub crop: Crop,
    pub furniture: usize,
    pub inventory: bool,
    pub journal: bool,
    pub map: bool,
    pub zoom: f32,
    waiting: Option<(f32, f32)>,
    pub layout: MineFloor,
    sale: usize,
}

#[derive(Clone, Debug, Default)]
pub struct MineFloor {
    cells: Vec<u16>,
    ores: Vec<u16>,
    down: u16,
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
        Self { cells, ores, down }
    }

    fn contains(&self, point: Vec2) -> bool {
        let cell = (point - MINE) / TILE + Vec2::splat(4.);
        let center = cell.round();
        if center.min_element() < 0. || center.max_element() >= f32::from(GRID) {
            return false;
        }
        self.cells
            .contains(&((center.y as u16) * GRID as u16 + center.x as u16))
    }
}

impl Adventure {
    pub fn offer(&self, garden: &Garden) -> Option<Product> {
        Product::at(self.sale, garden).or_else(|| Product::at(0, garden))
    }

    pub fn furniture_name(&self) -> &'static str {
        FURNITURE[self.furniture]
    }
    pub fn walkable(&self, point: Vec2) -> bool {
        match self.area {
            Area::Island | Area::District(_) => world::walkable(point),
            Area::Home => {
                let p = point - HOME;
                p.x.abs() < 3.35 && p.y > -2.65 && p.y < 2.93
            }
            Area::Mine(_) => self.layout.contains(point),
        }
    }

    pub fn position(&self, target: Target) -> Vec2 {
        match target {
            Target::Door => DOOR,
            Target::Portal => PORTAL,
            Target::Pier => PIER,
            Target::Tree(i) => TREES[i],
            Target::Rock(i) => ROCKS[i],
            Target::Bed => HOME + Vec2::new(-2.03, -1.74),
            Target::Exit => HOME + Vec2::new(0., 2.73),
            Target::Bench => HOME + Vec2::new(2.90, -0.44),
            Target::Slot(i) => HOME + SLOTS[i],
            Target::Up => cell_position(START) - Vec2::new(0.55, 0.57),
            Target::Down => cell_position(self.layout.down),
            Target::MineRock(cell) => cell_position(cell) + Vec2::new(0.41, -0.34),
            Target::Village(location) => location.position(),
            Target::Travel(district) => district.waypoint(),
            Target::Find(i) => FINDS[i].position(),
            Target::Person(person) => person.position(),
        }
    }

    fn targets(&self, garden: &Garden) -> Vec<Target> {
        match self.area {
            Area::Island | Area::District(_) => [Target::Door, Target::Portal, Target::Pier]
                .into_iter()
                .chain(District::ALL.into_iter().map(Target::Travel))
                .chain(Village::all().map(Target::Village))
                .chain(Npc::ALL.into_iter().map(Target::Person))
                .chain(FINDS.iter().enumerate().filter_map(|(i, _)| {
                    garden
                        .town
                        .find_ready(i, garden.day)
                        .then_some(Target::Find(i))
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
                .collect(),
            Area::Home => [Target::Bed, Target::Exit, Target::Bench]
                .into_iter()
                .chain((0..SLOTS.len()).map(Target::Slot))
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
        }
    }

    pub fn context(
        &self,
        garden: &Garden,
        player: Vec2,
        now: f32,
    ) -> (String, String, &'static str, bool) {
        if let Some(target) = self.focus {
            let near = player.distance(self.position(target)) < 1.58;
            if let Target::Village(location) = target {
                let (detail, action, enabled) =
                    location.describe(garden, self.crop, self.offer(garden));
                return if near {
                    (target.name().into(), detail, action, enabled)
                } else {
                    (
                        target.name().into(),
                        "Geh näher heran.".into(),
                        "Hingehen",
                        true,
                    )
                };
            }
            if let (Target::MineRock(cell), Area::Mine(depth)) = (target, self.area) {
                let ore = Ore::at(depth, cell);
                let amount = if garden.farm.pickaxe { 2 } else { 1 };
                return if near {
                    (
                        ore.title().into(),
                        format!("+{amount} {} · +1 Stein", ore.name()),
                        "Abbauen",
                        true,
                    )
                } else {
                    (
                        ore.title().into(),
                        "Geh näher heran.".into(),
                        "Hingehen",
                        true,
                    )
                };
            }
            let (hint, action) = match target {
                Target::Slot(i) => {
                    let old = garden.furniture[i]
                        .checked_sub(1)
                        .and_then(|i| FURNITURE.get(i as usize));
                    (
                        format!(
                            "{} · {}",
                            old.copied().unwrap_or("Freier Platz"),
                            FURNITURE[self.furniture]
                        ),
                        if old.is_some() {
                            "Einpacken"
                        } else {
                            "Aufstellen"
                        },
                    )
                }
                Target::Bench => (
                    format!(
                        "{} · Holz {} · Stein {} · Kupfer {}",
                        FURNITURE[self.furniture], garden.wood, garden.stone, garden.copper
                    ),
                    "Herstellen",
                ),
                Target::Pier => match self.waiting {
                    Some((bite, end)) if now >= bite && now <= end => {
                        ("Ein Fisch beißt an!".into(), "Einholen!")
                    }
                    Some((_, end)) if now <= end => ("Warte auf den Biss …".into(), "Einholen"),
                    _ => ("Beim Biss die rechte Taste antippen.".into(), "Auswerfen"),
                },
                Target::Bed => (
                    "Ein neuer Tag beginnt. Gegossene Pflanzen wachsen.".into(),
                    "Schlafen",
                ),
                Target::Tree(_) => ("Die Kiefer liefert 3 Holz.".into(), "Baum fällen"),
                Target::Rock(_) => ("Der Block liefert 2 Stein.".into(), "Stein abbauen"),
                Target::Travel(district) => (district.hint().into(), target.label()),
                Target::Person(person) => (person.role().into(), target.label()),
                Target::Find(i) => (
                    format!("+{} {}", FINDS[i].quantity, FINDS[i].name()),
                    target.label(),
                ),
                _ => (
                    "Tippe rechts, wenn du nah genug bist.".into(),
                    target.label(),
                ),
            };
            return (
                target.name().into(),
                if near {
                    hint
                } else {
                    "Geh näher heran.".into()
                },
                if near { action } else { "Hingehen" },
                !(near && matches!(target, Target::Travel(_))),
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
                "Tippe auf Bett, Werkbank oder Möbelplatz.".into(),
                "Ort wählen",
                true,
            ),
            Area::Mine(depth) => (
                format!("Mine · Tiefe {depth}"),
                "Tippe auf Erz oder eine Leiter.".into(),
                "Ort wählen",
                true,
            ),
        }
    }

    fn enter(&mut self, area: Area, player: &mut Gardener, transform: &mut Transform, from: Vec2) {
        self.area = area;
        self.focus = None;
        self.waiting = None;
        self.map = false;
        self.inventory = false;
        self.journal = false;
        if let Area::Mine(depth) = area {
            self.layout = MineFloor::generate(depth);
        }
        player.teleport(from);
        transform.translation = Vec3::new(from.x, 0., from.y);
    }

    pub fn restore(
        &mut self,
        checkpoint: Option<Checkpoint>,
        player: &mut Gardener,
        transform: &mut Transform,
    ) {
        *self = Self::default();
        if let Some(checkpoint) = checkpoint.filter(|point| point.area != Area::Mine(0)) {
            let position = Vec2::from_array(checkpoint.position);
            self.enter(checkpoint.area, player, transform, position);
            if position.is_finite() {
                if self.walkable(position) {
                    return;
                }
                if self.area.outdoors() {
                    for radius in [0.55, 1.0, 1.8, 2.8] {
                        if let Some(nearby) = (0..16)
                            .map(|i| {
                                let angle = i as f32 * std::f32::consts::TAU / 16.;
                                position + Vec2::from_angle(angle) * radius
                            })
                            .find(|&p| self.walkable(p))
                        {
                            player.teleport(nearby);
                            transform.translation =
                                Vec3::new(nearby.x, world::exterior_ground(nearby), nearby.y);
                            return;
                        }
                    }
                }
            }
        }
        self.enter(Area::Island, player, transform, world::SPAWN);
    }
}

pub fn select(
    controls: Res<Controls>,
    layout: Res<crate::ui::HudLayout>,
    mut adventure: ResMut<Adventure>,
    garden: Res<Garden>,
    mut focus: ResMut<GardenFocus>,
    camera: Single<(&Camera, &GlobalTransform), With<IslandCamera>>,
    mut player: Single<&mut Gardener>,
) {
    let (camera, view) = *camera;
    let Some(at) = controls.pick_at else {
        if controls.cycle && adventure.area.outdoors() {
            adventure.focus = None;
        }
        return;
    };
    let mut best = (f32::MAX, None);
    for target in adventure.targets(&garden) {
        let point = adventure.position(target);
        let y = if adventure.area.outdoors() {
            world::exterior_ground(point)
        } else {
            0.
        };
        let world = Vec3::new(point.x, y + target.height(), point.y);
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
    let landmark = if adventure.map && adventure.area.outdoors() {
        LANDMARKS
            .into_iter()
            .filter_map(|(_, target)| {
                let center =
                    landmark_center(camera, view, adventure.position(target), layout.scale)?;
                let delta = (at - center).abs() / layout.scale;
                (delta.x <= 53. && delta.y <= 12.).then_some((target, delta.length_squared()))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(target, _)| target)
    } else {
        None
    };
    // A board frame has priority over the scenery behind it.
    if landmark.is_none() && focus.selected.is_some() && adventure.area == Area::Island {
        adventure.focus = None;
        return;
    }
    adventure.focus = landmark.or(best.1);
    adventure.sale = 0;
    let destination = adventure
        .focus
        .map(|target| adventure.position(target))
        .or_else(|| {
            // The same controls can select a place on the ground.
            if !adventure.area.outdoors() {
                return None;
            }
            let ray = camera.viewport_to_world(view, at).ok()?;
            let down = Dir3::NEG_Y;
            let hit = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(down))?;
            let point = ray.get_point(hit).xz();
            world::walkable(point).then_some(point)
        });
    if let Some(destination) = destination {
        focus.selected = None;
        focus.approach = None;
        adventure.map = false;
        player.navigate(destination, &adventure);
    }
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
) {
    if controls.zoom != 0. {
        adventure.map = false;
        adventure.zoom = (adventure.zoom + controls.zoom).clamp(-0.7, 1.25);
    }
    if controls.map {
        adventure.map = !adventure.map;
    }
    if controls.inventory {
        adventure.inventory = !adventure.inventory;
        adventure.journal = false;
    }
    if controls.choice {
        if adventure.area == Area::Home {
            adventure.furniture = (adventure.furniture + 1) % FURNITURE.len();
        } else if adventure.focus == Some(Target::Village(Village::Sell)) {
            let next = adventure.sale + 1;
            adventure.sale = if Product::at(next, &garden).is_some() {
                next
            } else {
                0
            };
        } else {
            adventure.crop = Crop::ALL[(adventure.crop as usize + 1) % Crop::ALL.len()];
        }
    }
    let now = time.elapsed_secs();
    if controls.task {
        if adventure.area.outdoors() && District::at(player.0.position).is_some() {
            adventure.journal = !adventure.journal;
            adventure.inventory = false;
        } else if garden.claim() {
            commit(
                &garden,
                &path,
                &mut notice,
                now,
                "Aufgabe erledigt · +5 Münzen und Saatgut".into(),
                None,
                None,
            );
        } else {
            *notice = Notice {
                message: garden.task().0.into(),
                until: now + 3.7,
                ..default()
            };
        }
    }
    if !controls.act || focus.selected.is_some() {
        return;
    }
    let (player, transform) = &mut *player;
    let Some(target) = adventure.focus else {
        adventure.focus = adventure.targets(&garden).into_iter().min_by(|&a, &b| {
            player
                .position
                .distance_squared(adventure.position(a))
                .total_cmp(&player.position.distance_squared(adventure.position(b)))
        });
        if let Some(target) = adventure.focus {
            player.navigate(adventure.position(target), &adventure);
        }
        return;
    };
    if player.position.distance(adventure.position(target)) >= 1.58 {
        adventure.map = false;
        player.navigate(adventure.position(target), &adventure);
        return;
    }
    let mut persistent = true;
    let message = match target {
        Target::Travel(_) => return,
        Target::Find(i) => {
            let Some(message) = travel::collect(&mut garden, i) else {
                return;
            };
            adventure.focus = None;
            message
        }
        Target::Person(person) => {
            let position = player.position;
            player.teleport(position);
            adventure.journal = false;
            dialog.talk(person);
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
            if garden.felled.contains(&i) {
                return;
            }
            garden.felled.push(i);
            garden.wood = garden.wood.saturating_add(3);
            adventure.focus = None;
            "+3 Holz".into()
        }
        Target::Rock(i) => {
            if garden.broken.contains(&i) {
                return;
            }
            garden.broken.push(i);
            garden.stone = garden.stone.saturating_add(2);
            adventure.focus = None;
            "+2 Stein".into()
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
            format!("+{quantity} {} · +1 Stein", ore.name())
        }
        Target::Pier => match adventure.waiting.take() {
            Some((bite, end)) if (bite..=end).contains(&now) => {
                garden.fish = garden.fish.saturating_add(1);
                "+1 Fisch".into()
            }
            Some((bite, _)) if now < bite => {
                persistent = false;
                "Zu früh. Wirf erneut aus.".into()
            }
            _ => {
                let bite = now + 2.3 + (garden.fish % 3) as f32 * 0.5;
                adventure.waiting = Some((bite, bite + 2.));
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
                format!("{} aufgestellt.", FURNITURE[adventure.furniture])
            }
        }
        Target::Bench => {
            let item = adventure.furniture;
            let (wood, stone, copper) = [(0, 0, 0), (3, 0, 0), (1, 3, 1)][item];
            if garden.crafted[item] {
                persistent = false;
                "Schon vorhanden. Wähle einen Platz.".into()
            } else if garden.wood < wood || garden.stone < stone || garden.copper < copper {
                persistent = false;
                format!("Benötigt · {wood} Holz / {stone} Stein / {copper} Kupfer")
            } else {
                garden.wood -= wood;
                garden.stone -= stone;
                garden.copper -= copper;
                garden.crafted[item] = true;
                format!("{} hergestellt.", FURNITURE[item])
            }
        }
    };
    focus.selected = None;
    focus.approach = None;
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
        finds: [bool; FINDS.len()],
    },
    Home([u8; 6]),
    Mine(u32, Vec<u16>),
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
            felled: garden.felled.clone(),
            broken: garden.broken.clone(),
            fruit: std::array::from_fn(|i| garden.farm.fruit_ready(i, garden.day)),
            animals: garden.farm.animals.map(|animal| animal.owned),
            finds: std::array::from_fn(|i| garden.town.find_ready(i, garden.day)),
        },
        Area::Home => Stamp::Home(garden.furniture),
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
                finds,
            } => {
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
                for (i, point) in TREES.into_iter().enumerate() {
                    spawn(
                        if felled.contains(&i) {
                            "wood_stump"
                        } else {
                            "wood_tree"
                        },
                        point,
                        0.,
                        i as f32 * 0.62,
                    );
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
                        let base = Village::Animal(kind).position()
                            + if kind == Animal::Hen {
                                Vec2::new(-0.87, 0.45)
                            } else {
                                Vec2::new(-0.60, -0.61)
                            };
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
            Stamp::Home(furniture) => {
                spawn("interior", HOME, 0., 0.);
                spawn("bedroom_bed", adventure.position(Target::Bed), 0., 0.);
                spawn("furniture_bench", adventure.position(Target::Bench), 0., 0.);
                for (i, kind) in furniture.iter().enumerate() {
                    spawn("selection", HOME + SLOTS[i], 0.055, 0.);
                    if let Some(name) = kind
                        .checked_sub(1)
                        .and_then(|v| FURNITURE_MODELS.get(v as usize))
                    {
                        spawn(name, HOME + SLOTS[i], 0.08, 0.);
                    }
                }
                spawn("selection", adventure.position(Target::Exit), 0.05, 0.);
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
                    let (x, z) = (cell as i16 % GRID, cell as i16 / GRID);
                    for (dx, dz, angle) in
                        [(0, -1, 0.), (1, 0, 1.5708), (0, 1, 0.), (-1, 0, 1.5708)]
                    {
                        let (nx, nz) = (x + dx, z + dz);
                        if !(0..GRID).contains(&nx)
                            || !(0..GRID).contains(&nz)
                            || !adventure.layout.cells.contains(&((nz * GRID + nx) as u16))
                        {
                            spawn(
                                if *depth >= 3 {
                                    "mine_deep_wall"
                                } else {
                                    "mine_wall"
                                },
                                p + Vec2::new(dx as f32, dz as f32) * TILE * 0.49,
                                -0.05,
                                angle,
                            );
                        }
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
                            adventure.position(Target::MineRock(cell)),
                            0.,
                            f32::from(cell) * 0.43,
                        );
                    }
                }
                spawn("mine_ladder", adventure.position(Target::Up), 0., 0.);
                spawn("mine_ladder", adventure.position(Target::Down), 0., 0.25);
                if *depth >= 3 {
                    Color::srgb_u8(36, 39, 60)
                } else {
                    Color::srgb_u8(57, 42, 62)
                }
            }
        };
        drawing.root = Some(root);
        drawing.stamp = Some(stamp);
    }
    if let Some(target) = adventure.focus {
        let p = adventure.position(target);
        let floor = if adventure.area.outdoors() {
            world::exterior_ground(p)
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
    if let Some((bite, _)) = adventure
        .waiting
        .filter(|(_, end)| time.elapsed_secs() <= *end)
    {
        let now = time.elapsed_secs();
        let y = -0.755 + (now * 8.).sin() * if now >= bite { 0.08 } else { 0.021 };
        let transform = Transform::from_xyz(6.52, y, 19.13);
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
}
