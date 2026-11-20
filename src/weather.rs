use bevy::{
    gltf::Gltf,
    pbr::FogFalloff,
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        view::ColorGrading,
    },
};

use crate::{
    adventure::Adventure,
    garden::Garden,
    seasons::MEETING_POINT,
    terrain,
    world::{IslandCamera, camera_offset, exterior_ground, surface_height},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub const LENGTH: u32 = 7;
    pub const ALL: [Self; 4] = [Self::Spring, Self::Summer, Self::Autumn, Self::Winter];

    pub fn at(day: u32) -> Self {
        Self::ALL[(Self::period(day) % Self::ALL.len() as u32) as usize]
    }

    pub fn day(day: u32) -> u32 {
        day.saturating_sub(1) % Self::LENGTH + 1
    }

    pub fn period(day: u32) -> u32 {
        day.saturating_sub(1) / Self::LENGTH
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::Spring => "spring",
            Self::Summer => "summer",
            Self::Autumn => "autumn",
            Self::Winter => "winter",
        }
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
        match (Season::day(day), Season::at(day)) {
            (2 | 6, _) => Self::Mist,
            (3 | 5, Season::Winter) => Self::Snow,
            (3 | 5, _) => Self::Rain,
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

pub fn wind(sky: Sky, elapsed: f32) -> Vec2 {
    let speed = match sky {
        Sky::Sun => 0.45,
        Sky::Mist => 0.26,
        Sky::Rain => 2.10,
        Sky::Snow => 0.81,
    };
    Vec2::new(-0.84, 0.55).rotate(Vec2::from_angle(0.29 * (elapsed * 0.39).sin()))
        * speed
        * (1. + 0.26 * (elapsed * 0.78).sin())
}

#[derive(Component)]
pub struct WeatherTint;

#[derive(Component)]
pub struct Particle {
    seed: u32,
    snow: bool,
    speed: f32,
    phase: f32,
}

#[derive(Component)]
pub struct Splash {
    age: f32,
}

#[derive(Component)]
pub struct NightGlow;

#[derive(Resource)]
pub struct NightMaterials {
    windows: Handle<Gltf>,
    lantern: Handle<StandardMaterial>,
}

pub fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (snow, label, count) in [(false, "weather_rain", 64), (true, "weather_snow", 49)] {
        let mesh = assets.load::<Scene>(format!("models/{label}.glb#Scene0"));
        for i in 0..count {
            commands.spawn((
                Particle {
                    seed: 1731 + i * 397,
                    snow,
                    speed: 0.,
                    phase: 0.,
                },
                SceneRoot(mesh.clone()),
                Visibility::Hidden,
            ));
        }
    }
    let ring = meshes.add(Annulus::new(0.86, 1.).mesh().resolution(12));
    let splash = materials.add(StandardMaterial {
        base_color: Color::srgba(0.75, 0.89, 0.88, 0.46),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });
    for _ in 0..14 {
        commands.spawn((
            Splash { age: 1. },
            Mesh3d(ring.clone()),
            MeshMaterial3d(splash.clone()),
            Visibility::Hidden,
            Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ));
    }

    let halo = lamp_texture(&mut images);
    let lantern = materials.add(StandardMaterial {
        base_color_texture: Some(halo),
        base_color: Color::srgba(1., 0.66, 0.30, 0.),
        unlit: true,
        alpha_mode: AlphaMode::Add,
        fog_enabled: false,
        ..default()
    });
    let light_plane = meshes.add(Plane3d::default().mesh().size(1., 1.));
    for (point, size) in [
        (Vec2::new(-2.18, 1.45), 2.40),
        (Vec2::new(-4.3, 0.5), 2.10),
        (MEETING_POINT + Vec2::new(0.58, 0.28), 2.66),
        (Vec2::new(45.4, 8.1), 2.66),
        (Vec2::new(54.1, 12.8), 2.30),
    ] {
        commands.spawn((
            NightGlow,
            Mesh3d(light_plane.clone()),
            MeshMaterial3d(lantern.clone()),
            Visibility::Hidden,
            Transform::from_xyz(point.x, exterior_ground(point) + 0.029, point.y)
                .with_scale(Vec3::new(size, 1., size)),
        ));
    }
    commands.spawn((
        NightGlow,
        SceneRoot(assets.load("models/night_windows.glb#Scene0")),
        Visibility::Hidden,
    ));
    commands.insert_resource(NightMaterials {
        windows: assets.load("models/night_windows.glb"),
        lantern,
    });
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

fn lamp_texture(images: &mut Assets<Image>) -> Handle<Image> {
    const SIZE: u32 = 48;
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for z in 0..SIZE {
        for x in 0..SIZE {
            let p = Vec2::new(x as f32 + 0.5, z as f32 + 0.5) * (2. / SIZE as f32) - Vec2::ONE;
            let intensity = (1. - p.length_squared()).max(0.).powi(3);
            rgba.extend([255, 255, 255, (intensity * 255.) as u8]);
        }
    }
    images.add(Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ))
}

fn sample(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    (*seed >> 8) as f32 * (1. / 16_777_216.)
}

fn reset_drop(
    drop: &mut Particle,
    transform: &mut Transform,
    center: Vec2,
    width: f32,
    fill: bool,
) {
    let point = center
        + Vec2::new(sample(&mut drop.seed) - 0.5, sample(&mut drop.seed) - 0.5)
            * width
            * Vec2::new(1.2, 1.0);
    let size = 0.68 + sample(&mut drop.seed) * 0.48;
    drop.phase = sample(&mut drop.seed) * std::f32::consts::TAU;
    drop.speed = size * if drop.snow { 1.18 } else { 19. };
    transform.scale = Vec3::splat(size);
    let base = terrain::weather_cover(point).max(exterior_ground(point));
    let height = if fill {
        0.25 + 8.5 * sample(&mut drop.seed)
    } else {
        8.0 + 2.1 * size
    };
    transform.translation = Vec3::new(point.x, base.max(-0.75) + height, point.y);
}

pub fn precipitation(
    garden: Res<Garden>,
    adventure: Res<Adventure>,
    time: Res<Time>,
    camera: Single<(&Transform, &Projection), (With<IslandCamera>, Without<Particle>)>,
    mut particles: Query<(&mut Particle, &mut Transform, &mut Visibility), Without<Splash>>,
    mut splashes: Query<
        (&mut Splash, &mut Transform, &mut Visibility),
        (Without<Particle>, Without<IslandCamera>),
    >,
) {
    let sky = Sky::at(garden.day);
    let outdoor = adventure.area.outdoors() && !adventure.map;
    let dt = time.delta_secs().min(0.09);
    let elapsed = time.elapsed_secs();
    let gust = wind(sky, elapsed);
    let (camera, projection) = *camera;
    let center = (camera.translation - camera_offset(adventure.area)).xz();
    let width = match projection {
        Projection::Orthographic(ortho) => ortho.area.width().clamp(13., 46.),
        _ => 22.,
    };
    for (mut splash, mut transform, mut visible) in &mut splashes {
        splash.age = (splash.age + dt).min(1.);
        let active = outdoor && sky == Sky::Rain && splash.age < 0.30;
        set_visible(&mut visible, active);
        if active {
            transform.scale = Vec3::splat(0.035 + splash.age * 0.48);
        }
    }
    for (mut drop, mut transform, mut visible) in &mut particles {
        let active = outdoor
            && if drop.snow {
                sky == Sky::Snow
            } else {
                sky == Sky::Rain
            };
        let appearing = active && *visible == Visibility::Hidden;
        set_visible(&mut visible, active);
        if !active {
            continue;
        }
        let sway = if drop.snow {
            (elapsed * 2.4 + drop.phase).sin() * 0.8
        } else {
            0.
        };
        let velocity = Vec3::new(gust.x + sway, -drop.speed, gust.y - sway * 0.34);
        let next = transform.translation + velocity * dt;
        let point = next.xz();
        let outside = (point - center).abs().max_element() > width * 0.66;
        if appearing || outside {
            reset_drop(&mut drop, &mut transform, center, width, true);
            continue;
        }
        let ground = surface_height(point, &garden).max(-0.79);
        let surface = ground.max(terrain::weather_cover(point));
        if next.y <= surface {
            if !drop.snow
                && drop.seed % 3 == 0
                && surface < ground + 0.27
                && let Some((mut splash, mut location, mut visible)) =
                    splashes.iter_mut().find(|(splash, _, _)| splash.age >= 0.3)
            {
                splash.age = 0.;
                location.translation = Vec3::new(point.x, surface + 0.034, point.y);
                location.scale = Vec3::splat(0.048);
                set_visible(&mut visible, true);
            }
            reset_drop(&mut drop, &mut transform, center, width, false);
            continue;
        }
        transform.translation = next;
        transform.rotation = if drop.snow {
            Quat::from_rotation_y(elapsed + drop.phase)
        } else {
            Quat::from_rotation_arc(Vec3::Y, -velocity.normalize())
        };
    }
}

fn set_visible(visibility: &mut Visibility, visible: bool) {
    let next = if visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    if *visibility != next {
        *visibility = next;
    }
}

pub fn lighting(
    menu: Res<crate::farm_ui::FarmMenu>,
    garden: Res<Garden>,
    adventure: Res<Adventure>,
    time: Res<Time>,
    mut camera: Single<(&mut DistanceFog, &mut ColorGrading), With<IslandCamera>>,
    mut tint: Single<&mut BackgroundColor, With<WeatherTint>>,
    lights: Res<NightMaterials>,
    gltfs: Res<Assets<Gltf>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut glows: Query<&mut Visibility, With<NightGlow>>,
) {
    let outdoor = adventure.area.outdoors() && !adventure.map && menu.room_preview().is_none();
    let sky = Sky::at(garden.day);
    let season = Season::at(garden.day);
    let weather = if !outdoor {
        Color::NONE
    } else {
        match (sky, season) {
            (Sky::Rain, _) => Color::srgba(0.21, 0.33, 0.43, 0.20),
            (Sky::Mist, _) => Color::srgba(0.77, 0.82, 0.74, 0.11),
            (Sky::Snow, _) => Color::srgba(0.78, 0.86, 0.91, 0.12),
            (_, Season::Autumn) => Color::srgba(0.91, 0.58, 0.31, 0.061),
            (_, Season::Winter) => Color::srgba(0.66, 0.82, 0.96, 0.092),
            (_, Season::Summer) => Color::srgba(1., 0.81, 0.41, 0.042),
            _ => Color::srgba(0.97, 0.81, 0.64, 0.022),
        }
    };
    let dusk = if outdoor { garden.clock.dusk() } else { 0. };
    let golden = Color::srgba(0.96, 0.56, 0.30, 0.21);
    let night = Color::srgba(0.16, 0.27, 0.49, 0.31);
    let target = if dusk <= 0.55 {
        weather.mix(&golden, dusk / 0.55)
    } else {
        golden.mix(&night, (dusk - 0.55) / 0.45)
    };
    let ease = (1. - (-time.delta_secs() * 4.6).exp()).clamp(0., 1.);
    tint.0 = if outdoor {
        tint.0.mix(&target, ease)
    } else {
        Color::NONE
    };

    let (fog, grading) = &mut *camera;
    let moon = ((dusk - 0.46) / 0.54).clamp(0., 1.);
    let sunset = (1. - ((dusk - 0.43) / 0.43).abs()).max(0.);
    let overcast = if outdoor && sky == Sky::Rain {
        0.10
    } else {
        0.
    };
    grading.global.exposure = -moon * 0.91 - overcast + sunset * 0.095;
    grading.global.temperature = sunset * 0.017 - moon * 0.052;
    grading.global.tint = moon * 0.010;
    grading.global.post_saturation = 1. - moon * 0.19;

    let distance = camera_offset(adventure.area).length();
    let (haze, near, far) = match sky {
        Sky::Sun => (Color::srgba(0.79, 0.91, 0.84, 0.24), 7., 54.),
        Sky::Mist => (Color::srgba(0.78, 0.86, 0.80, 0.82), -10., 28.),
        Sky::Rain => (Color::srgba(0.55, 0.67, 0.74, 0.51), -4., 42.),
        Sky::Snow => (Color::srgba(0.79, 0.87, 0.91, 0.57), -6., 35.),
    };
    fog.color = if outdoor {
        haze.mix(&Color::srgba(0.19, 0.29, 0.46, 0.53), dusk)
    } else {
        Color::NONE
    };
    fog.falloff = FogFalloff::Linear {
        start: distance + near,
        end: distance + far,
    };

    let glow = ((dusk - 0.43) / 0.57).clamp(0., 1.);
    for mut visibility in &mut glows {
        set_visible(&mut visibility, glow > 0.025);
    }
    if glow > 0.025 {
        let flicker = 0.94 + (time.elapsed_secs() * 2.3).sin() * 0.04;
        if let Some(lantern) = materials.get_mut(&lights.lantern) {
            lantern.base_color = Color::srgba(1., 0.62, 0.25, glow * flicker * 0.44);
        }
        if let Some(source) = gltfs.get(&lights.windows) {
            for handle in &source.materials {
                let alpha = glow * 0.94;
                if materials.get(handle).is_some_and(|material| {
                    material.fog_enabled || (material.base_color.alpha() - alpha).abs() > 0.007
                }) && let Some(window) = materials.get_mut(handle)
                {
                    window.base_color = Color::linear_rgba(3.2, 2.72, 2.14, alpha);
                    window.fog_enabled = false;
                }
            }
        }
    }
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

    #[test]
    fn precipitation_hits_the_cabin_and_uses_the_same_wind_as_the_mill() {
        let cabin = Vec2::new(-2.65, -2.28);
        assert!(terrain::weather_cover(cabin) > 4.);
        let open = Vec2::new(2.6, 4.2);
        assert!(terrain::weather_cover(open) < 0.38);
        assert!(wind(Sky::Rain, 3.).length() > wind(Sky::Snow, 3.).length());
        assert!(wind(Sky::Snow, 3.).length() > wind(Sky::Sun, 3.).length());
    }
}
