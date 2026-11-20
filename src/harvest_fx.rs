use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::{audio::Volume, prelude::*};

use crate::{Adventure, Garden, adventure::Area, session::Session, weather::Season, world};

const CONTACT: f32 = 0.19;
const FEATURE_TIME: f32 = 1.37;
const CHIP_TIME: f32 = 0.79;

#[derive(Event)]
pub struct Break {
    pub area: Area,
    pub point: Vec2,
    pub away: Vec2,
    pub heading: f32,
    pub model: &'static str,
    pub tree: bool,
    pub floor: Option<f32>,
    pub scale: f32,
    pub snow_cap: bool,
}

#[derive(Component)]
pub struct HarvestEffect {
    area: Area,
    started: f32,
    floor: f32,
    point: Vec2,
    axis: Vec3,
    heading: f32,
    tree: bool,
    cracks: bool,
    scale: f32,
}

#[derive(Component)]
pub struct Chip {
    area: Area,
    started: f32,
    floor: f32,
    origin: Vec3,
    drift: Vec3,
    angle: Vec3,
    size: f32,
}

fn scene(assets: &AssetServer, asset: &str) -> SceneRoot {
    SceneRoot(assets.load(format!("models/{asset}.glb#Scene0")))
}

pub fn animate(
    mut commands: Commands,
    assets: Res<AssetServer>,
    garden: Res<Garden>,
    adventure: Res<Adventure>,
    session: Res<Session>,
    time: Res<Time>,
    mut hits: EventReader<Break>,
    mut effects: Query<(Entity, &mut HarvestEffect, &mut Transform), Without<Chip>>,
    mut fragments: Query<(Entity, &Chip, &mut Transform), Without<HarvestEffect>>,
) {
    let now = time.elapsed_secs();
    for hit in hits.read() {
        let floor = hit.floor.unwrap_or_else(|| {
            if hit.area.outdoors() {
                world::surface_height(hit.point, &garden)
            } else {
                0.
            }
        });
        let face = hit.away.try_normalize().unwrap_or(Vec2::X);
        let mut effect = commands.spawn((
            scene(&assets, hit.model),
            Transform::from_xyz(hit.point.x, floor, hit.point.y)
                .with_rotation(Quat::from_rotation_y(hit.heading))
                .with_scale(Vec3::splat(hit.scale)),
            HarvestEffect {
                area: hit.area,
                started: now,
                floor,
                point: hit.point,
                axis: Vec3::Y.cross(Vec3::new(face.x, 0., face.y)),
                heading: hit.heading,
                tree: hit.tree,
                cracks: false,
                scale: hit.scale,
            },
        ));
        if hit.snow_cap && Season::at(garden.day) == Season::Winter {
            effect.with_child(scene(&assets, "wood_snow"));
        }
    }
    for (entity, mut effect, mut pose) in &mut effects {
        let age = now - effect.started;
        if age > FEATURE_TIME || effect.area != adventure.area || !session.playing() {
            commands.entity(entity).despawn();
            continue;
        }
        if age < CONTACT {
            continue;
        }
        if !effect.cracks {
            effect.cracks = true;
            throw_chips(&mut commands, &assets, &effect, now);
            if !garden.sound_muted {
                let name = if effect.tree {
                    "axe_cut"
                } else {
                    "stone_break"
                };
                commands.spawn((
                    AudioPlayer::new(assets.load(format!("audio/{name}.wav"))),
                    PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.43)),
                ));
            }
        }
        let elapsed = age - CONTACT;
        let base = Quat::from_rotation_y(effect.heading);
        if effect.tree {
            let t = (elapsed / 0.79).clamp(0., 1.);
            let bounce = ((elapsed - 0.79) * 19.).sin() * 0.045 * (1. - elapsed).max(0.);
            pose.rotation =
                Quat::from_axis_angle(effect.axis, FRAC_PI_2 * t.powf(2.3) - bounce * t) * base;
            pose.translation.y = effect.floor - ((elapsed - 0.94) * 2.1).max(0.);
        } else {
            pose.rotation = Quat::from_rotation_y((elapsed * 21.).sin() * 0.19) * base;
            pose.scale = Vec3::splat(effect.scale * (1. - elapsed * 1.89).max(0.001));
        }
    }
    for (entity, chip, mut pose) in &mut fragments {
        let age = now - chip.started;
        if age > CHIP_TIME || chip.area != adventure.area || !session.playing() {
            commands.entity(entity).despawn();
            continue;
        }
        let mut point = chip.origin + chip.drift * age;
        point.y = (point.y - 4.95 * age * age).max(chip.floor + chip.size * 0.15);
        let fade = ((CHIP_TIME - age) / 0.24).clamp(0., 1.);
        pose.translation = point;
        pose.scale = Vec3::splat(chip.size * fade);
        pose.rotation = Quat::from_euler(
            EulerRot::XYZ,
            chip.angle.x * age,
            chip.angle.y * age,
            chip.angle.z * age,
        );
    }
}

fn throw_chips(commands: &mut Commands, assets: &AssetServer, effect: &HarvestEffect, now: f32) {
    let asset = if effect.tree {
        "wood_stump"
    } else {
        "resource_rock"
    };
    let center = Vec3::new(effect.point.x, effect.floor, effect.point.y);
    for i in 0..10 {
        let a = TAU * (i as f32 * 0.618).fract();
        let speed = 0.91 + (i % 4) as f32 * 0.43;
        let size = 0.092 + (i % 3) as f32 * 0.039;
        let drift = Vec3::new(
            a.cos() * speed,
            1.37 + (i % 5) as f32 * 0.24,
            a.sin() * speed,
        );
        let source = center
            + Vec3::new(
                a.cos() * 0.28,
                if effect.tree { 0.58 } else { 0.39 },
                a.sin() * 0.28,
            );
        commands.spawn((
            scene(assets, asset),
            Transform::from_translation(source).with_scale(Vec3::splat(size)),
            Chip {
                area: effect.area,
                started: now,
                floor: effect.floor,
                origin: source,
                drift,
                angle: Vec3::new(4.1 + i as f32, 8.8 - i as f32, 7.9),
                size,
            },
        ));
    }
}
