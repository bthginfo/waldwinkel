use crate::world::camera_offset;
use bevy::prelude::*;

use crate::{adventure::Adventure, garden::Garden, world::IslandCamera};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn at(day: u32) -> Self {
        [Self::Spring, Self::Summer, Self::Autumn, Self::Winter]
            [(day.saturating_sub(1) / 7 % 4) as usize]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Spring => "Frühling",
            Self::Summer => "Sommer",
            Self::Autumn => "Herbst",
            Self::Winter => "Winter",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sky {
    Sun,
    Mist,
    Rain,
    Snow,
}

impl Sky {
    pub fn at(day: u32) -> Self {
        match (day.saturating_sub(1) % 7, Season::at(day)) {
            (1 | 5, _) => Self::Mist,
            (2 | 4, Season::Winter) => Self::Snow,
            (2 | 4, _) => Self::Rain,
            _ => Self::Sun,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Sun => "Sonne",
            Self::Mist => "Nebel",
            Self::Rain => "Regen",
            Self::Snow => "Schnee",
        }
    }
}

#[derive(Component)]
pub struct WeatherTint;

#[derive(Component)]
pub struct Particle {
    phase: f32,
    x: f32,
    z: f32,
    snow: bool,
}

pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    for (snow, label, count) in [(false, "weather_rain", 54), (true, "weather_snow", 37)] {
        let mesh = assets.load::<Scene>(format!("models/{label}.glb#Scene0"));
        for i in 0..count {
            let a = i as f32;
            commands.spawn((
                Particle {
                    phase: (a * 0.626).fract(),
                    x: (a * 0.618 + 0.21).fract(),
                    z: (a * 0.414 + 0.47).fract(),
                    snow,
                },
                SceneRoot(mesh.clone()),
                Visibility::Hidden,
            ));
        }
    }
    commands.spawn((
        WeatherTint,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        BackgroundColor(Color::NONE),
        GlobalZIndex(-2),
    ));
}

pub fn update(
    garden: Res<Garden>,
    adventure: Res<Adventure>,
    time: Res<Time>,
    camera: Single<(&Transform, &Projection), (With<IslandCamera>, Without<Particle>)>,
    mut particles: Query<(&Particle, &mut Transform, &mut Visibility), Without<IslandCamera>>,
    mut tint: Single<&mut BackgroundColor, With<WeatherTint>>,
) {
    let outdoor = adventure.area.outdoors() && !adventure.map;
    let sky = Sky::at(garden.day);
    let season = Season::at(garden.day);
    let (camera, projection) = *camera;
    // In world space this is the focus of the same fixed-direction camera used for movement.
    let center = camera.translation - camera_offset(adventure.area);
    let width = match projection {
        Projection::Orthographic(ortho) => ortho.area.width().clamp(5., 65.),
        _ => 15.,
    };
    let elapsed = time.elapsed_secs();
    for (particle, mut transform, mut visible) in &mut particles {
        let active =
            outdoor && (particle.snow && sky == Sky::Snow || !particle.snow && sky == Sky::Rain);
        let visibility = if active {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visible != visibility {
            *visible = visibility;
        }
        if !active {
            continue;
        }
        let speed = if particle.snow { 0.12 } else { 0.83 };
        let phase = (particle.phase + elapsed * speed).fract();
        let drift = if particle.snow {
            (elapsed * 1.1 + particle.phase * 6.).sin() * 0.72
        } else {
            -phase * 0.88
        };
        transform.translation = Vec3::new(
            center.x + (particle.x - 0.5) * width + drift,
            9.1 - phase * 9.7,
            center.z + (particle.z - 0.5) * width * 0.73,
        );
    }
    tint.0 = if !outdoor {
        Color::NONE
    } else {
        match (sky, season) {
            (Sky::Rain, _) => Color::srgba(0.22, 0.39, 0.47, 0.21),
            (Sky::Mist, _) => Color::srgba(0.83, 0.87, 0.81, 0.18),
            (Sky::Snow, _) => Color::srgba(0.76, 0.86, 0.93, 0.25),
            (_, Season::Autumn) => Color::srgba(0.86, 0.49, 0.22, 0.10),
            (_, Season::Winter) => Color::srgba(0.70, 0.83, 0.97, 0.17),
            (_, Season::Summer) => Color::srgba(0.99, 0.73, 0.35, 0.06),
            _ => Color::NONE,
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seasons_repeat_and_winter_swaps_rain_for_snow() {
        assert_eq!(Season::at(1), Season::Spring);
        assert_eq!(Season::at(8), Season::Summer);
        assert_eq!(Season::at(15), Season::Autumn);
        assert_eq!(Season::at(22), Season::Winter);
        assert_eq!(Sky::at(24), Sky::Snow);
        assert_eq!(Sky::at(31), Sky::Rain);
    }
}
