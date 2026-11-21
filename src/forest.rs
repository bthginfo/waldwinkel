use std::{collections::HashMap, sync::LazyLock};

use bevy::prelude::*;

use crate::{
    garden::Garden,
    weather::Season,
    world::{self, Exterior},
};

const CELL: f32 = 8.;
const ID_OFFSET: usize = 1_024;
const MODELS: [[&str; 2]; 6] = [
    ["forage_0_summer", "forage_0_winter"],
    ["forage_1_summer", "forage_1_winter"],
    ["forage_2_summer", "forage_2_winter"],
    ["forage_3_summer", "forage_3_winter"],
    ["forage_4_summer", "forage_4_winter"],
    ["forage_5_summer", "forage_5_winter"],
];
const LEAF_MODELS: [[&str; 4]; 2] = [
    [
        "forage_6_spring",
        "forage_6_summer",
        "forage_6_autumn",
        "forage_6_winter",
    ],
    [
        "forage_7_spring",
        "forage_7_summer",
        "forage_7_autumn",
        "forage_7_winter",
    ],
];

#[derive(Clone, Copy)]
pub struct Node {
    pub id: usize,
    pub point: Vec2,
    pub height: f32,
    pub scale: f32,
    pub heading: f32,
    variant: u8,
}

impl Node {
    pub fn is_tree(self) -> bool {
        self.variant != 8
    }

    pub fn tool_level(self) -> u8 {
        match self.variant {
            6 => 1,
            7 => 2,
            8 if self.scale > 1.48 => 2,
            8 if self.scale > 1.27 => 1,
            _ => 0,
        }
    }

    pub fn name(self) -> &'static str {
        match self.variant {
            0..=5 => "Kiefer",
            6 => "Laubbaum",
            7 => "Moorbaum",
            8 if self.scale > 1.48 => "Granitfindling",
            8 if self.scale > 1.27 => "Schwerer Findling",
            _ => "Stein",
        }
    }

    pub fn top(self) -> f32 {
        (match self.variant {
            0..=5 => 4.88,
            6 | 7 => 3.72,
            _ => 0.97,
        }) * self.scale
    }

    pub fn radius(self) -> f32 {
        if self.is_tree() {
            0.32 * self.scale + 0.17
        } else {
            0.76 * self.scale + 0.12
        }
    }

    pub fn model(self, season: Season) -> &'static str {
        match self.variant {
            0..=5 => MODELS[self.variant as usize][usize::from(season == Season::Winter)],
            6 | 7 => LEAF_MODELS[(self.variant - 6) as usize][season as usize],
            _ => "forage_8_summer",
        }
    }

    fn drawn_model(self, garden: &Garden) -> Option<&'static str> {
        if garden.workshop.annex_at(self.point) {
            None
        } else if self.is_tree() && garden.felled.contains(&self.id) {
            Some("wood_stump")
        } else if !self.is_tree() && garden.broken.contains(&self.id) {
            None
        } else {
            Some(self.model(Season::at(garden.day)))
        }
    }
}

#[derive(Default)]
struct Forest {
    nodes: Vec<Node>,
    by_id: HashMap<usize, usize>,
    cells: HashMap<IVec2, Vec<usize>>,
}

fn cell(point: Vec2) -> IVec2 {
    (point / CELL).floor().as_ivec2()
}

static FOREST: LazyLock<Forest> = LazyLock::new(|| {
    let data = include_bytes!("../assets/world/harvest.bin");
    let (header, body) = data.split_at(8);
    assert_eq!(&header[..4], b"WHV1");
    assert_eq!(
        body.len(),
        u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize * 21
    );
    let mut forest = Forest::default();
    for row in body.chunks_exact(21) {
        let value = |i: usize| f32::from_le_bytes(row[i * 4..i * 4 + 4].try_into().unwrap());
        let point = Vec2::new(value(0), value(1));
        // Coordinate-based IDs keep saves stable when unrelated scenery is added.
        let grid = (point * 16.).round().as_ivec2() + IVec2::splat(4096);
        assert!((0..8192).contains(&grid.x) && (0..8192).contains(&grid.y));
        let id = ID_OFFSET + (grid.x * 8192 + grid.y) as usize;
        let index = forest.nodes.len();
        assert!(
            forest.by_id.insert(id, index).is_none(),
            "duplicate resource at {point}"
        );
        forest.nodes.push(Node {
            id,
            point,
            height: value(2),
            scale: value(3),
            heading: -value(4),
            variant: row[20],
        });
        forest.cells.entry(cell(point)).or_default().push(index);
    }
    forest
});

pub fn node(id: usize) -> Option<Node> {
    FOREST.by_id.get(&id).map(|&i| FOREST.nodes[i])
}

pub fn near(point: Vec2, radius: f32) -> impl Iterator<Item = Node> {
    let low = cell(point - Vec2::splat(radius));
    let high = cell(point + Vec2::splat(radius));
    (low.x..=high.x)
        .flat_map(move |x| (low.y..=high.y).map(move |y| IVec2::new(x, y)))
        .filter_map(|key| FOREST.cells.get(&key))
        .flatten()
        .map(|&i| FOREST.nodes[i])
        .filter(move |node| node.point.distance_squared(point) <= radius * radius)
}

pub fn blocks(point: Vec2, garden: &Garden) -> bool {
    near(point, 1.65).any(|node| {
        let radius = if node.is_tree() && garden.felled.contains(&node.id) {
            0.27 * node.scale + 0.15
        } else if !node.is_tree() && garden.broken.contains(&node.id) {
            return false;
        } else {
            node.radius()
        };
        node.point.distance_squared(point) < radius * radius
    })
}

#[derive(Component)]
pub struct Forage {
    node: Node,
    drawn: Option<&'static str>,
}

pub fn setup(mut commands: Commands) {
    for &node in &FOREST.nodes {
        commands.spawn((
            Exterior,
            Forage { node, drawn: None },
            Visibility::Inherited,
            Transform::from_xyz(node.point.x, node.height, node.point.y)
                .with_rotation(Quat::from_rotation_y(node.heading))
                .with_scale(Vec3::splat(node.scale)),
        ));
    }
}

#[derive(Default)]
pub struct VisualState {
    season: Option<Season>,
    felled: Vec<usize>,
    broken: Vec<usize>,
    house: u8,
}

pub fn draw(
    mut commands: Commands,
    assets: Res<AssetServer>,
    garden: Res<Garden>,
    mut state: Local<VisualState>,
    mut nodes: Query<(Entity, &mut Forage)>,
) {
    let season = Season::at(garden.day);
    if state.season == Some(season)
        && state.felled == garden.felled
        && state.broken == garden.broken
        && state.house == garden.workshop.house
    {
        return;
    }
    state.season = Some(season);
    state.house = garden.workshop.house;
    state.felled.clone_from(&garden.felled);
    state.broken.clone_from(&garden.broken);
    for (entity, mut forage) in &mut nodes {
        let model = forage.node.drawn_model(&garden);
        if forage.drawn != model {
            let scene = model.map(|name| assets.load(format!("models/{name}.glb#Scene0")));
            world::replace_model(&mut commands, entity, scene.as_ref());
            forage.drawn = model;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baked_resources_keep_ids_and_respect_removal() {
        let mut garden = Garden::default();
        let tree = FOREST
            .nodes
            .iter()
            .copied()
            .find(|node| node.is_tree())
            .unwrap();
        let stone = FOREST
            .nodes
            .iter()
            .copied()
            .find(|node| !node.is_tree())
            .unwrap();
        assert_eq!(node(tree.id).unwrap().point, tree.point);
        assert!(blocks(tree.point, &garden));
        assert!(garden.chop_tree(tree.id));
        assert_eq!(tree.drawn_model(&garden), Some("wood_stump"));
        for _ in 0..3 {
            garden.sleep();
        }
        assert_eq!(tree.drawn_model(&garden), Some(tree.model(Season::Spring)));
        assert!(blocks(stone.point, &garden));
        garden.broken.push(stone.id);
        assert!(stone.drawn_model(&garden).is_none());
        assert!(!blocks(stone.point, &garden));
    }
}
