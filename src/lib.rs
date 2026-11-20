mod adventure;
mod battle_ui;
mod building_plan;
mod campaign;
mod cinematic;
mod commissions;
mod controller;
mod craft;
mod day;
mod duel;
mod evening_market;
mod farm_ui;
mod fieldwork;
mod forest;
mod furniture;
mod garden;
mod harbor;
mod harvest_fx;
mod homestead;
mod input;
mod inventory;
#[cfg(target_os = "android")]
mod mobile;
mod onboarding;
mod processing;
mod progression;
mod regions;
mod resident;
mod resident_view;
mod season_view;
mod seasons;
mod session;
mod sound;
mod story;
mod terrain;
mod town;
mod travel;
mod ui;
mod waywatch;
mod weather;
mod wildlife;
mod workshop;
mod world;

use adventure::{Adventure, Area};
use bevy::{asset::AssetMetaCheck, log::LogPlugin, prelude::*, window::PresentMode};

use garden::{Action, Garden, SavePath};
use world::{Gardener, PLOTS};

#[derive(Resource, Default)]
struct Ready(bool, usize);

/// Platform bindings fill these frame inputs. Garden and movement systems consume them.
#[derive(Resource, Default)]
struct Controls {
    move_axis: Vec2,
    pick_at: Option<Vec2>,
    finger_pick: bool,
    cycle: bool,
    pad_cycle: i8,
    pad_action: bool,
    pad_quick: bool,
    cancel: bool,
    act: bool,
    zoom: f32,
    map: bool,
    choice: bool,
    fertilize: bool,
    field_tool: bool,
    task: bool,
    inventory: bool,
    menu: bool,
}

#[derive(Resource, Default)]
struct GardenFocus {
    selected: Option<usize>,
    pending: Option<(Action, garden::Crop)>,
    tool: fieldwork::FieldTool,
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
    let plugins = DefaultPlugins
        .set(LogPlugin {
            filter: "wgpu=error,naga=warn,bevy_render=info,waldwinkel=info".to_owned(),
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
        });
    #[cfg(target_os = "android")]
    let plugins = {
        use bevy::render::{
            RenderPlugin,
            settings::{RenderCreation, WgpuSettings, WgpuSettingsPriority},
        };
        plugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                priority: WgpuSettingsPriority::Compatibility,
                ..default()
            }),
            synchronous_pipeline_compilation: true,
            ..default()
        })
    };
    app.add_plugins(plugins);
    #[cfg(target_os = "android")]
    {
        use bevy::render::{
            RenderApp,
            batching::gpu_preprocessing::{GpuPreprocessingMode, GpuPreprocessingSupport},
        };
        // Set this before the render plugins finish so both the mesh batch
        // builder and the camera and shadow render phases use CPU preparation.
        app.sub_app_mut(RenderApp)
            .insert_resource(GpuPreprocessingSupport {
                max_supported_mode: GpuPreprocessingMode::None,
            });
    }
    app.insert_resource(ClearColor(Color::srgb_u8(132, 204, 201)))
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
        .init_resource::<resident_view::ResidentPins>()
        .init_resource::<ui::SafeInsets>()
        .init_resource::<world::Models>()
        .init_resource::<ui::Icons>()
        .init_resource::<ui::UiFonts>()
        .init_resource::<battle_ui::DeckIcons>()
        .add_event::<harvest_fx::Break>()
        .add_event::<sound::Cue>()
        .add_event::<world::Watered>()
        .add_systems(
            Startup,
            (
                world::setup,
                fieldwork::setup,
                forest::setup,
                season_view::setup,
                ui::setup,
                weather::setup,
                session::setup,
                town::setup,
                farm_ui::setup,
                battle_ui::setup,
                cinematic::setup,
                sound::setup,
                wildlife::setup,
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
            )
                .chain()
                .after(session::open_menu)
                .run_if(session::playing),
        )
        .add_systems(
            Update,
            (
                adventure::draw,
                resident::move_residents,
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
                cinematic::show,
                weather::precipitation,
                weather::lighting,
                sound::play,
                wildlife::animate,
                adventure::animate_animals,
                ui::update,
                ui::update_fertilizer,
            )
                .chain()
                .after(adventure::interact)
                .after(session::open_menu),
        )
        .add_systems(
            Update,
            (
                fieldwork::show.after(world::update_selection),
                harvest_fx::animate.before(sound::play),
                forest::draw,
                session::update,
                town::update,
                farm_ui::update,
                battle_ui::update,
                battle_ui::draw_trial_guardian,
                battle_ui::animate_guardian,
                ui::sync_visibility,
            )
                .after(adventure::interact),
        )
        .add_systems(PostUpdate, session::autosave)
        .add_systems(
            PostUpdate,
            (ui::map_labels, resident_view::update)
                .after(bevy::render::camera::CameraUpdateSystem)
                .before(bevy::ui::UiSystem::Prepare),
        );
    #[cfg(target_os = "android")]
    app.add_systems(Startup, controller::setup)
        .add_systems(
            Update,
            (
                session::finish_document.before(session::input),
                mobile::refresh_insets.before(ui::resize),
            ),
        )
        .add_systems(
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
    mut focus: ResMut<GardenFocus>,
    mut player: Single<&mut Gardener>,
    mut notice: ResMut<Notice>,
    time: Res<Time>,
) {
    if adventure.area != Area::Island {
        return;
    }
    if controls.field_tool && !adventure.map && player.position.length() < 17. {
        focus.tool = focus.tool.next();
        if focus.pending.take().is_some() {
            player.stop();
        }
        *notice = Notice {
            message: if focus.tool == fieldwork::FieldTool::Auto {
                "Werkzeugwahl automatisch".into()
            } else {
                format!("{} bleibt ausgewählt", focus.tool.name())
            },
            until: time.elapsed_secs() + Notice::DURATION,
            ..default()
        };
    }
    if controls.move_axis != Vec2::ZERO
        && focus
            .selected
            .is_some_and(|index| PLOTS[index].distance(player.position) > 5.)
    {
        focus.selected = None;
        focus.pending = None;
    }
    if controls.cycle && !adventure.map && player.position.length() < 16. {
        focus.selected = Some(
            focus
                .selected
                .map_or(0, |previous| (previous + 1) % PLOTS.len()),
        );
        focus.pending = None;
    }
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
    mut watered: EventWriter<world::Watered>,
) {
    if controls.move_axis != Vec2::ZERO
        || adventure.area != Area::Island
        || adventure.map
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
        && matches!(
            focus.tool,
            fieldwork::FieldTool::Auto | fieldwork::FieldTool::Seeds
        )
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
        focus.tool.action(&garden, index, adventure.crop)
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
    let reach = world::work_reach(PLOTS[index], action, &garden);
    if !reach.reached(player.position, &|p| adventure.walkable(p, &garden)) {
        if requested.is_some() {
            focus.pending = player
                .navigate(reach, &adventure, &garden)
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
    let stroke = fieldwork::stroke(&garden, index, crop, action, focus.tool);
    if let Some(mut message) = garden.work(index, crop, action) {
        let position = player.position;
        player.stop();
        let direction = PLOTS[index] - position;
        if direction.length_squared() > 0.01 {
            transform.rotation = Quat::from_rotation_y(direction.x.atan2(direction.y));
        }
        let mut worked = 1;
        if action == Action::Water {
            watered.write(world::Watered(index));
        }
        for &other in &stroke.targets()[1..] {
            if garden.work(other, crop, action).is_some() {
                worked += 1;
                if action == Action::Water {
                    watered.write(world::Watered(other));
                }
            }
        }
        if worked > 1 {
            message = if action == Action::Water {
                format!("{worked} Beete gleichzeitig gegossen.")
            } else {
                format!("{worked} Beete vorbereitet.")
            };
        }
        if let Some(next) = stroke.next {
            focus.selected = Some(next);
            message.push_str(&format!(" · Nächstes Beet {}", next + 1));
        }
        adventure::commit(
            &garden,
            &path,
            &mut notice,
            now,
            message,
            focus.selected,
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
