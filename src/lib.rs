mod adventure;
mod battle_ui;
mod controller;
mod day;
mod duel;
mod farm_ui;
mod furniture;
mod garden;
mod homestead;
mod input;
mod processing;
mod progression;
mod regions;
mod season_view;
mod seasons;
mod session;
mod story;
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
    fertilize: bool,
    task: bool,
    inventory: bool,
    menu: bool,
}

#[derive(Resource, Default)]
struct GardenFocus {
    selected: Option<usize>,
    pending: Option<(Action, garden::Crop)>,
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
    .init_resource::<farm_ui::FarmMenu>()
    .init_resource::<Adventure>()
    .init_resource::<Ready>()
    .init_resource::<Controls>()
    .init_resource::<controller::Controller>()
    .init_resource::<GardenFocus>()
    .init_resource::<Notice>()
    .init_resource::<input::Gesture>()
    .init_resource::<ui::HudLayout>()
    .init_resource::<world::Models>()
    .init_resource::<ui::Icons>()
    .init_resource::<ui::UiFonts>()
    .init_resource::<battle_ui::DeckIcons>()
    .add_systems(
        Startup,
        (
            world::setup,
            season_view::setup,
            ui::setup,
            weather::setup,
            session::setup,
            town::setup,
            farm_ui::setup,
            battle_ui::setup,
        ),
    )
    .add_systems(
        Update,
        (
            ui::resize,
            session::input,
            town::input,
            farm_ui::input,
            battle_ui::input,
            input::collect,
            session::open_menu,
        )
            .chain(),
    )
    .add_systems(
        Update,
        (
            day::tick,
            select_plot,
            adventure::select,
            world::advance_player,
            act,
            adventure::interact,
            adventure::draw,
            (
                world::update_plots,
                world::show_exterior,
                season_view::visibility,
                season_view::ground,
                world::update_selection,
                world::animate_gardener,
                world::watering_effect,
                world::move_clouds,
                world::turn_mill,
                world::update_camera,
                weather::precipitation,
                weather::lighting,
                adventure::animate_animals,
                ui::update,
                ui::update_fertilizer,
                ui::map_labels.after(world::update_camera),
            ),
        )
            .chain()
            .after(session::open_menu)
            .run_if(session::playing),
    )
    .add_systems(
        Update,
        (
            session::update,
            town::update,
            farm_ui::update,
            battle_ui::update,
            battle_ui::animate_guardian,
            ui::sync_visibility,
        )
            .after(adventure::interact),
    )
    .add_systems(PostUpdate, session::autosave);
    #[cfg(target_os = "android")]
    app.add_systems(Startup, controller::setup).add_systems(
        PreUpdate,
        (controller::poll, controller::hide_after_touch)
            .chain()
            .before(bevy::input::InputSystem),
    );
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
    if controls.move_axis != Vec2::ZERO
        && focus
            .selected
            .is_some_and(|index| PLOTS[index].distance(player.position) > 5.)
    {
        focus.selected = None;
        focus.pending = None;
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
    focus.pending = None;
}

fn act(
    controls: Res<Controls>,
    adventure: Res<Adventure>,
    mut focus: ResMut<GardenFocus>,
    mut player: Single<(&mut Gardener, &mut Transform)>,
    time: Res<Time>,
    mut notice: ResMut<Notice>,
    mut garden: ResMut<Garden>,
    path: Res<SavePath>,
    mut farm_menu: ResMut<farm_ui::FarmMenu>,
) {
    if controls.move_axis != Vec2::ZERO
        || adventure.area != Area::Island
        || adventure.focus.is_some()
    {
        focus.pending = None;
        return;
    }
    if !controls.act && !controls.fertilize && focus.pending.is_none() {
        return;
    }
    let (player, transform) = &mut *player;
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
        return;
    };
    let now = time.elapsed_secs();
    if controls.act
        && !garden.clock.night()
        && focus.pending.is_none()
        && garden.plots[index].stage() == garden::Stage::Soil
        && garden.seed_count(adventure.crop) == 0
    {
        farm_menu.show(farm_ui::FarmPage::Seeds);
        return;
    }
    let requested = if controls.fertilize {
        if garden.fertilizer == 0 && garden.fertilizable(index) {
            *notice = Notice {
                index: Some(index),
                message: "Dünger gibt es am Saatgutstand.".into(),
                until: now + Notice::DURATION,
                ..default()
            };
        }
        (garden.fertilizable(index) && garden.fertilizer > 0).then_some(Action::Fertilize)
    } else if controls.act {
        garden.action(index, adventure.crop)
    } else {
        None
    };
    if garden.clock.night() {
        focus.pending = None;
        player.stop();
        return;
    }
    let Some((action, crop)) = requested
        .map(|action| (action, adventure.crop))
        .or(focus.pending)
    else {
        return;
    };
    if !world::can_work(player.position, PLOTS[index], &adventure, &garden) {
        if requested.is_some() {
            focus.pending = player
                .navigate(world::Reach::ground(PLOTS[index]), &adventure, &garden)
                .then_some((action, crop));
        }
        if !player.approaching() {
            focus.pending = None;
            *notice = Notice {
                index: Some(index),
                message: "Von hier aus komme ich gerade nicht ans Beet.".into(),
                until: now + Notice::DURATION,
                ..default()
            };
        }
        return;
    }
    focus.pending = None;
    if let Some(message) = garden.work(index, crop, action) {
        let position = player.position;
        player.stop();
        let direction = PLOTS[index] - position;
        if direction.length_squared() > 0.01 {
            transform.rotation = Quat::from_rotation_y(direction.x.atan2(direction.y));
        }
        adventure::commit(
            &garden,
            &path,
            &mut notice,
            now,
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
