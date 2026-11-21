//! The walkable surfaces baked alongside the Blender landscape.
use bevy::prelude::*;

const MAP: &[u8] = include_bytes!("../assets/world/terrain.bin");
const WEATHER: &[u8] = include_bytes!("../assets/world/weather.bin");
const HEADER: usize = 20;
const SAMPLE: usize = 3;

const fn map_float(data: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

#[derive(Clone, Copy)]
struct Tile {
    data: &'static [u8],
    origin: Vec2,
    step: f32,
    cols: usize,
    rows: usize,
}

impl Tile {
    const fn new(data: &'static [u8]) -> Self {
        let cols = u16::from_le_bytes([data[16], data[17]]) as usize;
        let rows = u16::from_le_bytes([data[18], data[19]]) as usize;
        assert!(data[0] == b'W' && data[3] == b'5' && data.len() == HEADER + SAMPLE * cols * rows);
        Self {
            data,
            origin: Vec2::new(map_float(data, 4), map_float(data, 8)),
            step: map_float(data, 12),
            cols,
            rows,
        }
    }

    fn sample(self, x: usize, z: usize) -> (f32, bool) {
        let offset = HEADER + SAMPLE * (z * self.cols + x);
        let height = i16::from_le_bytes([self.data[offset], self.data[offset + 1]]) as f32 / 100.;
        (height, self.data[offset + 2] != 0)
    }

    fn grid(self, point: Vec2) -> Option<Vec2> {
        let p = (point - self.origin) / self.step;
        (p.is_finite() && p.x >= 0. && p.y >= 0. && p.x < (self.cols - 1) as f32 && p.y < (self.rows - 1) as f32)
            .then_some(p)
    }

    fn height_at(self, p: Vec2) -> f32 {
        let x = p.x as usize;
        let z = p.y as usize;
        let (tx, tz) = (p.x.fract(), p.y.fract());
        let back = self.sample(x, z).0.lerp(self.sample(x + 1, z).0, tx);
        let front = self.sample(x, z + 1).0.lerp(self.sample(x + 1, z + 1).0, tx);
        back.lerp(front, tz)
    }
}

const TILES: [Tile; 4] = [
    Tile::new(include_bytes!("../assets/world/extension_amber.bin")),
    Tile::new(include_bytes!("../assets/world/extension_heights.bin")),
    Tile::new(include_bytes!("../assets/world/extension_reef.bin")),
    Tile::new(MAP),
];
const ATLAS_MIN: Vec2 = Vec2::new(-323., -104.);
const ATLAS_SPAN: Vec2 = Vec2::new(644., 395.);

fn grid(point: Vec2) -> Option<(Tile, Vec2)> {
    TILES.into_iter().find_map(|tile| tile.grid(point).map(|p| (tile, p)))
}

pub fn ground(point: Vec2) -> f32 {
    grid(point).map_or(-1.43, |(tile, p)| tile.height_at(p))
}

pub fn atlas_position(point: Vec2) -> Vec2 {
    ((point - ATLAS_MIN) / ATLAS_SPAN).clamp(Vec2::ZERO, Vec2::ONE)
}

pub fn atlas_ratio() -> f32 {
    ATLAS_SPAN.y / ATLAS_SPAN.x
}

pub fn walkable(point: Vec2) -> bool {
    grid(point).is_some_and(|(tile, p)| {
        let blocked = tile.sample(p.x.round() as usize, p.y.round() as usize).1;
        !blocked && tile.height_at(p) > -0.40
    })
}

fn legacy_cover(point: Vec2) -> f32 {
    const ORIGIN: Vec2 = Vec2::new(map_float(WEATHER, 4), map_float(WEATHER, 8));
    const STEP: f32 = map_float(WEATHER, 12);
    const COLS: usize = u16::from_le_bytes([WEATHER[16], WEATHER[17]]) as usize;
    const ROWS: usize = u16::from_le_bytes([WEATHER[18], WEATHER[19]]) as usize;
    const _: () = assert!(
        WEATHER[0] == b'W' && WEATHER[3] == b'1' && WEATHER.len() == HEADER + COLS * ROWS * 2
    );
    let cell = ((point - ORIGIN) / STEP).round();
    if !cell.is_finite()
        || cell.min_element() < 0.
        || cell.x >= COLS as f32
        || cell.y >= ROWS as f32
    {
        return -7.0;
    }
    let index = HEADER + (cell.y as usize * COLS + cell.x as usize) * 2;
    f32::from(i16::from_le_bytes([WEATHER[index], WEATHER[index + 1]])) * 0.01
}

pub fn weather_cover(point: Vec2) -> f32 {
    const ROOFS: &[u8] = include_bytes!("../assets/world/extension_roofs.bin");
    const _: () = assert!(ROOFS[3] == b'1' && (ROOFS.len() - 4) % 24 == 0);
    let mut cover = legacy_cover(point);
    for roof in ROOFS[4..].chunks_exact(24) {
        let offset = (point - Vec2::new(map_float(roof, 0), map_float(roof, 4))).abs();
        let half_width = map_float(roof, 12);
        if offset.x < half_width && offset.y < map_float(roof, 16) {
            let height = map_float(roof, 8) + (1. - offset.x / half_width) * map_float(roof, 20);
            cover = cover.max(height);
        }
    }
    cover
}
