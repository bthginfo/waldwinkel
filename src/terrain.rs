//! The walkable surfaces baked alongside the Blender landscape.
use bevy::prelude::*;

const MAP: &[u8] = include_bytes!("../assets/world/terrain.bin");
const HEADER: usize = 20;
const SAMPLE: usize = 3;

const fn map_float(offset: usize) -> f32 {
    f32::from_le_bytes([
        MAP[offset],
        MAP[offset + 1],
        MAP[offset + 2],
        MAP[offset + 3],
    ])
}

const ORIGIN: Vec2 = Vec2::new(map_float(4), map_float(8));
const STEP: f32 = map_float(12);
const COLS: usize = u16::from_le_bytes([MAP[16], MAP[17]]) as usize;
const ROWS: usize = u16::from_le_bytes([MAP[18], MAP[19]]) as usize;
const _: () =
    assert!(MAP[0] == b'W' && MAP[3] == b'5' && MAP.len() == HEADER + SAMPLE * COLS * ROWS);

fn sample(x: usize, z: usize) -> (f32, bool) {
    let offset = HEADER + SAMPLE * (z * COLS + x);
    let height = i16::from_le_bytes([MAP[offset], MAP[offset + 1]]) as f32 / 100.0;
    (height, MAP[offset + 2] != 0)
}

fn grid(point: Vec2) -> Option<Vec2> {
    let p = (point - ORIGIN) / STEP;
    (p.is_finite() && p.x >= 0. && p.y >= 0. && p.x < (COLS - 1) as f32 && p.y < (ROWS - 1) as f32)
        .then_some(p)
}

fn height_at(p: Vec2) -> f32 {
    let x = p.x as usize;
    let z = p.y as usize;
    let tx = p.x.fract();
    let tz = p.y.fract();
    let back = sample(x, z).0.lerp(sample(x + 1, z).0, tx);
    let front = sample(x, z + 1).0.lerp(sample(x + 1, z + 1).0, tx);
    back.lerp(front, tz)
}

pub fn ground(point: Vec2) -> f32 {
    grid(point).map_or(-1.43, height_at)
}

pub fn walkable(point: Vec2) -> bool {
    grid(point).is_some_and(|p| {
        let blocked = sample(p.x.round() as usize, p.y.round() as usize).1;
        !blocked && height_at(p) > -0.40
    })
}
