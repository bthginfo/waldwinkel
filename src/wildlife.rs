use std::f32::consts::TAU;

use bevy::{prelude::*, scene::SceneInstanceReady};

use crate::{
    Adventure, Garden,
    weather::{Season, Sky},
    world::{self, Gardener},
};

#[derive(Clone, Copy)]
enum Kind {
    Bird,
    Butterfly,
    Beetle,
    Firefly,
}

impl Kind {
    fn asset(self) -> &'static str {
        match self {
            Self::Bird => "wild_bird",
            Self::Butterfly => "wild_butterfly",
            Self::Beetle => "wild_beetle",
            Self::Firefly => "wild_firefly",
        }
    }

    fn active(self, minute: u16, sky: Sky, season: Season) -> bool {
        match self {
            Self::Bird => (360..1200).contains(&minute) && !matches!(sky, Sky::Rain | Sky::Snow),
            Self::Butterfly => {
                (390..1120).contains(&minute) && sky == Sky::Sun && season != Season::Winter
            }
            Self::Beetle => {
                (390..1190).contains(&minute) && sky != Sky::Snow && season != Season::Winter
            }
            Self::Firefly => {
                minute >= 1110
                    && !matches!(sky, Sky::Rain | Sky::Snow)
                    && matches!(season, Season::Spring | Season::Summer)
            }
        }
    }
}

#[derive(Component)]
pub(crate) struct Critter {
    kind: Kind,
    home: Vec2,
    variation: f32,
    scale: f32,
}

#[derive(Component)]
pub(crate) struct Wing {
    host: Entity,
    side: f32,
}

pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let mut seed = 0;
    for (kind, sites, count) in [
        (
            Kind::Bird,
            &[
                (-3.5, 7.4),
                (-53.1, 4.7),
                (51.9, 9.3),
                (6.3, 53.7),
                (19.3, 118.9),
            ][..],
            2,
        ),
        (
            Kind::Butterfly,
            &[
                (3.25, 4.35),
                (-3.4, 5.65),
                (-47.6, -8.4),
                (44.6, 6.6),
                (13.1, 115.9),
            ][..],
            2,
        ),
        (
            Kind::Beetle,
            &[
                (-2.6, 4.),
                (3.75, 4.2),
                (-50.3, -7.),
                (45.3, 6.4),
                (-142.1, -5.),
            ][..],
            1,
        ),
        (
            Kind::Firefly,
            &[(2.0, 6.0), (-5.6, 5.5), (-47.5, -7.4), (-145.2, -5.3)][..],
            3,
        ),
    ] {
        let model: Handle<Scene> = assets.load(format!("models/{}.glb#Scene0", kind.asset()));
        for &(x, z) in sites {
            for member in 0..count {
                let variation = seed as f32 * 1.724 + member as f32 * 0.7;
                let size = 0.89 + (seed % 5) as f32 * 0.058;
                commands
                    .spawn((
                        Critter {
                            kind,
                            home: Vec2::new(x, z),
                            variation,
                            scale: size,
                        },
                        SceneRoot(model.clone()),
                        Transform::default(),
                        Visibility::Hidden,
                    ))
                    .observe(bind_wings);
            }
            seed += 1;
        }
    }
}

fn bind_wings(
    trigger: Trigger<SceneInstanceReady>,
    children: Query<&Children>,
    names: Query<&Name>,
    mut commands: Commands,
) {
    for entity in children.iter_descendants(trigger.target()) {
        let Ok(name) = names.get(entity) else {
            continue;
        };
        let side = if name.as_str().ends_with("_wing_l") {
            -1.
        } else if name.as_str().ends_with("_wing_r") {
            1.
        } else {
            continue;
        };
        commands.entity(entity).insert(Wing {
            host: trigger.target(),
            side,
        });
    }
}

pub fn animate(
    garden: Res<Garden>,
    adventure: Res<Adventure>,
    player: Single<&Gardener>,
    time: Res<Time>,
    mut critters: Query<(&Critter, &mut Transform, &mut Visibility), Without<Wing>>,
    mut wings: Query<(&Wing, &mut Transform), Without<Critter>>,
) {
    let outside = adventure.area.outdoors() && !adventure.map;
    let minute = garden.clock.minute();
    let season = Season::at(garden.day);
    let sky = Sky::at(garden.day);
    let now = time.elapsed_secs();
    for (critter, mut transform, mut visibility) in &mut critters {
        if !outside
            || !critter.kind.active(minute, sky, season)
            || critter.home.distance_squared(player.position) > 25.0_f32.powi(2)
        {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        }
        let seed = critter.variation;
        let (speed, radius, lift) = match critter.kind {
            Kind::Bird => (0.37, Vec2::new(4.85, 2.58), 2.85),
            Kind::Butterfly => (0.84, Vec2::new(0.91, 0.69), 0.91),
            Kind::Beetle => (0.36, Vec2::new(0.43, 0.29), 0.012),
            Kind::Firefly => (0.68, Vec2::new(1.25, 1.04), 0.71),
        };
        let orbit = (now * speed + seed) % TAU;
        let drift = Vec2::new(orbit.cos(), orbit.sin()) * radius;
        let point = critter.home + drift;
        if matches!(critter.kind, Kind::Beetle) && !world::walkable(point) {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        }
        let rise = match critter.kind {
            Kind::Bird => (orbit * 2.).sin() * 0.33,
            Kind::Butterfly => (now * 2.37 + seed).sin() * 0.24,
            Kind::Beetle => 0.,
            Kind::Firefly => (now * 1.42 + seed).sin() * 0.35,
        };
        let ground = world::surface_height(point, &garden);
        let direction = Vec2::new(-orbit.sin() * radius.x, orbit.cos() * radius.y);
        let bank = if matches!(critter.kind, Kind::Bird) {
            0.17
        } else {
            0.
        };
        let scale = if matches!(critter.kind, Kind::Firefly) {
            critter.scale * (0.5 + 0.5 * (now * 2.5 + seed).sin()).powi(2).max(0.04)
        } else {
            critter.scale
        };
        transform.translation = Vec3::new(point.x, ground + lift + rise, point.y);
        transform.rotation =
            Quat::from_rotation_y(direction.x.atan2(direction.y)) * Quat::from_rotation_z(bank);
        transform.scale = Vec3::splat(scale);
        visibility.set_if_neq(Visibility::Inherited);
    }
    for (wing, mut transform) in &mut wings {
        let Ok((critter, _, visibility)) = critters.get(wing.host) else {
            continue;
        };
        if *visibility == Visibility::Hidden {
            continue;
        }
        let seed = critter.variation;
        let angle = match critter.kind {
            Kind::Bird => {
                let flap = ((now * 0.92 + seed).sin() * 2.).clamp(0., 1.);
                (now * 20. + seed).sin() * flap * 0.87 + 0.08
            }
            Kind::Butterfly => 0.61 + 0.59 * (now * 27. + seed).sin(),
            _ => 0.,
        };
        transform.rotation = Quat::from_rotation_z(angle * wing.side);
    }
}
