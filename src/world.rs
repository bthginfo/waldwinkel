use std::f32::consts::TAU;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, VecDeque},
};

use bevy::{
    core_pipeline::{fxaa::Fxaa, tonemapping::Tonemapping},
    prelude::*,
    render::camera::ScalingMode,
    scene::SceneInstanceReady,
    window::PrimaryWindow,
};

use crate::{
    Controls, GardenFocus, Notice, Ready,
    adventure::{Adventure, Area},
    garden::{Action, Crop, Garden, Stage},
    terrain,
    town::Npc,
    travel::District,
};

pub const SPAWN: Vec2 = Vec2::new(-0.62, 3.47);
pub const PLOTS: [Vec2; 12] = [
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
];
const REACH: f32 = 1.30;
const SPEED: f32 = 2.75;
const ANKLE: f32 = 0.224;
const CAMERA_TARGET: Vec3 = Vec3::new(0.28, 1.24, 0.03);
const CAMERA_AT: Vec3 = Vec3::new(-8.50, 17.50, 22.80);

#[derive(Component)]
pub struct IslandCamera;

#[derive(Component)]
pub struct Exterior;

#[derive(Component)]
pub struct Gardener {
    pub position: Vec2,
    velocity: Vec2,
    route: VecDeque<Vec2>,
    phase: f32,
    gait: f32,
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
pub struct Selection;

#[derive(Component)]
pub struct PlotModel {
    index: usize,
    stage: Option<(Stage, Crop, bool)>,
    base: Entity,
    plant: Entity,
}

#[derive(Resource)]
pub struct Models {
    exterior: Vec<(Vec2, Handle<Scene>)>,
    frame: Handle<Scene>,
    lawn: Handle<Scene>,
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
    hoe: Handle<Scene>,
    clouds: [Handle<Scene>; 3],
    selection: Handle<Scene>,
    shadow: Handle<Scene>,
}

impl FromWorld for Models {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let get = |name: &str| assets.load(format!("models/{name}.glb#Scene0"));
        Self {
            exterior: [(Vec2::ZERO, get("island")), (Vec2::ZERO, get("mainland"))]
                .into_iter()
                .chain(District::ALL.map(|district| (district.origin(), get(district.model()))))
                .collect(),
            frame: get("bed"),
            lawn: get("lawn"),
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
            hoe: get("hoe"),
            clouds: [get("cloud_1"), get("cloud_2"), get("cloud_3")],
            selection: get("selection"),
            shadow: get("shadow"),
        }
    }
}

pub fn setup(mut commands: Commands, models: Res<Models>, assets: Res<AssetServer>) {
    commands.spawn((
        IslandCamera,
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
        Msaa::Off,
        Fxaa::default(),
        Transform::from_translation(CAMERA_TARGET + camera_offset(Area::Island))
            .looking_at(CAMERA_TARGET, Vec3::Y),
    ));
    let scenery_count = models.exterior.len();
    for (origin, model) in &models.exterior {
        commands
            .spawn((
                Exterior,
                SceneRoot(model.clone()),
                Transform::from_translation(position3(*origin, 0.)),
            ))
            .observe(
                move |_: Trigger<SceneInstanceReady>, mut ready: ResMut<Ready>| {
                    ready.1 += 1;
                    ready.0 = ready.1 >= scenery_count;
                },
            );
    }
    for (i, person) in Npc::ALL.into_iter().enumerate() {
        let point = person.position();
        commands.spawn((
            Exterior,
            SceneRoot(assets.load(format!("models/{}.glb#Scene0", person.model()))),
            Transform::from_translation(position3(point, exterior_ground(point) + 0.04))
                .with_rotation(Quat::from_rotation_y(-0.48 + i as f32 * 0.13)),
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
        commands
            .entity(parent)
            .add_children(&[frame, base, plant])
            .insert(PlotModel {
                index,
                stage: None,
                base,
                plant,
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
            Gardener {
                position: SPAWN,
                velocity: Vec2::ZERO,
                route: VecDeque::new(),
                phase: 0.0,
                gait: 0.0,
            },
            Transform::from_translation(position3(SPAWN, 0.0))
                .with_rotation(Quat::from_rotation_y(-0.22)),
            Visibility::Inherited,
        ))
        .with_children(|player| {
            player.spawn((Body, SceneRoot(models.body.clone())));
            player.spawn((
                Tail,
                SceneRoot(models.tail.clone()),
                Transform::from_xyz(0., 0.58, -0.18),
            ));
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
                    if is_arm && side > 0.0 {
                        limb.with_child((
                            SceneRoot(models.hoe.clone()),
                            Transform::from_xyz(0.007, -0.263, 0.042),
                        ));
                    } else if !is_arm {
                        limb.with_child((
                            Boot(side),
                            SceneRoot(models.boot.clone()),
                            Transform::from_xyz(0.0, -ANKLE, 0.006),
                        ));
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

pub fn click_plot(camera: &Camera, transform: &GlobalTransform, screen: Vec2) -> Option<usize> {
    let ray = camera.viewport_to_world(transform, screen).ok()?;
    let t = (0.18 - ray.origin.y) / ray.direction.y;
    if t < 0.0 {
        return None;
    }
    let hit = (ray.origin + ray.direction * t).xz();
    PLOTS
        .iter()
        .position(|position| (hit - position).abs().cmple(Vec2::splat(0.755)).all())
}

pub fn show_exterior(
    adventure: Res<Adventure>,
    mut entities: Query<&mut Visibility, With<Exterior>>,
    mut previous: Local<Option<bool>>,
) {
    let visible = adventure.area.outdoors();
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

pub fn camera_offset(area: Area) -> Vec3 {
    (CAMERA_AT - CAMERA_TARGET) * if area.outdoors() { 5.25 } else { 1. }
}

pub fn update_camera(
    window: Single<&Window, With<PrimaryWindow>>,
    mut camera: Single<(&mut Projection, &mut Transform), With<IslandCamera>>,
    player: Single<&Gardener>,
    adventure: Res<Adventure>,
    time: Res<Time>,
    mut last: Local<Option<Area>>,
) {
    let (projection, transform) = &mut *camera;
    let Projection::Orthographic(ortho) = &mut **projection else {
        return;
    };
    let (base, center) = match (adventure.area, adventure.map) {
        (Area::Island | Area::District(_), true) => (108., Vec3::new(-4., 0.75, 15.5)),
        (Area::Island | Area::District(_), false) => (12.3, position3(player.position, 0.89)),
        (Area::Home, _) => (8.55, position3(adventure.area.origin(), 0.95)),
        (Area::Mine(_), true) => (21., position3(adventure.area.origin(), 0.38)),
        (Area::Mine(_), false) => (9.4, position3(player.position, 0.40)),
    };
    let height = if adventure.map {
        base
    } else {
        base * (-adventure.zoom).exp()
    };
    let width = if adventure.map && adventure.area.outdoors() {
        179.0
    } else {
        height * 1.30
    };
    let desired = center + camera_offset(adventure.area);
    let smooth = (time.delta_secs() * 8.).min(1.);
    let current = match ortho.scaling_mode {
        ScalingMode::AutoMin { min_height, .. } => min_height,
        _ => height,
    };
    let applied = if *last != Some(adventure.area) {
        height
    } else {
        current + (height - current) * smooth
    };
    ortho.scaling_mode = ScalingMode::AutoMin {
        min_width: if window.width() > 0. {
            width * applied / height
        } else {
            width
        },
        min_height: applied,
    };
    transform.translation = if *last != Some(adventure.area) {
        desired
    } else {
        transform.translation.lerp(desired, smooth)
    };
    *last = Some(adventure.area);
}

pub fn update_plots(
    mut commands: Commands,
    garden: Res<Garden>,
    models: Res<Models>,
    mut plots: Query<&mut PlotModel>,
) {
    for mut plot in &mut plots {
        let state = garden.plots[plot.index];
        let stage = state.stage();
        let wet = state.remaining().is_some_and(|(_, wet)| wet);
        if plot.stage == Some((stage, state.crop(), wet)) {
            continue;
        }
        let bare = stage == Stage::Grass;
        if plot.stage.is_none()
            || plot
                .stage
                .is_some_and(|(previous, _, was_wet)| previous == Stage::Grass || was_wet != wet)
            || bare
        {
            replace_model(
                &mut commands,
                plot.base,
                Some(if bare {
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
        plot.stage = Some((stage, state.crop(), wet));
    }
}

fn replace_model(commands: &mut Commands, mount: Entity, scene: Option<&Handle<Scene>>) {
    let mut mount = commands.entity(mount);
    mount.despawn_related::<Children>();
    if let Some(scene) = scene {
        mount.with_child(SceneRoot(scene.clone()));
    }
}

pub fn update_selection(
    focus: Res<GardenFocus>,
    mut selection: Single<(&mut Visibility, &mut Transform), With<Selection>>,
) {
    let (visibility, transform) = &mut *selection;
    if let Some(index) = focus.selected {
        transform.translation = position3(PLOTS[index], 0.032);
        **visibility = Visibility::Inherited;
    } else {
        **visibility = Visibility::Hidden;
    }
}

pub fn anticipate_walk(
    mut focus: ResMut<GardenFocus>,
    mut player: Single<&mut Gardener>,
    adventure: Res<Adventure>,
) {
    if let Some(index) = focus.approach.take() {
        player.navigate(PLOTS[index], &adventure);
    }
}

pub fn advance_player(
    time: Res<Time>,
    controls: Res<Controls>,
    adventure: Res<Adventure>,
    camera: Single<&Transform, (With<IslandCamera>, Without<Gardener>)>,
    mut player: Single<(&mut Gardener, &mut Transform)>,
) {
    let (player, transform) = &mut *player;
    let delta = time.delta_secs().min(0.06);
    if delta == 0.0 {
        return;
    }
    let desired = if controls.move_axis.length_squared() > 0.006 {
        player.route.clear();
        let right = camera.right().xz().normalize_or_zero();
        let forward = camera.forward().xz().normalize_or_zero();
        (right * controls.move_axis.x + forward * controls.move_axis.y) * SPEED
    } else {
        while player
            .route
            .front()
            .is_some_and(|point| point.distance(player.position) < 0.035)
        {
            player.route.pop_front();
        }
        player.route.front().map_or(Vec2::ZERO, |point| {
            let diff = *point - player.position;
            diff.normalize_or_zero() * SPEED.min(diff.length() / delta)
        })
    };
    player.velocity = if player.route.is_empty() {
        player.velocity.lerp(desired, (delta * 15.0).min(1.0))
    } else {
        desired
    };
    let old = player.position;
    let step = player.velocity * delta;
    let mut moved = old + step;
    if !adventure.walkable(moved) {
        moved = Vec2::new(old.x + step.x, old.y);
        if !adventure.walkable(moved) {
            moved = Vec2::new(old.x, old.y + step.y);
            if !adventure.walkable(moved) {
                moved = old;
            }
        }
    }
    player.position = moved;
    let distance = old.distance(moved);
    player.phase = (player.phase + distance / 0.92).fract();
    player.gait = player.gait.lerp(
        (distance / (SPEED * delta)).clamp(0.0, 1.0),
        (delta * 11.0).min(1.0),
    );
    if distance > 0.0004 {
        let direction = moved - old;
        let angle = direction.x.atan2(direction.y);
        transform.rotation = transform
            .rotation
            .slerp(Quat::from_rotation_y(angle), (delta * 11.5).min(1.0));
    }
    let height = transform.translation.y.lerp(
        match adventure.area {
            Area::Island | Area::District(_) => exterior_ground(moved),
            _ => 0.0,
        },
        (delta * 16.0).min(1.0),
    );
    transform.translation = position3(moved, height);
}

pub fn animate_gardener(
    time: Res<Time>,
    notice: Res<Notice>,
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
        tail.rotation = Quat::from_rotation_y(
            wave * 0.25 * player.gait + 0.08 * (time.elapsed_secs() * 2.).sin(),
        );
    }
    let work_time = Notice::DURATION + time.elapsed_secs() - notice.until;
    let work = if notice.action.is_some() && (0.0..0.55).contains(&work_time) {
        (work_time / 0.55 * std::f32::consts::PI).sin() * (1.0 - player.gait)
    } else {
        0.0
    };
    body.translation.y = 0.023 * wave.abs() * player.gait
        + 0.009 * (time.elapsed_secs() * 2.0).sin() * (1.0 - player.gait);
    body.rotation =
        Quat::from_rotation_x(0.11 * work) * Quat::from_rotation_z(0.035 * wave * player.gait);
    for (joint, mut transform) in &mut joints {
        let phase = step_phase(player.phase, joint.side);
        let pitch = if joint.is_arm {
            let reach = if notice.action == Some(Action::Till) && joint.side > 0.0 {
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
            if joint.is_arm { joint.side * 0.11 } else { 0.0 },
        );
    }
    for (boot, mut transform) in &mut boots {
        let (swing, _) = foot_pose(step_phase(player.phase, boot.0));
        transform.rotation = Quat::from_rotation_x(swing * 0.66 * player.gait);
    }
    shadow.translation = root.translation + Vec3::new(0.026, 0.022, 0.04);
}

fn step_phase(phase: f32, side: f32) -> f32 {
    (phase + if side < 0.0 { 0.5 } else { 0.0 }).fract()
}

fn foot_pose(phase: f32) -> (f32, f32) {
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

pub fn move_clouds(time: Res<Time>, mut clouds: Query<(&Cloud, &mut Transform)>) {
    for (cloud, mut transform) in &mut clouds {
        let phase = time.elapsed_secs() * 0.17 + cloud.phase;
        transform.translation =
            cloud.origin + Vec3::new(phase.sin() * 0.53, phase.cos() * 0.10, 0.0);
    }
}

fn position3(point: Vec2, height: f32) -> Vec3 {
    Vec3::new(point.x, height, point.y)
}

pub fn exterior_ground(p: Vec2) -> f32 {
    if on_dock(p) && p.y >= 15.86 {
        let step = ((p.y - 16.0) / 0.27).round().max(0.0);
        return 0.17 - (step - 5.0).max(0.0) * 0.043;
    }
    if (-2.77..=-1.39).contains(&p.x) && (0.58..1.28).contains(&p.y) {
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
    terrain::ground(p)
}

fn rect(p: Vec2, x: (f32, f32), z: (f32, f32)) -> bool {
    (x.0..=x.1).contains(&p.x) && (z.0..=z.1).contains(&p.y)
}

fn on_dock(p: Vec2) -> bool {
    rect(p, (4.97, 6.63), (15.10, 19.48))
}

pub fn walkable(p: Vec2) -> bool {
    if on_dock(p) {
        return true;
    }
    if !terrain::walkable(p)
        || rect(p, (-4.93, -0.37), (-4.05, -0.53))
        || rect(p, (-5.22, -0.08), (-0.55, 0.80)) && !(-2.77..=-1.39).contains(&p.x)
        || ((p - Vec2::new(-4.77, 2.54)) / Vec2::new(1.66, 1.18)).length_squared() < 1.0
        || rect(p, (0.79, 5.99), (-0.72, -0.19))
        || rect(p, (5.73, 6.17), (-0.35, 1.95))
    {
        return false;
    }
    const TRUNKS_AND_ROCKS: &[(f32, f32, f32)] = &[
        (-6.39, -2.23, 0.53),
        (-5.45, -3.92, 0.46),
        (-3.46, -4.92, 0.57),
        (-1.32, -5.22, 0.58),
        (0.43, -4.89, 0.49),
        (2.02, -4.35, 0.55),
        (3.86, -3.51, 0.51),
        (5.25, -2.49, 0.49),
        (6.48, -1.48, 0.43),
        (6.72, 1.74, 0.33),
        (-6.74, -0.36, 0.39),
        (-5.36, 0.06, 0.62),
        (-6.88, -1.06, 0.43),
        (-5.27, -3.42, 0.61),
        (1.84, -3.71, 0.59),
        (0.16, -3.04, 0.41),
        (4.72, -2.78, 0.43),
        (5.8, -0.90, 0.50),
    ];
    !TRUNKS_AND_ROCKS
        .iter()
        .any(|&(x, z, r)| p.distance_squared(Vec2::new(x, z)) < r * r)
}

fn clear_walk(a: Vec2, b: Vec2, passable: &impl Fn(Vec2) -> bool) -> bool {
    let steps = (a.distance(b) / 0.12).ceil().max(1.) as usize;
    (1..=steps).all(|i| passable(a.lerp(b, i as f32 / steps as f32)))
}

pub fn can_work(player: Vec2, plot: Vec2) -> bool {
    player.distance_squared(plot) < REACH * REACH && clear_walk(player, plot, &walkable)
}

impl Gardener {
    pub fn teleport(&mut self, p: Vec2) {
        self.position = p;
        self.velocity = Vec2::ZERO;
        self.route.clear();
    }

    pub fn navigate(&mut self, p: Vec2, adventure: &Adventure) {
        self.route = approach(self.position, p, adventure.area.origin(), |p| {
            adventure.walkable(p)
        });
    }
}

fn approach(
    start: Vec2,
    target: Vec2,
    origin: Vec2,
    passable: impl Fn(Vec2) -> bool,
) -> VecDeque<Vec2> {
    let near =
        |p: Vec2| p.distance_squared(target) < 1.23f32.powi(2) && clear_walk(p, target, &passable);
    if near(start) {
        return VecDeque::new();
    }
    let direct = target + (start - target).normalize_or_zero() * 0.94;
    if near(direct) && clear_walk(start, direct, &passable) {
        return VecDeque::from([direct]);
    }
    if !passable(target) {
        return VecDeque::new();
    }
    const STEP: f32 = 0.32;
    const DIRECTIONS: [IVec2; 8] = [
        IVec2::X,
        IVec2::NEG_X,
        IVec2::Y,
        IVec2::NEG_Y,
        IVec2::ONE,
        IVec2::NEG_ONE,
        IVec2::new(1, -1),
        IVec2::new(-1, 1),
    ];
    let position = |cell: IVec2| origin + cell.as_vec2() * STEP;
    let cell = |p: Vec2| ((p - origin) / STEP).round().as_ivec2();
    let base = cell(start);
    let goal = cell(target);
    let first = (-2..=2)
        .flat_map(|x| (-2..=2).map(move |z| base + IVec2::new(x, z)))
        .filter(|&c| clear_walk(start, position(c), &passable))
        .min_by(|&a, &b| {
            start
                .distance_squared(position(a))
                .total_cmp(&start.distance_squared(position(b)))
        });
    let Some(first) = first else {
        return VecDeque::new();
    };
    let estimate = |c: IVec2| {
        let delta = (goal - c).abs();
        let diagonal = delta.min_element() as u32;
        let straight = (delta.max_element() - delta.min_element()) as u32;
        (diagonal * 1414 + straight * 1000).saturating_sub(3850)
    };
    let h = estimate(first);
    let mut open = BinaryHeap::from([Reverse((h, h, 0, first.x, first.y))]);
    let mut visited = HashMap::from([(first, (0u32, first))]);
    while let Some(Reverse((_, _, cost, x, z))) = open.pop() {
        let at = IVec2::new(x, z);
        if visited[&at].0 != cost {
            continue;
        }
        let from = position(at);
        if near(from) {
            let mut trail = vec![from];
            let mut cursor = at;
            while cursor != first {
                cursor = visited[&cursor].1;
                trail.push(position(cursor));
            }
            trail.reverse();
            let mut route = VecDeque::new();
            let mut next = 0;
            let mut anchor = start;
            while next < trail.len() {
                let last = (next + 28).min(trail.len() - 1);
                let until = (next..=last)
                    .rev()
                    .find(|&i| clear_walk(anchor, trail[i], &passable))
                    .unwrap_or(next);
                anchor = trail[until];
                route.push_back(anchor);
                next = until + 1;
            }
            return route;
        }
        for direction in DIRECTIONS {
            let next = at + direction;
            let weight = if direction.x != 0 && direction.y != 0 {
                1414
            } else {
                1000
            };
            let cost = cost + weight;
            if visited.get(&next).is_some_and(|(best, _)| *best <= cost)
                || !clear_walk(from, position(next), &passable)
            {
                continue;
            }
            visited.insert(next, (cost, at));
            let h = estimate(next);
            open.push(Reverse((cost + h, h, cost, next.x, next.y)));
        }
    }
    VecDeque::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tap_can_route_around_the_cabin_and_fence() {
        let start = Vec2::new(-5.65, -2.65);
        let target = PLOTS[1];
        assert!(walkable(start));
        assert!(!clear_walk(start, target, &walkable));
        let route = approach(start, target, Vec2::ZERO, walkable);
        assert!(!route.is_empty());
        let mut from = start;
        for point in route {
            assert!(clear_walk(from, point, &walkable));
            from = point;
        }
        assert!(can_work(from, target));
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
            let route = approach(pair[0], pair[1], Vec2::ZERO, walkable);
            assert!(!route.is_empty(), "{} to {}", pair[0], pair[1]);
            let mut previous = pair[0];
            for point in route {
                assert!(clear_walk(previous, point, &walkable));
                previous = point;
            }
            assert!(previous.distance(pair[1]) < 1.3);
        }
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
