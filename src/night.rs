use bevy::{prelude::*, time::Real};

use crate::{
    Adventure, Garden,
    controller::Controller,
    garden::Stage,
    session::{Checkpoint, SaveSlots, Session},
    sound::Cue,
    ui::UiFonts,
    world::Gardener,
};

const DAWN: f32 = 0.62;
const FADE: f32 = 1.85;
const END: f32 = 2.30;

#[derive(Resource, Default)]
pub struct Night {
    rest: Option<Rest>,
}

struct Rest {
    elapsed: f32,
    morning: Option<Morning>,
}

struct Morning {
    day: u32,
    ripe: usize,
    saved: bool,
}

impl Night {
    pub fn active(&self) -> bool {
        self.rest.is_some()
    }

    pub fn begin(&mut self) {
        self.rest.get_or_insert(Rest {
            elapsed: 0.,
            morning: None,
        });
    }
}

#[derive(Component)]
pub struct Shade;
#[derive(Component)]
pub struct Disc;
#[derive(Component)]
pub struct Crescent;
#[derive(Component)]
pub enum Caption {
    Title,
    Growth,
    Save,
}

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>) {
    commands
        .spawn((
            Shade,
            GlobalZIndex(180),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|scene| {
            scene
                .spawn(Node {
                    width: Val::Px(92.),
                    height: Val::Px(91.),
                    ..default()
                })
                .with_children(|sky| {
                    sky.spawn((
                        Disc,
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(16.),
                            top: Val::Px(17.),
                            width: Val::Px(60.),
                            height: Val::Px(60.),
                            ..default()
                        },
                        BorderRadius::MAX,
                        BackgroundColor(Color::WHITE),
                    ))
                    .with_child((
                        Crescent,
                        Node {
                            position_type: PositionType::Absolute,
                            right: Val::Px(-9.),
                            top: Val::Px(-8.),
                            width: Val::Px(52.),
                            height: Val::Px(52.),
                            ..default()
                        },
                        BorderRadius::MAX,
                        BackgroundColor(Color::NONE),
                    ));
                });
            scene.spawn((Caption::Title, fonts.title("", 39.)));
            scene.spawn((Caption::Growth, fonts.button("", 17.)));
            scene.spawn((Caption::Save, fonts.button("", 15.)));
        });
}

fn wake_in_saved_slot(
    garden: &mut Garden,
    session: &mut Session,
    slots: &mut SaveSlots,
    point: Checkpoint,
    now: f32,
) -> Morning {
    let before = garden
        .plots
        .iter()
        .filter(|p| p.stage() == Stage::Ripe)
        .count();
    garden.sleep();
    let ripe = garden
        .plots
        .iter()
        .filter(|p| p.stage() == Stage::Ripe)
        .count()
        .saturating_sub(before);
    let saved = match session.save(slots, garden, point, now) {
        Ok(()) => true,
        Err(error) => {
            warn!("Speichern beim Schlafen fehlgeschlagen: {error}");
            false
        }
    };
    Morning {
        day: garden.day,
        ripe,
        saved,
    }
}

pub fn update(
    mut night: ResMut<Night>,
    time: Res<Time<Real>>,
    clock: Res<Time>,
    controller: Res<Controller>,
    keys: Res<ButtonInput<KeyCode>>,
    mut session: ResMut<Session>,
    mut slots: ResMut<SaveSlots>,
    mut garden: ResMut<Garden>,
    adventure: Res<Adventure>,
    player: Single<&Gardener>,
    mut shade: Single<(&mut Node, &mut BackgroundColor), With<Shade>>,
    mut disc: Single<(&mut Node, &mut BackgroundColor), (With<Disc>, Without<Shade>)>,
    mut moon: Single<
        (&mut Node, &mut BackgroundColor),
        (With<Crescent>, Without<Shade>, Without<Disc>),
    >,
    mut captions: Query<(&Caption, &mut Text, &mut TextColor)>,
    mut sounds: EventWriter<Cue>,
) {
    let Some(rest) = night.rest.as_mut() else {
        shade.0.display = Display::None;
        return;
    };
    let morning = rest.morning.get_or_insert_with(|| {
        wake_in_saved_slot(
            &mut garden,
            &mut session,
            &mut slots,
            Checkpoint::new(adventure.area, player.position),
            clock.elapsed_secs(),
        )
    });
    let elapsed = rest.elapsed + time.delta_secs().min(0.12);
    if rest.elapsed < DAWN && elapsed >= DAWN {
        sounds.write(Cue::Dawn);
    }
    rest.elapsed = elapsed;
    let awake = elapsed >= DAWN;
    let ready = elapsed >= 1.22;
    let keep_open = !morning.saved;
    let pressed = controller.accept
        || keys.any_just_pressed([KeyCode::Space, KeyCode::Enter, KeyCode::Escape]);
    if ready && (pressed || (!keep_open && elapsed >= END) || elapsed >= 5.2) {
        night.rest = None;
        shade.0.display = Display::None;
        return;
    }
    let fade = if keep_open {
        1.
    } else {
        1. - smooth((elapsed - FADE) / (END - FADE))
    };
    let rise = smooth((elapsed - DAWN) / 0.46);
    let sky = Color::srgba(
        0.075 + 0.12 * rise,
        0.125 + 0.063 * rise,
        0.188 - 0.048 * rise,
        fade * 0.98,
    );
    shade.0.display = Display::Flex;
    shade.1.0 = sky;
    disc.0.top = Val::Px(35. - 19. * rise - 6. * smooth(elapsed / DAWN));
    disc.1.0 = if awake {
        Color::srgba(1., 0.72, 0.32, fade)
    } else {
        Color::srgba(0.94, 0.91, 0.76, fade)
    };
    moon.0.display = if awake { Display::None } else { Display::Flex };
    moon.1.0 = sky;
    for (caption, mut text, mut color) in &mut captions {
        let value = match (caption, awake) {
            (Caption::Title, false) => "Gute Nacht".into(),
            (Caption::Title, true) => format!("Guten Morgen · Tag {}", morning.day),
            (Caption::Growth, true) if garden.fish_supper.stage == crate::fish_supper::Stage::Visiting => "Bo wartet draußen an deiner Hofküche.".into(),
            (Caption::Growth, true) => match morning.ripe {
                0 => "06:00 Uhr · Ausgeruht".into(),
                1 => "06:00 Uhr · Ein Beet ist erntebereit".into(),
                ripe => format!("06:00 Uhr · {ripe} Beete sind erntebereit"),
            },
            (Caption::Save, true) if morning.saved => "Spielstand gespeichert".into(),
            (Caption::Save, true) => "Speichern fehlgeschlagen · Im Menü erneut versuchen".into(),
            _ => String::new(),
        };
        if text.0 != value {
            text.0 = value;
        }
        color.0 = Color::srgba(1., 0.94, 0.79, fade);
    }
}

fn smooth(value: f32) -> f32 {
    let t = value.clamp(0., 1.);
    t * t * (3. - 2. * t)
}
