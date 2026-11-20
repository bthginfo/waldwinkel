mod adventure;
mod garden;
mod homestead;
mod input;
mod session;
mod terrain;
mod town;
mod travel;
mod ui;
mod weather;
mod world;

use adventure::{Adventure, Area};
use bevy::{asset::AssetMetaCheck, log::LogPlugin, prelude::*, window::PresentMode};

use garden::{Action, Garden, SavePath};
use world::{Gardener, IslandCamera, PLOTS};

#[derive(Resource, Default)]
struct Ready(bool, usize);

/// Platform bindings fill these frame inputs. Garden and movement systems consume them.
#[derive(Resource, Default)]
struct Controls {
    move_axis: Vec2,
    pick_at: Option<Vec2>,
    cycle: bool,
    act: bool,
    zoom: f32,
    map: bool,
    choice: bool,
    task: bool,
    inventory: bool,
    menu: bool,
}

#[derive(Resource, Default)]
struct GardenFocus {
    selected: Option<usize>,
    approach: Option<usize>,
}

#[derive(Resource, Default)]
struct Notice {
    index: Option<usize>,
    message: String,
    action: Option<Action>,
    until: f32,
}

impl Notice {
    const DURATION: f32 = 2.6;
}

#[bevy_main]
pub fn main() {
    let path = SavePath::for_device();
    let slots = session::SaveSlots::new(path.0.clone());
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                filter: "wgpu=error,bevy_render=info,waldwinkel=info".to_owned(),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Waldwinkel".into(),
                    resolution: (1600.0f32, 900.0f32).into(),
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }),
    )
    .insert_resource(ClearColor(Color::srgb_u8(132, 204, 201)))
    .init_resource::<Garden>()
    .insert_resource(path)
    .insert_resource(slots)
    .init_resource::<session::Session>()
    .init_resource::<town::Dialog>()
    .init_resource::<Adventure>()
    .init_resource::<Ready>()
    .init_resource::<Controls>()
    .init_resource::<GardenFocus>()
    .init_resource::<Notice>()
    .init_resource::<input::Gesture>()
    .init_resource::<ui::HudLayout>()
    .init_resource::<world::Models>()
    .init_resource::<ui::Icons>()
    .init_resource::<ui::UiFonts>()
    .add_systems(
        Startup,
        (
            world::setup,
            ui::setup,
            weather::setup,
            session::setup,
            town::setup,
        ),
    )
    .add_systems(
        Update,
        (
            ui::resize,
            session::input,
            town::input,
            input::collect,
            session::open_menu,
        )
            .chain(),
    )
    .add_systems(
        Update,
        (
            select_plot,
            adventure::select,
            world::anticipate_walk,
            world::advance_player,
            act,
            adventure::interact,
            adventure::draw,
            (
                world::update_plots,
                world::show_exterior,
                world::update_selection,
                world::animate_gardener,
                world::move_clouds,
                world::update_camera,
                weather::update.after(world::update_camera),
                adventure::animate_animals,
                ui::update,
                ui::map_labels.after(world::update_camera),
            ),
        )
            .chain()
            .after(session::open_menu)
            .run_if(session::playing),
    )
    .add_systems(
        Update,
        (session::update, town::update, ui::sync_visibility).after(adventure::interact),
    )
    .add_systems(PostUpdate, session::autosave);
    #[cfg(not(target_os = "android"))]
    app.add_systems(Update, capture);
    app.run();
}

fn select_plot(
    controls: Res<Controls>,
    adventure: Res<Adventure>,
    camera: Single<(&Camera, &GlobalTransform), With<IslandCamera>>,
    mut focus: ResMut<GardenFocus>,
    player: Single<&Gardener>,
) {
    if adventure.area != Area::Island {
        return;
    }
    if controls.move_axis.length_squared() > 0.006
        && focus
            .selected
            .is_some_and(|index| PLOTS[index].distance(player.position) > 5.)
    {
        focus.selected = None;
        focus.approach = None;
    }
    let (camera, transform) = *camera;
    let next = if controls.cycle && player.position.length() < 16. {
        Some(
            focus
                .selected
                .map_or(0, |previous| (previous + 1) % PLOTS.len()),
        )
    } else if let Some(at) = controls.pick_at {
        world::click_plot(camera, transform, at)
    } else {
        return;
    };
    focus.selected = next;
    focus.approach = next;
}

fn act(
    controls: Res<Controls>,
    adventure: Res<Adventure>,
    mut focus: ResMut<GardenFocus>,
    player: Single<&Gardener>,
    time: Res<Time>,
    mut notice: ResMut<Notice>,
    mut garden: ResMut<Garden>,
    path: Res<SavePath>,
) {
    if !controls.act || adventure.area != Area::Island || adventure.focus.is_some() {
        return;
    }
    let Some(index) = focus.selected else {
        let closest = PLOTS
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.distance_squared(player.position)
                    .total_cmp(&b.distance_squared(player.position))
            })
            .filter(|(_, point)| point.distance(player.position) < 4.5)
            .map(|(i, _)| i);
        focus.selected = closest;
        focus.approach = closest;
        return;
    };
    if !world::can_work(player.position, PLOTS[index]) {
        focus.approach = Some(index);
        return;
    }
    if let Some((action, message)) = garden.work(index, adventure.crop) {
        adventure::commit(
            &garden,
            &path,
            &mut notice,
            time.elapsed_secs(),
            message,
            Some(index),
            Some(action),
        );
    }
}

#[cfg(not(target_os = "android"))]
fn capture(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<Entity, With<bevy::window::PrimaryWindow>>,
    mut count: Local<u32>,
) {
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    if !keys.just_pressed(KeyCode::F12) {
        return;
    }
    let dir = std::env::var_os("WALDWINKEL_SCREEN_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("build/screenshots"));
    if let Err(error) = std::fs::create_dir_all(&dir) {
        warn!("Screenshot-Ordner konnte nicht angelegt werden: {error}");
        return;
    }
    *count += 1;
    commands
        .spawn(Screenshot::window(*window))
        .observe(save_to_disk(
            dir.join(format!("waldwinkel-{:02}.png", *count)),
        ));
}
