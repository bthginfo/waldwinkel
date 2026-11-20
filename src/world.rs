use std::f32::consts::TAU;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, VecDeque},
    time::{Duration, Instant},
};

use bevy::{
    core_pipeline::{fxaa::Fxaa, tonemapping::Tonemapping},
    prelude::*,
    render::{camera::ScalingMode, view::ColorGrading},
    scene::SceneInstanceReady,
    window::PrimaryWindow,
};

use crate::{
    Controls, GardenFocus, Notice, Ready,
    adventure::{Adventure, Area},
    duel::Gear,
    garden::{Action, Crop, Garden, Stage},
    resident::{self, Resident},
    season_view::SeasonLayer,
    terrain,
    town::Npc,
    travel::District,
    weather::{self, Season, Sky},
};

pub const SPAWN: Vec2 = Vec2::new(-0.62, 3.47);
pub const PLOTS: [Vec2; crate::garden::PLOT_COUNT] = [
    Vec2::new(1.59, 0.48),
    Vec2::new(3.15, 0.48),
    Vec2::new(4.71, 0.48),
    Vec2::new(1.59, 2.10),
    Vec2::new(3.15, 2.10),
    Vec2::new(4.71, 2.10),
    Vec2::new(6.78, 3.90),
    Vec2::new(8.34, 3.90),
    Vec2::new(9.90, 3.90),
    Vec2::new(6.78, 5.52),
    Vec2::new(8.34, 5.52),
    Vec2::new(9.90, 5.52),
    Vec2::new(1.55, 10.48),
    Vec2::new(3.11, 10.48),
    Vec2::new(4.67, 10.48),
    Vec2::new(1.55, 12.10),
    Vec2::new(3.11, 12.10),
    Vec2::new(4.67, 12.10),
];
const REACH: f32 = 1.30;
const SPEED: f32 = 4.80;
const ACCELERATION: f32 = 36.;
const BRAKING: f32 = 64.;
const STEERING: f32 = 56.;
const MAX_STEP: f32 = 1.0 / 120.0;
const ROUTE_TIME_PER_FRAME: Duration = Duration::from_millis(3);
const ANKLE: f32 = 0.224;
const CAMERA_TARGET: Vec3 = Vec3::new(0.28, 1.24, 0.03);
const CAMERA_AT: Vec3 = Vec3::new(-8.50, 17.50, 22.80);

#[derive(Component)]
pub struct IslandCamera;

#[derive(Component, Default)]
pub struct ViewFrame {
    pub focus: Vec3,
    pub distance: f32,
    pub width: f32,
    height: f32,
}

#[derive(Component)]
pub struct Exterior;

#[derive(Component)]
pub struct Gardener {
    pub position: Vec2,
    velocity: Vec2,
    route: VecDeque<Vec2>,
    route_goal: Option<Reach>,
    route_search: Option<RouteSearch>,
    phase: f32,
    gait: f32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Hoe,
    Axe,
    Pickaxe,
    WateringCan,
    Rod,
    Weapon(Gear),
}

#[derive(Component)]
pub struct Joint {
    top: Vec3,
    side: f32,
    is_arm: bool,
}

#[derive(Component)]
pub struct Body;

#[derive(Component)]
pub struct Boot(f32);

#[derive(Component)]
pub struct Tail;

#[derive(Component)]
pub struct ContactShadow;

#[derive(Component)]
pub struct Cloud {
    origin: Vec3,
    phase: f32,
}

#[derive(Component)]
pub struct MillSails;

#[derive(Component)]
pub struct Selection;

#[derive(Event)]
pub struct Watered(pub usize);

#[derive(Component)]
pub struct PlotModel {
    index: usize,
    stage: Option<(Stage, Crop, bool, bool)>,
    base: Entity,
    plant: Entity,
    soil_amendment: Entity,
    fertilized: bool,
}

#[derive(Resource)]
pub struct Models {
    exterior: Vec<(Vec2, Handle<Scene>)>,
    seasonal: [Handle<Scene>; Season::ALL.len()],
    frame: Handle<Scene>,
    lawn: Handle<Scene>,
    winter_lawn: Handle<Scene>,
    soil: Handle<Scene>,
    wet_soil: Handle<Scene>,
    sprout: Handle<Scene>,
    young: Handle<Scene>,
    mature: [Handle<Scene>; 6],
    tail: Handle<Scene>,
    body: Handle<Scene>,
    arm: Handle<Scene>,
    leg: Handle<Scene>,
    boot: Handle<Scene>,
    tools: [(Tool, Handle<Scene>); 9],
    fertilized_soil: Handle<Scene>,
    water_drop: Handle<Scene>,
    clouds: [Handle<Scene>; 3],
    selection: Handle<Scene>,
    shadow: Handle<Scene>,
}

impl FromWorld for Models {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let get = |name: &str| assets.load(format!("models/{name}.glb#Scene0"));
        Self {
            exterior: [
                (Vec2::ZERO, get("island")),
                (Vec2::ZERO, get("mainland")),
                (Vec2::ZERO, get("world_ground")),
                (Vec2::ZERO, get("island_relief")),
            ]
            .into_iter()
            .chain(District::ALL.map(|district| (district.origin(), get(district.model()))))
            .collect(),
            seasonal: Season::ALL.map(|season| get(&format!("season_{}", season.slug()))),
            frame: get("bed"),
            lawn: get("lawn"),
            winter_lawn: get("lawn_winter"),
            soil: get("soil"),
            wet_soil: get("wet_soil"),
            sprout: get("sprout"),
            young: get("young"),
            mature: [
                get("carrot"),
                get("turnip"),
                get("pumpkin"),
                get("crop_strawberry"),
                get("crop_tomato"),
                get("crop_sunflower"),
            ],
            tail: get("tail"),
            body: get("gardener"),
            arm: get("arm"),
            leg: get("leg"),
            boot: get("boot"),
            tools: [
                (Tool::Hoe, get("hoe")),
                (Tool::Axe, get("axe")),
                (Tool::Pickaxe, get("pickaxe")),
                (Tool::WateringCan, get("watering_can")),
                (Tool::Rod, get("fishing_rod")),
                (Tool::Weapon(Gear::TrailStick), get("gear_staff")),
                (Tool::Weapon(Gear::CopperSickle), get("gear_sickle")),
                (Tool::Weapon(Gear::SilverGuard), get("gear_silver")),
                (Tool::Weapon(Gear::CrystalBlade), get("gear_crystal")),
            ],
            fertilized_soil: get("fertilized_soil"),
            water_drop: get("water_drop"),
            clouds: [get("cloud_1"), get("cloud_2"), get("cloud_3")],
            selection: get("selection"),
            shadow: get("shadow"),
        }
    }
}

pub fn setup(
    mut commands: Commands,
    models: Res<Models>,
    assets: Res<AssetServer>,
    garden: Res<Garden>,
) {
    let mill = Vec2::new(16., 122.8);
    commands.spawn((
        MillSails,
        Exterior,
        SceneRoot(assets.load("models/windmill_sails.glb#Scene0")),
        Transform::from_xyz(mill.x, exterior_ground(mill) + 4.94, mill.y + 1.51),
    ));
    commands.spawn((
        IslandCamera,
        ViewFrame::default(),
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 22.9,
                min_height: 13.63,
            },
            near: 0.01,
            far: 385.0,
            ..OrthographicProjection::default_3d()
        }),
        Tonemapping::None,
        ColorGrading::default(),
        DistanceFog {
            color: Color::NONE,
            ..default()
        },
        Msaa::Off,
        Fxaa::default(),
        Transform::from_translation(CAMERA_TARGET + camera_offset(Area::Island))
            .looking_at(CAMERA_TARGET, Vec3::Y),
    ));
    let scenery_count = models.exterior.len() + models.seasonal.len();
    let scene_ready = move |_: Trigger<SceneInstanceReady>, mut ready: ResMut<Ready>| {
        ready.1 += 1;
        ready.0 = ready.1 >= scenery_count;
    };
    for (origin, model) in &models.exterior {
        commands
            .spawn((
                Exterior,
                SceneRoot(model.clone()),
                Transform::from_translation(position3(*origin, 0.)),
            ))
            .observe(scene_ready)
            .observe(crate::water::bind);
    }
    for (season, model) in Season::ALL.into_iter().zip(&models.seasonal) {
        commands
            .spawn((
                SeasonLayer(season),
                SceneRoot(model.clone()),
                if season == Season::Spring {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
            ))
            .observe(scene_ready);
    }
    for (i, person) in Npc::ALL.into_iter().enumerate() {
        let point = person.position(&garden);
        commands
            .spawn((
                Exterior,
                Resident(person),
                SceneRoot(assets.load(format!("models/{}.glb#Scene0", person.model()))),
                Transform::from_translation(position3(point, exterior_ground(point) + 0.04))
                    .with_rotation(Quat::from_rotation_y(-0.48 + i as f32 * 0.13)),
            ))
            .observe(resident::bind_limbs)
            .observe(crate::figure::bind)
            .with_child((
                SceneRoot(models.shadow.clone()),
                Transform::from_xyz(0., -0.010, 0.03).with_scale(Vec3::new(0.63, 1., 0.67)),
            ));
    }
    for (index, position) in PLOTS.into_iter().enumerate() {
        let parent = commands
            .spawn((
                Name::new(format!("Beet {}", index + 1)),
                Transform::from_translation(position3(position, 0.0)),
                Visibility::Inherited,
            ))
            .id();
        let frame = commands.spawn(SceneRoot(models.frame.clone())).id();
        let base = commands
            .spawn((Transform::default(), Visibility::Inherited))
            .id();
        let plant = commands
            .spawn((Transform::default(), Visibility::Inherited))
            .id();
        let soil_amendment = commands
            .spawn((Transform::default(), Visibility::Inherited))
            .id();
        commands
            .entity(parent)
            .add_children(&[frame, base, plant, soil_amendment])
            .insert(PlotModel {
                index,
                stage: None,
                base,
                plant,
                soil_amendment,
                fertilized: false,
            });
    }

    commands.spawn((
        ContactShadow,
        SceneRoot(models.shadow.clone()),
        Transform::from_translation(position3(SPAWN, 0.018)).with_scale(Vec3::new(0.46, 1.0, 0.35)),
    ));
    commands
        .spawn((
            Name::new("Gärtnerfigur"),
            Gardener::new(SPAWN),
            Transform::from_translation(position3(SPAWN, 0.0))
                .with_rotation(Quat::from_rotation_y(-0.22)),
            Visibility::Inherited,
        ))
        .with_children(|player| {
            player
                .spawn((Body, SceneRoot(models.body.clone())))
                .observe(crate::figure::bind);
            player
                .spawn((
                    Tail,
                    SceneRoot(models.tail.clone()),
                    Transform::from_xyz(0., 0.58, -0.18),
                ))
                .observe(crate::figure::bind);
            for side in [-1.0, 1.0] {
                for (top, model, is_arm) in [
                    (Vec3::new(side * 0.244, 0.813, 0.0), &models.arm, true),
                    (Vec3::new(side * 0.112, 0.356, 0.008), &models.leg, false),
                ] {
                    let mut limb = player.spawn((
                        Joint { top, side, is_arm },
                        Transform::from_translation(top),
                        SceneRoot(model.clone()),
                    ));
                    limb.observe(crate::figure::bind);
                    if is_arm && side > 0.0 {
                        limb.with_children(|hand| {
                            for (tool, scene) in &models.tools {
                                hand.spawn((
                                    *tool,
                                    SceneRoot(scene.clone()),
                                    Transform::from_xyz(0.007, -0.263, 0.042),
                                    if *tool == Tool::Hoe {
                                        Visibility::Inherited
                                    } else {
                                        Visibility::Hidden
                                    },
                                ));
                            }
                        });
                    } else if !is_arm {
                        limb.with_children(|foot| {
                            foot.spawn((
                                Boot(side),
                                SceneRoot(models.boot.clone()),
                                Transform::from_xyz(0.0, -ANKLE, 0.006),
                            ))
                            .observe(crate::figure::bind);
                        });
                    }
                }
            }
        });
    commands.spawn((
        Selection,
        Visibility::Hidden,
        SceneRoot(models.selection.clone()),
    ));
    for (index, (origin, scale)) in [
        (Vec3::new(-8.28, 3.72, -6.64), 1.0),
        (Vec3::new(11.95, 2.62, -4.03), 1.12),
        (Vec3::new(13.10, 2.21, -2.99), 0.68),
    ]
    .into_iter()
    .enumerate()
    {
        commands.spawn((
            Cloud {
                origin,
                phase: index as f32 * 1.4,
            },
            SceneRoot(models.clouds[index].clone()),
            Transform::from_translation(origin).with_scale(Vec3::splat(scale)),
        ));
    }
}

pub fn ground_point(ray: Ray3d, area: Area, garden: &Garden) -> Option<Vec2> {
    let plane = InfinitePlane3d::new(Dir3::NEG_Y);
    if !area.outdoors() {
        return Some(ray.get_point(ray.intersect_plane(Vec3::ZERO, plane)?).xz());
    }
    let first = ray.intersect_plane(Vec3::Y * 34., plane)?;
    let last = ray.intersect_plane(Vec3::NEG_Y * 2., plane)?;
    let below = |distance| {
        let p = ray.get_point(distance);
        p.y <= surface_height(p.xz(), garden)
    };
    let mut previous = first;
    for i in 1..=38 {
        let distance = first.lerp(last, i as f32 / 38.);
        if below(distance) {
            let mut top = previous;
            let mut bottom = distance;
            for _ in 0..6 {
                let midpoint = (top + bottom) / 2.;
                if below(midpoint) {
                    bottom = midpoint;
                } else {
                    top = midpoint;
                }
            }
            return Some(ray.get_point(bottom).xz());
        }
        previous = distance;
    }
    None
}

pub fn show_exterior(
    adventure: Res<Adventure>,
    menu: Res<crate::farm_ui::FarmMenu>,
    mut entities: Query<&mut Visibility, (With<Exterior>, Without<Gardener>, Without<PlotModel>)>,
    mut hero: Single<&mut Visibility, (With<Gardener>, Without<Exterior>)>,
    mut previous: Local<Option<bool>>,
) {
    let preview = menu.room_preview().is_some();
    hero.set_if_neq(if preview {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    });
    let visible = adventure.area.outdoors() && !preview;
    if *previous == Some(visible) {
        return;
    }
    *previous = Some(visible);
    for mut visibility in &mut entities {
        *visibility = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn camera_offset(area: Area) -> Vec3 {
    (CAMERA_AT - CAMERA_TARGET) * if area.outdoors() { 10.5 } else { 1. }
}

pub fn update_camera(
    window: Single<&Window, With<PrimaryWindow>>,
    mut camera: Single<(&mut Projection, &mut Transform, &mut ViewFrame), With<IslandCamera>>,
    player: Single<&Gardener>,
    adventure: Res<Adventure>,
    garden: Res<Garden>,
    menu: Res<crate::farm_ui::FarmMenu>,
    layout: Res<crate::decorate::Layout>,
    time: Res<Time>,
    mut last: Local<Option<(Area, bool, bool)>>,
) {
    let (projection, transform, frame) = &mut *camera;
    if layout.open() {
        let (point, height, width) = if adventure.area.outdoors() {
            (Vec2::new(1.5, 5.5), 24., 30.)
        } else {
            let east = if garden.workshop.house == 1 { 1.5 } else { 0. };
            (adventure.area.origin() + Vec2::X * east, 9.3, 16.)
        };
        frame.focus = position3(point, 0.);
        frame.distance = 30.;
        frame.height = height;
        frame.width = width;
        transform.translation = frame.focus + Vec3::Y * frame.distance;
        transform.look_at(frame.focus, Vec3::NEG_Z);
        **projection = Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin { min_width: width, min_height: height },
            far: 120.,
            ..OrthographicProjection::default_3d()
        });
        *last = None;
        return;
    }
    let preview = menu.room_preview();
    let area = if preview.is_some() {
        Area::Home
    } else {
        adventure.area
    };
    let map = preview.is_none() && adventure.map;
    let clip = menu.clip().map(|(clip, elapsed)| clip.framing(elapsed));
    let town = area.outdoors() && !map && District::at(player.position) == Some(District::Town);
    let (base, center) = match (area, map) {
        (Area::Island | Area::District(_), true) => (258., Vec3::new(0., 3., 54.)),
        (Area::Island | Area::District(_), false) => {
            let eye = surface_height(player.position, &garden) + if town { 1.26 } else { 0.89 };
            (12.3, position3(player.position, eye))
        }
        (Area::Home, _) => {
            let (height, east) = match garden.workshop.house {
                0 => (8.55, 0.),
                1 => (10.6, 1.65),
                _ => (11.7, 0.),
            };
            (
                height,
                position3(
                    preview.map_or(area.origin() + Vec2::X * east, |view| {
                        let point =
                            adventure.position(crate::adventure::Target::Slot(view.slot), &garden);
                        area.origin().lerp(point, 0.70)
                    }),
                    if preview.is_some() { 0.55 } else { 0.95 },
                ),
            )
        }
        (Area::Mine(_) | Area::Expedition(_), true) => {
            (23., position3(adventure.area.origin(), 0.38))
        }
        (Area::Mine(_) | Area::Expedition(_), false) => (9.4, position3(player.position, 0.40)),
    };
    let (base, center) = clip.map_or((base, center), |(center, height)| (height, center));
    let height = if map || preview.is_some() || clip.is_some() {
        base
    } else {
        base * (-adventure.zoom).exp()
    };
    let width = if map && area.outdoors() {
        448.0
    } else {
        height * 1.30
    };
    let aspect = window.width().max(1.) / window.height().max(1.);
    let perspective = area.outdoors() && !map && clip.is_none();
    let offset = if perspective {
        approach_camera(player.position, center.y, height, adventure.zoom, town)
    } else {
        camera_offset(area)
    };
    let desired = center + offset;
    let smooth = 1. - (-time.delta_secs() * if map { 11. } else { 15. }).exp();
    let shot = (area, map, clip.is_some());
    if *last != Some(shot) {
        frame.focus = center;
        frame.height = height;
        transform.translation = desired;
    } else {
        frame.focus = frame.focus.lerp(center, smooth);
        frame.height = frame.height.lerp(height, smooth);
        transform.translation = transform.translation.lerp(desired, smooth);
    }
    transform.look_at(frame.focus, Vec3::Y);
    frame.distance = transform.translation.distance(frame.focus);
    let min_width = width * frame.height / height;
    frame.width = min_width.max(frame.height * aspect);
    if perspective {
        let fov = 2. * (frame.width / aspect / (2. * frame.distance)).atan();
        let view = PerspectiveProjection {
            fov,
            aspect_ratio: aspect,
            near: 0.08,
            far: 520.,
        };
        if let Projection::Perspective(current) = &mut **projection {
            *current = view;
        } else {
            **projection = Projection::Perspective(view);
        }
    } else {
        let scaling_mode = ScalingMode::AutoMin {
            min_width,
            min_height: frame.height,
        };
        let far = if map && area.outdoors() { 775. } else { 385. };
        if let Projection::Orthographic(current) = &mut **projection {
            current.scaling_mode = scaling_mode;
            current.far = far;
        } else {
            **projection = Projection::Orthographic(OrthographicProjection {
                scaling_mode,
                far,
                near: 0.01,
                ..OrthographicProjection::default_3d()
            });
        }
    }
    *last = Some(shot);
}

pub(crate) fn approach_camera(point: Vec2, eye: f32, height: f32, zoom: f32, town: bool) -> Vec3 {
    let close = zoom.clamp(0., 1.);
    let mut aim = CAMERA_AT - CAMERA_TARGET;
    let pitch = if town {
        0.79 - close * 0.22
    } else {
        0.96 - close * 0.24
    };
    let toward = aim.xz().normalize();
    let rise = aim.y * pitch / aim.xz().length();
    let cover = [1.5, 2.5, 3.6, 5.1, 7.0]
        .into_iter()
        .fold(0_f32, |blocked, dist| {
            let sample = point + toward * dist;
            let surface = terrain::ground(sample).max(terrain::weather_cover(sample));
            blocked.max(surface - eye - rise * dist - 0.24)
        });
    // Have the camera clear nearby ground or overhanging foliage at close zoom.
    aim.y *= pitch.lerp(1.19, (cover * 0.61).clamp(0., 1.));
    let distance =
        (47. + height * 0.38).lerp(7.8 + height * 0.66, close * close * (3. - 2. * close));
    aim.normalize() * distance
}

pub fn update_plots(
    mut commands: Commands,
    garden: Res<Garden>,
    adventure: Res<Adventure>,
    menu: Res<crate::farm_ui::FarmMenu>,
    models: Res<Models>,
    mut plots: Query<(&mut PlotModel, &mut Visibility)>,
) {
    let winter = Season::at(garden.day) == Season::Winter;
    for (mut plot, mut visibility) in &mut plots {
        let active = plot.index < garden.active_beds();
        visibility.set_if_neq(if active && adventure.area.outdoors() && menu.room_preview().is_none() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        if !active { continue; }
        let fertilized = garden.fertilized[plot.index];
        if plot.fertilized != fertilized {
            replace_model(
                &mut commands,
                plot.soil_amendment,
                fertilized.then_some(&models.fertilized_soil),
            );
            plot.fertilized = fertilized;
        }
        let state = garden.plots[plot.index];
        let stage = state.stage();
        let wet = state.remaining().is_some_and(|(_, wet)| wet);
        let snow = stage == Stage::Grass && winter;
        if plot.stage == Some((stage, state.crop(), wet, snow)) {
            continue;
        }
        let bare = stage == Stage::Grass;
        if plot.stage.is_none()
            || plot
                .stage
                .is_some_and(|(previous, _, was_wet, _)| previous == Stage::Grass || was_wet != wet)
            || bare
        {
            replace_model(
                &mut commands,
                plot.base,
                Some(if snow {
                    &models.winter_lawn
                } else if bare {
                    &models.lawn
                } else if wet {
                    &models.wet_soil
                } else {
                    &models.soil
                }),
            );
        }
        let plant = match stage {
            Stage::Sprout => Some(&models.sprout),
            Stage::Young => Some(&models.young),
            Stage::Ripe => Some(&models.mature[state.crop() as usize]),
            _ => None,
        };
        replace_model(&mut commands, plot.plant, plant);
        plot.stage = Some((stage, state.crop(), wet, snow));
    }
}

pub(crate) fn replace_model(commands: &mut Commands, mount: Entity, scene: Option<&Handle<Scene>>) {
    let mut mount = commands.entity(mount);
    mount.despawn_related::<Children>();
    if let Some(scene) = scene {
        mount.with_child(SceneRoot(scene.clone()));
    }
}

pub fn update_selection(
    focus: Res<GardenFocus>,
    adventure: Res<Adventure>,
    garden: Res<Garden>,
    mut selection: Single<(&mut Visibility, &mut Transform), With<Selection>>,
) {
    let (visibility, transform) = &mut *selection;
    let lesson = crate::tutorial::suggested_plot(&garden);
    let selection = focus.selected.map(|index| (PLOTS[index], 1.)).or_else(|| {
        adventure
            .focus
            .map(|entry| (adventure.position(entry, &garden), entry.selection_scale()))
    }).or_else(|| {
        (adventure.area == Area::Island && !garden.clock.night())
            .then_some(lesson)
            .flatten()
            .map(|i| (PLOTS[i], 1.))
    });
    if let Some((point, scale)) = selection.filter(|_| !adventure.map) {
        let ground = if adventure.area.outdoors() {
            surface_height(point, &garden).max(0.)
        } else {
            0.
        };
        let raised = lesson.is_some_and(|index| PLOTS[index] == point);
        transform.translation = position3(point, ground + if raised { 0.15 } else { 0.052 });
        transform.scale = Vec3::new(scale, 1., scale);
        visibility.set_if_neq(Visibility::Inherited);
    } else {
        visibility.set_if_neq(Visibility::Hidden);
    }
}

fn drive_velocity(current: Vec2, desired: Vec2, dt: f32) -> Vec2 {
    let reversing = desired.dot(current) < 0.;
    let adjusting = desired.length_squared() < current.length_squared()
        || current.normalize_or_zero().dot(desired.normalize_or_zero()) < 0.65;
    let rate = if desired == Vec2::ZERO || reversing {
        BRAKING
    } else if adjusting && current != Vec2::ZERO {
        STEERING
    } else {
        ACCELERATION
    };
    current + (desired - current).clamp_length_max(rate * dt)
}

pub fn advance_player(
    time: Res<Time>,
    controls: Res<Controls>,
    adventure: Res<Adventure>,
    garden: Res<Garden>,
    camera: Single<&Transform, (With<IslandCamera>, Without<Gardener>)>,
    mut player: Single<(&mut Gardener, &mut Transform)>,
) {
    let (player, transform) = &mut *player;
    // Bound movement after a long rendering stall or a resume from the system menu.
    let delta = time.delta_secs().min(0.075);
    if delta == 0.0 {
        return;
    }
    let manual = controls.move_axis != Vec2::ZERO;
    let desired = if manual {
        player.route.clear();
        player.route_goal = None;
        player.route_search = None;
        let right = camera.right().xz().normalize_or_zero();
        let forward = camera.forward().xz().normalize_or_zero();
        (right * controls.move_axis.x + forward * controls.move_axis.y).clamp_length_max(1.) * SPEED
    } else {
        Vec2::ZERO
    };
    let old = adventure.step_away_from_resident(player.position, &garden);
    player.position = old;
    let steps = (delta / MAX_STEP).ceil() as u32;
    let step_time = delta / steps as f32;
    let passable = |p| adventure.walkable(p, &garden);
    let deadline = Instant::now() + ROUTE_TIME_PER_FRAME;
    if let Some(search) = &mut player.route_search
        && let Some(route) = search.poll(&passable, || Instant::now() < deadline)
    {
        player.route_search = None;
        player.route = route;
        if player.route.is_empty() {
            player.route_goal = None;
        }
    }
    for _ in 0..steps {
        if let Some((&corner, &next)) = player.route.front().zip(player.route.get(1))
            && player.route_goal.is_some()
            && corner.distance_squared(player.position) < 0.55 * 0.55
            && clear_walk(player.position, next, &passable)
        {
            player.route.pop_front();
        }
        while player
            .route
            .front()
            .is_some_and(|point| point.distance_squared(player.position) < 0.0001)
        {
            player.route.pop_front();
            if player.route.is_empty() {
                player.velocity = Vec2::ZERO;
                player.route_goal = None;
            }
        }
        if let Some(point) = player.route.front() {
            let diff = *point - player.position;
            let distance = diff.length();
            let limit = if player.route.len() == 1 {
                (2.0 * BRAKING * distance).sqrt().min(SPEED)
            } else {
                SPEED
            };
            let speed = (player.velocity.length() + ACCELERATION * step_time)
                .min(limit)
                .min(distance / step_time);
            player.velocity = diff.normalize_or_zero() * speed;
        } else {
            player.velocity = drive_velocity(player.velocity, desired, step_time);
        }
        let advance = player.velocity * step_time;
        let moved = slide(player.position, advance, &passable);
        let actual = moved - player.position;
        if (actual - advance).length_squared() > 1e-8 {
            player.velocity = actual / step_time;
            if actual == Vec2::ZERO && !manual {
                player.route.clear();
                if let Some(reach) = player.route_goal.take() {
                    player.route_search = Some(RouteSearch::new(
                        player.position,
                        reach,
                        adventure.area.origin(),
                        &passable,
                        false,
                    ));
                }
                break;
            }
        }
        player.position = moved;
    }
    let moved = player.position;
    let distance = old.distance(moved);
    let speed = (distance / (SPEED * delta)).clamp(0., 1.);
    let stride = 0.49 + 0.43 * speed;
    player.phase = (player.phase + distance / stride).fract();
    player.gait = player.gait.lerp(speed, 1.0 - (-delta * 13.0).exp());
    if distance > 0.0004 {
        let direction = moved - old;
        let angle = direction.x.atan2(direction.y);
        transform.rotation = transform
            .rotation
            .slerp(Quat::from_rotation_y(angle), 1.0 - (-delta * 16.0).exp());
    }
    let height = transform.translation.y.lerp(
        match adventure.area {
            Area::Island | Area::District(_) => surface_height(moved, &garden),
            _ => 0.0,
        },
        1.0 - (-delta * 20.0).exp(),
    );
    transform.translation = position3(moved, height);
}

fn slide(point: Vec2, step: Vec2, passable: &impl Fn(Vec2) -> bool) -> Vec2 {
    let next = point + step;
    if passable(next) {
        return next;
    }
    let x = Vec2::new(next.x, point.y);
    let y = Vec2::new(point.x, next.y);
    let axes = if step.x.abs() >= step.y.abs() {
        [x, y]
    } else {
        [y, x]
    };
    axes.into_iter()
        .find(|&candidate| candidate != point && passable(candidate))
        .unwrap_or(point)
}

pub fn animate_gardener(
    time: Res<Time>,
    notice: Res<Notice>,
    adventure: Res<Adventure>,
    focus: Res<GardenFocus>,
    garden: Res<Garden>,
    mut tools: Query<(&Tool, &mut Visibility)>,
    player: Single<(&Gardener, &Transform)>,
    mut joints: Query<
        (&Joint, &mut Transform),
        (
            Without<Gardener>,
            Without<Body>,
            Without<ContactShadow>,
            Without<Boot>,
        ),
    >,
    mut boots: Query<
        (&Boot, &mut Transform),
        (
            Without<Gardener>,
            Without<Body>,
            Without<ContactShadow>,
            Without<Joint>,
        ),
    >,
    mut tails: Query<
        &mut Transform,
        (
            With<Tail>,
            Without<Body>,
            Without<Joint>,
            Without<Gardener>,
            Without<ContactShadow>,
            Without<Boot>,
        ),
    >,
    mut body: Single<&mut Transform, (With<Body>, Without<Gardener>, Without<ContactShadow>)>,
    mut shadow: Single<&mut Transform, (With<ContactShadow>, Without<Gardener>, Without<Body>)>,
) {
    let (player, root) = *player;
    let wave = (player.phase * TAU).sin();
    for mut tail in &mut tails {
        let breathing = (time.elapsed_secs() * 1.7).sin();
        tail.rotation = Quat::from_euler(
            EulerRot::YXZ,
            wave * 0.26 * player.gait + 0.082 * breathing,
            0.034 * breathing - (wave * 0.07 + 0.034) * player.gait,
            wave * 0.055 * player.gait,
        );
    }
    let now = time.elapsed_secs();
    let swing = adventure.swing(now);
    let work_time = Notice::DURATION + now - notice.until;
    let pouring = notice.action == Some(Action::Water) && (0.0..0.86).contains(&work_time);
    let harvest = notice.action == Some(Action::Harvest) && (0.0..0.64).contains(&work_time);
    let held = if matches!(adventure.area, Area::Expedition(_)) {
        Tool::Weapon(garden.rpg.gear)
    } else if let Some((tool, _)) = swing {
        tool
    } else if focus.tool == crate::fieldwork::FieldTool::Axe {
        Tool::Axe
    } else if focus.tool == crate::fieldwork::FieldTool::Pickaxe {
        Tool::Pickaxe
    } else if pouring
        || (adventure.area == Area::Island
            && adventure.focus.is_none()
            && (focus.tool == crate::fieldwork::FieldTool::Water
                || focus.selected.is_some_and(|index| {
                    focus.tool.action(&garden, index, adventure.crop) == Some(Action::Water)
                })))
    {
        Tool::WateringCan
    } else {
        adventure.held_tool()
    };
    for (&tool, mut visibility) in &mut tools {
        visibility.set_if_neq(if held == tool {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    let work = if pouring {
        (work_time / 0.86 * std::f32::consts::PI).sin().sqrt()
    } else if let Some((_, elapsed)) = swing {
        if elapsed < 0.23 {
            let lift = (elapsed / 0.23).clamp(0., 1.);
            lift * lift * (3. - 2. * lift)
        } else {
            let release = ((elapsed - 0.23) / 0.29).clamp(0., 1.);
            (1. - release).powi(2)
        }
    } else if harvest {
        (work_time / 0.64 * std::f32::consts::PI).sin() * (1. - player.gait)
    } else if notice.action.is_some() && (0.0..0.55).contains(&work_time) {
        (work_time / 0.55 * std::f32::consts::PI).sin() * (1.0 - player.gait)
    } else {
        0.0
    };
    body.translation.y = 0.027 * wave.abs() * player.gait
        + 0.009 * (time.elapsed_secs() * 2.0).sin() * (1.0 - player.gait);
    if harvest {
        body.translation.y -= 0.074 * work;
    }
    let lean = if harvest { 0.30 } else { 0.11 };
    body.rotation = Quat::from_rotation_x(lean * work + 0.045 * player.gait)
        * Quat::from_rotation_z(0.035 * wave * player.gait);
    for (joint, mut transform) in &mut joints {
        let phase = step_phase(player.phase, joint.side);
        let pitch = if joint.is_arm {
            let reach = if joint.side > 0.0 && swing.is_some() {
                1.65
            } else if (pouring || notice.action == Some(Action::Till)) && joint.side > 0.0 {
                1.12
            } else {
                0.62
            };
            (phase * TAU).cos() * 0.49 * player.gait - reach * work
        } else {
            let (swing, lift) = foot_pose(phase);
            let angle = -swing * 0.66 * player.gait;
            // The boot stays at ground height through its stance and lifts during its swing.
            transform.translation.y =
                joint.top.y - (1.0 - angle.cos()) * ANKLE + lift * player.gait;
            angle
        };
        transform.rotation = Quat::from_euler(
            EulerRot::XYZ,
            pitch,
            0.0,
            if joint.is_arm {
                joint.side * if pouring { 0.22 } else { 0.11 }
            } else {
                0.0
            },
        );
    }
    for (boot, mut transform) in &mut boots {
        let (swing, _) = foot_pose(step_phase(player.phase, boot.0));
        transform.rotation = Quat::from_rotation_x(swing * 0.66 * player.gait);
    }
    let outdoor = adventure.area.outdoors();
    let daylight = if outdoor {
        ((f32::from(garden.clock.minute()) - 360.0) / (16.0 * 60.0)).clamp(0., 1.)
    } else {
        0.5
    };
    let azimuth = -0.48 + daylight * 2.48;
    let direction = Vec2::from_angle(azimuth);
    let length = 0.34 + (1. - (std::f32::consts::PI * daylight).sin()) * 0.56;
    let softness = if !outdoor || garden.clock.night() || Sky::at(garden.day) != Sky::Sun {
        0.28
    } else {
        length
    };
    let point = root.translation.xz() + direction * softness * 0.19;
    let ground = if outdoor {
        let floor = surface_height(point, &garden);
        let top = terrain::weather_cover(point);
        if top < floor + 0.18 {
            floor.max(top)
        } else {
            floor
        }
    } else {
        0.
    };
    shadow.translation = position3(point, ground + 0.023);
    shadow.rotation = Quat::from_rotation_y(-azimuth);
    shadow.scale = Vec3::new(0.38 + softness * 0.40, 1.0, 0.32);
}

#[derive(Component)]
pub struct WaterDrop {
    from: Vec3,
    to: Vec3,
    start: f32,
}

pub fn watering_effect(
    mut commands: Commands,
    mut watered: EventReader<Watered>,
    time: Res<Time>,
    models: Res<Models>,
    player: Single<&Transform, With<Gardener>>,
    mut drops: Query<(Entity, &WaterDrop, &mut Transform, &mut Visibility), Without<Gardener>>,
) {
    let now = time.elapsed_secs();
    for &Watered(plot) in watered.read() {
        let from = player.translation + player.rotation * Vec3::new(0.24, 0.61, 0.39);
        for i in 0..12 {
            let angle = i as f32 * 2.39996;
            let to = position3(
                PLOTS[plot] + Vec2::from_angle(angle) * (i as f32 * 0.039),
                0.08,
            );
            commands.spawn((
                WaterDrop {
                    from,
                    to,
                    start: now + 0.13 + i as f32 * 0.025,
                },
                SceneRoot(models.water_drop.clone()),
                Visibility::Hidden,
                Transform::from_translation(from),
            ));
        }
    }
    for (entity, drop, mut transform, mut visible) in &mut drops {
        let fraction = (now - drop.start) / 0.40;
        if fraction >= 1.0 {
            commands.entity(entity).despawn();
        } else if fraction >= 0.0 {
            *visible = Visibility::Inherited;
            transform.translation = drop.from.lerp(drop.to, fraction);
            transform.translation.y += 0.26 * fraction * (1.0 - fraction);
        }
    }
}

pub(crate) fn step_phase(phase: f32, side: f32) -> f32 {
    (phase + if side < 0.0 { 0.5 } else { 0.0 }).fract()
}

pub(crate) fn foot_pose(phase: f32) -> (f32, f32) {
    if phase < 0.57 {
        (1.0 - 2.0 * phase / 0.57, 0.0)
    } else {
        let swing_phase = (phase - 0.57) / 0.43;
        (
            -(swing_phase * std::f32::consts::PI).cos(),
            (swing_phase * std::f32::consts::PI).sin() * 0.14,
        )
    }
}

pub fn move_clouds(
    time: Res<Time>,
    garden: Res<Garden>,
    mut clouds: Query<(&Cloud, &mut Transform)>,
) {
    let gust = weather::wind(Sky::at(garden.day), time.elapsed_secs());
    for (cloud, mut transform) in &mut clouds {
        let phase = time.elapsed_secs() * 0.17 + cloud.phase;
        transform.translation = cloud.origin
            + Vec3::new(
                phase.sin() * gust.x,
                phase.cos() * 0.10,
                phase.sin() * gust.y,
            );
    }
}

pub fn turn_mill(
    time: Res<Time>,
    garden: Res<Garden>,
    sail: Single<&mut Transform, With<MillSails>>,
) {
    let speed = weather::wind(Sky::at(garden.day), time.elapsed_secs()).length();
    sail.into_inner()
        .rotate_z((0.32 + speed * 0.6) * time.delta_secs());
}

fn position3(point: Vec2, height: f32) -> Vec3 {
    Vec3::new(point.x, height, point.y)
}

pub fn exterior_ground(p: Vec2) -> f32 {
    if on_dock(p) && p.y >= 15.86 {
        let step = ((p.y - 16.0) / 0.27).round().max(0.0);
        return 0.17 - (step - 5.0).max(0.0) * 0.043;
    }
    if (-2.77..=-1.39).contains(&p.x) && (-0.53..1.28).contains(&p.y) {
        return if p.y < 0.70 {
            0.47
        } else if p.y < 0.91 {
            0.36
        } else if p.y < 1.10 {
            0.25
        } else {
            0.145
        };
    }
    let local = p - District::Town.origin();
    let rim = (local / Vec2::new(10.72, 6.65)).length_squared();
    let paving = ((1.0 - rim) / 0.16).clamp(0., 1.)
        * ((local.y + 4.58) * 4.).clamp(0., 1.)
        * ((5.55 - local.y) * 4.).clamp(0., 1.);
    terrain::ground(p) + paving * 0.061
}

pub fn surface_height(point: Vec2, garden: &Garden) -> f32 {
    garden
        .harbor
        .phase
        .deck_height(point)
        .unwrap_or_else(|| exterior_ground(point))
}

fn rect(p: Vec2, x: (f32, f32), z: (f32, f32)) -> bool {
    (x.0..=x.1).contains(&p.x) && (z.0..=z.1).contains(&p.y)
}

pub fn on_dock(p: Vec2) -> bool {
    rect(p, (4.97, 6.63), (15.10, 19.48))
}

pub fn walkable(p: Vec2) -> bool {
    if on_dock(p) {
        return true;
    }
    terrain::walkable(p)
        && !rect(p, (-4.93, -0.37), (-4.05, -0.53))
        && (!rect(p, (-5.22, -0.08), (-0.55, 0.80)) || (-2.77..=-1.39).contains(&p.x))
        && ((p - Vec2::new(-4.77, 2.54)) / Vec2::new(1.66, 1.18)).length_squared() >= 1.0
}

fn clear_walk(a: Vec2, b: Vec2, passable: &impl Fn(Vec2) -> bool) -> bool {
    let distance = a.distance(b);
    // A saved position can lie beside a very thin collision boundary.
    // Check the first steps of a route as well as the regular samples.
    if [0.004, 0.012]
        .into_iter()
        .any(|step| step < distance && !passable(a.lerp(b, step / distance)))
    {
        return false;
    }
    let steps = (distance / 0.05).ceil().max(1.) as usize;
    (1..=steps).all(|i| passable(a.lerp(b, i as f32 / steps as f32)))
}

pub fn work_reach(plot: Vec2, action: Action, garden: &Garden) -> Reach {
    use crate::workshop::ToolKind;
    let tier = match action {
        Action::Till => ToolKind::Hoe.level(garden),
        Action::Water => ToolKind::Can.level(garden),
        _ => 0,
    };
    Reach::target(plot, 0., REACH + f32::from(tier) * 0.22)
}

#[derive(Clone, Copy)]
pub struct Reach {
    point: Vec2,
    clearance: f32,
    limit: f32,
}

impl Reach {
    pub fn walk(point: Vec2) -> Self {
        Self::target(point, 0., 0.30)
    }

    #[cfg(test)]
    pub fn ground(point: Vec2) -> Self {
        Self {
            point,
            clearance: 0.,
            limit: REACH,
        }
    }

    pub fn target(point: Vec2, clearance: f32, limit: f32) -> Self {
        Self {
            point,
            clearance,
            limit,
        }
    }

    pub fn reached(self, from: Vec2, passable: &impl Fn(Vec2) -> bool) -> bool {
        let offset = from - self.point;
        let distance = offset.length();
        distance >= self.clearance
            && distance < self.limit
            && clear_walk(
                from,
                self.point + offset.normalize_or_zero() * self.clearance,
                passable,
            )
    }
}

impl Gardener {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            velocity: Vec2::ZERO,
            route: VecDeque::new(),
            route_goal: None,
            route_search: None,
            phase: 0.,
            gait: 0.,
        }
    }

    pub fn teleport(&mut self, p: Vec2) {
        self.position = p;
        self.stop();
    }

    pub fn stop(&mut self) {
        self.velocity = Vec2::ZERO;
        self.route.clear();
        self.route_goal = None;
        self.route_search = None;
    }

    pub fn navigate(&mut self, reach: Reach, adventure: &Adventure, garden: &Garden) -> bool {
        self.route_search = None;
        self.route = approach(self.position, reach, adventure.area.origin(), |p| {
            adventure.walkable(p, garden)
        });
        self.route_goal = (!self.route.is_empty()).then_some(reach);
        !self.route.is_empty()
    }

    pub fn navigate_over_time(&mut self, reach: Reach, adventure: &Adventure, garden: &Garden) {
        self.route.clear();
        self.route_goal = Some(reach);
        self.route_search = Some(RouteSearch::new(
            self.position,
            reach,
            adventure.area.origin(),
            &|p| adventure.walkable(p, garden),
            true,
        ));
    }

    pub fn finding_route(&self) -> bool {
        self.route_search.is_some()
    }

    pub fn approaching(&self) -> bool {
        self.finding_route() || !self.route.is_empty()
    }

    pub(crate) fn destination_moved(&self, point: Vec2) -> bool {
        !self.finding_route()
            && self
                .route_goal
                .is_some_and(|goal| goal.point.distance_squared(point) > 1.35_f32.powi(2))
    }
}

fn approach(
    start: Vec2,
    reach: Reach,
    origin: Vec2,
    passable: impl Fn(Vec2) -> bool,
) -> VecDeque<Vec2> {
    RouteSearch::new(start, reach, origin, &passable, true)
        .poll(&passable, || true)
        .unwrap_or_default()
}

const ROUTE_GRID: f32 = 0.32;
const ROUTE_DIRECTIONS: [IVec2; 8] = [
    IVec2::X,
    IVec2::NEG_X,
    IVec2::Y,
    IVec2::NEG_Y,
    IVec2::ONE,
    IVec2::NEG_ONE,
    IVec2::new(1, -1),
    IVec2::new(-1, 1),
];

struct RouteSearch {
    start: Vec2,
    reach: Reach,
    origin: Vec2,
    shortcuts: bool,
    target_cell: IVec2,
    direct: Option<Vec2>,
    // Total estimated cost, remaining estimate, distance so far, and cell coordinates.
    open: BinaryHeap<Reverse<(u32, u32, u32, i32, i32)>>,
    visited: HashMap<IVec2, (u32, IVec2)>,
}

impl RouteSearch {
    fn new(
        start: Vec2,
        reach: Reach,
        origin: Vec2,
        passable: &impl Fn(Vec2) -> bool,
        shortcuts: bool,
    ) -> Self {
        let mut search = Self {
            start,
            reach,
            origin,
            shortcuts,
            target_cell: ((reach.point - origin) / ROUTE_GRID).round().as_ivec2(),
            direct: None,
            open: BinaryHeap::new(),
            visited: HashMap::new(),
        };
        if reach.reached(start, passable) {
            return search;
        }
        let radius = (reach.clearance + 0.22).max(0.94).min(reach.limit - 0.08);
        let direct = reach.point + (start - reach.point).normalize_or_zero() * radius;
        if shortcuts && search.near(direct, passable) && clear_walk(start, direct, passable) {
            search.direct = Some(direct);
            return search;
        }
        let reachable = if reach.clearance == 0. {
            passable(reach.point)
        } else {
            (0..24).any(|i| {
                let direction = Vec2::from_angle(i as f32 * TAU / 24.);
                search.near(reach.point + direction * (reach.limit - 0.06), passable)
            })
        };
        if !reachable {
            return search;
        }
        let base = ((start - origin) / ROUTE_GRID).round().as_ivec2();
        for cell in (-2..=2).flat_map(|x| (-2..=2).map(move |z| base + IVec2::new(x, z))) {
            let point = search.position(cell);
            if clear_walk(start, point, passable) {
                let cost = (start.distance(point) / ROUTE_GRID * 1000.).round() as u32;
                search.visit(cell, cell, cost);
            }
        }
        search
    }

    fn position(&self, cell: IVec2) -> Vec2 {
        self.origin + cell.as_vec2() * ROUTE_GRID
    }

    fn near(&self, point: Vec2, passable: &impl Fn(Vec2) -> bool) -> bool {
        // Stay inside interaction range if movement stops just short of the waypoint.
        let goal = Reach {
            limit: self.reach.limit - 0.025,
            ..self.reach
        };
        goal.reached(point, passable)
    }

    fn visit(&mut self, cell: IVec2, previous: IVec2, cost: u32) {
        let delta = (self.target_cell - cell).abs();
        let diagonal = delta.min_element() as u32;
        let straight = (delta.max_element() - delta.min_element()) as u32;
        let estimate = (diagonal * 1414 + straight * 1000)
            .saturating_sub((self.reach.limit / ROUTE_GRID * 1100.) as u32);
        self.visited.insert(cell, (cost, previous));
        self.open.push(Reverse((cost + estimate, estimate, cost, cell.x, cell.y)));
    }

    fn poll(
        &mut self,
        passable: &impl Fn(Vec2) -> bool,
        mut time_left: impl FnMut() -> bool,
    ) -> Option<VecDeque<Vec2>> {
        if let Some(point) = self.direct.take() {
            return Some(VecDeque::from([point]));
        }
        while time_left() {
            let Some(Reverse((_, _, cost, x, z))) = self.open.pop() else {
                return Some(VecDeque::new());
            };
            let at = IVec2::new(x, z);
            if self.visited[&at].0 != cost {
                continue;
            }
            let from = self.position(at);
            if self.near(from, passable) {
                return Some(self.trail(at, passable));
            }
            for direction in ROUTE_DIRECTIONS {
                let next = at + direction;
                let weight = if direction.x != 0 && direction.y != 0 {
                    1414
                } else {
                    1000
                };
                let cost = cost + weight;
                if self.visited.get(&next).is_some_and(|(best, _)| *best <= cost)
                    || !clear_walk(from, self.position(next), passable)
                {
                    continue;
                }
                self.visit(next, at, cost);
            }
        }
        None
    }

    fn trail(&self, goal: IVec2, passable: &impl Fn(Vec2) -> bool) -> VecDeque<Vec2> {
        let mut cursor = goal;
        let mut trail = vec![self.position(cursor)];
        while self.visited[&cursor].1 != cursor {
            cursor = self.visited[&cursor].1;
            trail.push(self.position(cursor));
        }
        trail.reverse();
        if !self.shortcuts {
            return trail.into();
        }
        let mut route = VecDeque::new();
        let mut next = 0;
        let mut anchor = self.start;
        while next < trail.len() {
            let last = (next + 28).min(trail.len() - 1);
            let until = (next..=last)
                .rev()
                .find(|&i| clear_walk(anchor, trail[i], passable))
                .unwrap_or(next);
            anchor = trail[until];
            route.push_back(anchor);
            next = until + 1;
        }
        route
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turning_starts_on_the_first_step_and_releasing_stops_within_a_tenth_second() {
        let step = 1. / 120.;
        let walking = Vec2::X * SPEED;
        let turning = drive_velocity(walking, Vec2::Y * SPEED, step);
        assert!(turning.x < walking.x && turning.y > 0.);
        let mut braking = walking;
        for _ in 0..12 {
            braking = drive_velocity(braking, Vec2::ZERO, step);
        }
        assert_eq!(braking, Vec2::ZERO);
    }

    #[test]
    fn the_three_passes_and_their_terrain_barriers_allow_only_the_marked_routes() {
        use crate::regions::Frontier;
        let open: Garden = serde_json::from_str(r#"{"regions":{"gates":[2,2,2]}}"#).unwrap();
        let sealed = Garden::default();
        let adventure = Adventure::default();
        for gate in Frontier::ALL {
            let exit = gate.entrance();
            let entrance = gate.position() * 2. - exit;
            assert!(adventure.walkable(entrance, &sealed), "{gate:?}");
            assert!(!adventure.walkable(exit, &sealed), "{gate:?}");
            let passable = |p| adventure.walkable(p, &open);
            let path = approach(entrance, Reach::walk(exit), Vec2::ZERO, passable);
            assert!(
                path.back().is_some_and(|pos| pos.distance(exit) < 0.30),
                "{gate:?}"
            );
        }
        for outside in [
            Vec2::new(78., -4.),
            Vec2::new(-87., 2.),
            Vec2::new(17., 76.),
        ] {
            assert!(!adventure.walkable(outside, &open), "{outside}");
        }
    }

    #[test]
    fn blocked_diagonal_preserves_the_longer_free_component() {
        let step = Vec2::new(0.015, 0.03);
        let obstacle = |p: Vec2| p.x == 0. || p.y == 0.;
        assert_eq!(slide(Vec2::ZERO, step, &obstacle), Vec2::new(0., 0.03));
        assert_eq!(
            slide(Vec2::ZERO, -step.yx(), &obstacle),
            Vec2::new(-0.03, 0.)
        );
    }

    #[test]
    fn tap_can_route_around_the_cabin_and_fence() {
        let start = Vec2::new(-5.65, -2.65);
        let target = PLOTS[1];
        assert!(walkable(start));
        assert!(!clear_walk(start, target, &walkable));
        let route = approach(start, Reach::ground(target), Vec2::ZERO, walkable);
        assert!(!route.is_empty());
        let mut from = start;
        for point in route {
            assert!(clear_walk(from, point, &walkable));
            from = point;
        }
        assert!(Reach::ground(target).reached(from, &walkable));
    }

    #[test]
    fn the_marked_first_wacht_is_reachable_from_the_harbor_walkway() {
        let exterior = Adventure::default();
        let start = Vec2::new(43.5054, 2.879982);
        let reach = Reach::target(crate::waywatch::Watch::Fern.point(), 2.08, 2.95);
        for minute in [434, 862, 1190] {
            let garden: Garden =
                serde_json::from_value(serde_json::json!({"clock": minute})).unwrap();
            let passable = |p| exterior.walkable(p, &garden);
            assert!(passable(start));
            let route = approach(start, reach, Vec2::ZERO, passable);
            assert!(
                route
                    .back()
                    .is_some_and(|point| reach.reached(*point, &passable)),
                "Minute {minute}"
            );
        }
    }

    #[test]
    fn actual_walking_reaches_the_first_wacht_from_the_harbor() {
        let garden = serde_json::from_str::<Garden>(r#"{"clock":510.0}"#).unwrap();
        let adventure = Adventure::default();
        let reach = Reach::target(crate::waywatch::Watch::Fern.point(), 2.08, 2.95);
        let mut gardener = Gardener::new(Vec2::new(43.5054, 2.879982));
        assert!(gardener.navigate(reach, &adventure, &garden));
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(Controls::default())
            .insert_resource(garden)
            .insert_resource(adventure)
            .add_systems(Update, advance_player);
        app.world_mut().spawn((IslandCamera, Transform::default()));
        let entity = app.world_mut().spawn((gardener, Transform::default())).id();
        for _ in 0..3000 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(16));
            app.update();
            if !app.world().get::<Gardener>(entity).unwrap().approaching() {
                break;
            }
        }
        let end = app.world().get::<Gardener>(entity).unwrap().position;
        assert!(
            reach.reached(end, &|point| app
                .world()
                .resource::<Adventure>()
                .walkable(point, app.world().resource::<Garden>())),
            "{end:?}"
        );
    }

    #[test]
    fn every_region_is_reachable_on_foot_in_the_same_area() {
        let landmarks = [
            SPAWN,
            District::Forest.waypoint(),
            District::Beach.waypoint(),
            District::Town.waypoint(),
            SPAWN,
        ];
        for pair in landmarks.windows(2) {
            let route = approach(pair[0], Reach::ground(pair[1]), Vec2::ZERO, walkable);
            assert!(!route.is_empty(), "{} to {}", pair[0], pair[1]);
            let mut previous = pair[0];
            for point in route {
                assert!(clear_walk(previous, point, &walkable));
                previous = point;
            }
            assert!(previous.distance(pair[1]) < REACH);
        }
    }

    #[test]
    fn the_outer_regions_connect_to_walkable_routes_from_the_old_world() {
        let garden: Garden = serde_json::from_str(r#"{"regions":{"gates":[2,2,2]}}"#).unwrap();
        let adventure = Adventure::default();
        let passable = |p| adventure.walkable(p, &garden);
        for (start, district) in [
            (District::Town.waypoint(), District::Copper),
            (District::Beach.waypoint(), District::Wind),
            (District::Forest.waypoint(), District::Moor),
        ] {
            let reach = Reach::ground(district.waypoint());
            let route = approach(start, reach, Vec2::ZERO, passable);
            let mut current = start;
            assert!(!route.is_empty(), "{district:?}");
            for point in route {
                assert!(clear_walk(current, point, &passable), "{district:?}");
                current = point;
            }
            assert!(reach.reached(current, &passable), "{district:?}");
        }
    }

    #[test]
    fn routes_end_at_the_reachable_side_of_an_obstacle() {
        let target = Vec2::ZERO;
        let reach = Reach::target(target, 0.9, 1.49);
        let wall = |p: Vec2| (-2.8..=-1.04).contains(&p.x) && p.y.abs() < 0.85;
        let passable = |p: Vec2| p.length_squared() >= 0.8_f32.powi(2) && !wall(p);
        let start = Vec2::new(-4., 0.);
        let route = approach(start, reach, Vec2::ZERO, passable);
        assert!(!route.is_empty());
        let mut from = start;
        for point in route {
            assert!(clear_walk(from, point, &passable));
            from = point;
        }
        assert!(reach.reached(from, &passable));
        assert!(!passable(target));
    }

    #[test]
    fn the_route_from_the_farm_to_edda_can_pause_and_finish() {
        let garden: Garden =
            serde_json::from_str(r#"{"regions":{"gates":[2,2,2]},"clock":618}"#).unwrap();
        let adventure = Adventure::default();
        let passable = |p| adventure.walkable(p, &garden);
        let reach = Reach::target(Npc::Edda.position(&garden), 0.50, 1.49);
        let mut search = RouteSearch::new(SPAWN, reach, Vec2::ZERO, &passable, true);
        let mut remaining = 12;
        assert!(
            search
                .poll(&passable, || {
                    remaining -= 1;
                    remaining > 0
                })
                .is_none()
        );
        let route = search.poll(&passable, || true).unwrap();
        let mut from = SPAWN;
        assert!(!route.is_empty());
        for point in route {
            assert!(clear_walk(from, point, &passable));
            from = point;
        }
        assert!(reach.reached(from, &passable));
    }

    #[test]
    fn stance_and_swing_meet_without_a_jump() {
        assert_eq!(foot_pose(0.), (1., 0.));
        assert_eq!(foot_pose(0.57), (-1., 0.));
        assert!(foot_pose(0.785).1 > 0.13);
        let end = foot_pose(1.);
        assert!((end.0 - 1.).abs() < 1e-4 && end.1.abs() < 1e-4);
    }
}
