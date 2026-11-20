use bevy::{audio::Volume, prelude::*};

use crate::{
    Adventure, Garden, Notice,
    adventure::Area,
    farm_ui::FarmMenu,
    garden::Action,
    session::Session,
    terrain,
    town::Dialog,
    travel::District,
    weather::{Season, Sky},
    world::Gardener,
};

#[derive(Component)]
pub enum Bed {
    Coast,
    Rain,
    Birds,
    Piano,
    Harp,
}

#[derive(Clone, Copy, Event)]
pub enum Cue {
    Axe,
    Stone,
    Crack,
    Fall,
    Craft,
    Skill,
    CardSlide,
    CardHit,
    CardGuard,
    CardBloom,
    Turn,
    KitchenChop,
    KitchenStir,
    KitchenJar,
    KitchenMill,
    KitchenDone,
    Dawn,
}

#[derive(Resource)]
pub struct Sounds {
    steps: [[Handle<AudioSource>; 2]; 5],
    garden: [Handle<AudioSource>; 5],
    axe: [Handle<AudioSource>; 2],
    stone: [Handle<AudioSource>; 2],
    crack: Handle<AudioSource>,
    fall: Handle<AudioSource>,
    craft: Handle<AudioSource>,
    skill: Handle<AudioSource>,
    impact: Handle<AudioSource>,
    support: Handle<AudioSource>,
    actions: [Handle<AudioSource>; 8],
}

#[derive(Default)]
pub struct Pace {
    previous: Option<(Area, Vec2)>,
    distance: f32,
    take: usize,
    acted_at: f32,
    confirmed_at: f32,
    field_cue: Option<(Action, f32)>,
    duck_until: f32,
}

pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    for (layer, track) in [
        (Bed::Coast, "coast.wav"),
        (Bed::Rain, "rain_wood.wav"),
        (Bed::Birds, "wild_birds.ogg"),
        (Bed::Piano, "farm_piano.ogg"),
        (Bed::Harp, "farm_harp.ogg"),
    ] {
        commands.spawn((
            layer,
            AudioPlayer::new(assets.load(format!("audio/{track}"))),
            PlaybackSettings::LOOP.with_volume(Volume::SILENT),
        ));
    }
    let wave = |name| assets.load(format!("audio/{name}.wav"));
    commands.insert_resource(Sounds {
        steps: ["grass", "wood", "snow", "sand", "stone"].map(|surface| {
            [
                assets.load(format!("audio/step_{surface}.wav")),
                assets.load(format!("audio/step_{surface}_alt.wav")),
            ]
        }),
        garden: [
            "soil_till",
            "soil_sow",
            "garden_water",
            "soil_fertilize",
            "harvest_sound",
        ]
        .map(wave),
        axe: ["axe_cut", "axe_cut_alt"].map(wave),
        stone: ["stone_break", "stone_break_alt"].map(wave),
        crack: wave("tree_crack"),
        fall: wave("tree_fall"),
        craft: wave("craft_done"),
        skill: wave("skill_learn"),
        impact: wave("card_hit"),
        support: wave("card_shield"),
        actions: ["card_slide", "card_bloom", "turn_page", "kitchen_chop", "kitchen_stir",
                  "kitchen_jar", "kitchen_mill", "kitchen_done"].map(wave),
    });
}

fn one_shot(commands: &mut Commands, clip: &Handle<AudioSource>, gain: f32, take: usize) {
    const PITCH: [f32; 5] = [1., 0.965, 1.04, 0.98, 1.025];
    const LEVEL: [f32; 5] = [1., 0.93, 0.98, 1.06, 0.95];
    let i = take % PITCH.len();
    commands.spawn((
        AudioPlayer::new(clip.clone()),
        PlaybackSettings::DESPAWN
            .with_volume(Volume::Linear(gain * LEVEL[i]))
            .with_speed(PITCH[i]),
    ));
}

pub fn play(
    mut commands: Commands,
    session: Res<Session>,
    garden: Res<Garden>,
    adventure: Res<Adventure>,
    player: Single<&Gardener>,
    time: Res<Time>,
    sounds: Res<Sounds>,
    menu: Res<FarmMenu>,
    dialog: Res<Dialog>,
    notice: Res<Notice>,
    mut beds: Query<(&Bed, &mut AudioSink)>,
    mut cues: EventReader<Cue>,
    mut pace: Local<Pace>,
) {
    let area = adventure.area;
    let point = player.position;
    let now = time.elapsed_secs();
    let audible = session.playing() && !garden.sound_muted;
    let sky = Sky::at(garden.day);
    let district = District::at(point);
    let outside = area.outdoors();

    if notice.toast == Some(crate::feedback::Tone::Success) && pace.confirmed_at != notice.until {
        pace.confirmed_at = notice.until;
        if audible && now < notice.until {
            one_shot(&mut commands, &sounds.actions[7], 0.15, pace.take);
        }
    }
    if let Some(action) = notice.action.filter(|_| notice.until != pace.acted_at) {
        pace.acted_at = notice.until;
        let delay = if matches!(action, Action::Till | Action::Water) {
            0.14
        } else {
            0.02
        };
        pace.field_cue = Some((action, now + delay));
    }
    if !audible {
        pace.field_cue = None;
    }
    if let Some((action, _)) = pace.field_cue.filter(|&(_, at)| now >= at) {
        let gain = if action == Action::Harvest {
            0.34
        } else {
            0.50
        };
        let slot = match action {
            Action::Till => 0,
            Action::Sow => 1,
            Action::Water => 2,
            Action::Fertilize => 3,
            Action::Harvest => 4,
        };
        one_shot(&mut commands, &sounds.garden[slot], gain, pace.take);
        pace.take += 1;
        pace.field_cue = None;
    }
    for cue in cues.read().filter(|_| audible) {
        let take = pace.take;
        let (clip, level) = match cue {
            Cue::Axe => (&sounds.axe[take % 2], 0.64),
            Cue::Stone => (&sounds.stone[take % 2], 0.61),
            Cue::Crack => (&sounds.crack, 0.43),
            Cue::Fall => (&sounds.fall, 0.59),
            Cue::Craft => (&sounds.craft, 0.36),
            Cue::Skill => (&sounds.skill, 0.43),
            Cue::CardSlide => (&sounds.actions[0], 0.31),
            Cue::CardHit => (&sounds.impact, 0.56),
            Cue::CardGuard => (&sounds.support, 0.40),
            Cue::CardBloom => (&sounds.actions[1], 0.43),
            Cue::Turn => (&sounds.actions[2], 0.29),
            Cue::KitchenChop => (&sounds.actions[3], 0.37),
            Cue::KitchenStir => (&sounds.actions[4], 0.40),
            Cue::KitchenJar => (&sounds.actions[5], 0.38),
            Cue::KitchenMill => (&sounds.actions[6], 0.38),
            Cue::KitchenDone => (&sounds.actions[7], 0.43),
            Cue::Dawn => (&sounds.skill, 0.22),
        };
        one_shot(&mut commands, clip, level, take);
        if !matches!(cue, Cue::CardSlide | Cue::Turn) {
            pace.duck_until = now + 0.38;
        }
        pace.take += 1;
    }

    let coast = if outside {
        [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y]
            .into_iter()
            .map(|direction| ((-terrain::ground(point + direction * 7.)) - 0.24).clamp(0., 1.))
            .fold(0.0_f32, f32::max)
            * 0.31
    } else {
        0.
    };
    let rain = if sky == Sky::Rain {
        match area {
            Area::Home => 0.14,
            Area::Island | Area::District(_) => 0.27,
            _ => 0.,
        }
    } else {
        0.
    };
    let dusk = garden.clock.dusk();
    let music = if garden.music_muted || (!outside && area != Area::Home) {
        0.
    } else {
        0.56 * (1. - dusk * 0.59) * if sky == Sky::Rain { 0.80 } else { 1. }
    };
    let duck = if menu.battle().is_some() {
        0.24
    } else if dialog.open() {
        0.49
    } else if now < pace.duck_until {
        0.70
    } else {
        1.
    };
    let harp = match district {
        Some(District::Town) => 0.82,
        Some(District::Beach | District::Wind) => 0.44,
        _ => 0.25,
    };
    let birds = if outside && !matches!(sky, Sky::Rain | Sky::Snow) {
        let density =
            if district == Some(District::Forest) || point.length_squared() < 18.0_f32.powi(2) {
                0.36
            } else {
                0.21
            };
        density
            * (1. - dusk)
            * if Season::at(garden.day) == Season::Winter {
                0.47
            } else {
                1.
            }
    } else {
        0.
    };
    for (layer, mut sink) in &mut beds {
        let target = match layer {
            Bed::Coast => coast,
            Bed::Rain => rain,
            Bed::Birds => birds,
            Bed::Piano => music * duck,
            Bed::Harp => music * harp * duck * (1. - dusk),
        };
        let volume = sink.volume().to_linear();
        let next = if !audible || target == 0. && volume < 0.001 {
            0.
        } else {
            let response = if target < volume { 5.2 } else { 1.85 };
            volume.lerp(target, 1. - (-time.delta_secs() * response).exp())
        };
        if volume != next {
            sink.set_volume(Volume::Linear(next));
        }
    }

    let previous = pace.previous.replace((area, point));
    let distance = previous
        .filter(|(past, _)| *past == area)
        .map_or(0., |(_, last)| last.distance(point));
    if !audible || distance > 1.65 || previous.is_some_and(|(past, _)| past != area) {
        pace.distance = 0.;
        return;
    }
    let speed = (distance / time.delta_secs().max(0.001) / 4.8).clamp(0., 1.);
    let stride = 0.68 + 0.32 * speed;
    pace.distance += distance;
    if pace.distance < stride {
        return;
    }
    pace.distance %= stride;
    let wooden = area == Area::Home
        || (outside
            && (garden.harbor.phase.deck_height(point).is_some() || crate::world::on_dock(point)));
    let sample = if wooden {
        1
    } else if outside && Season::at(garden.day) == Season::Winter {
        2
    } else if district == Some(District::Beach) && outside {
        3
    } else if !outside || matches!(district, Some(District::Town | District::Copper)) {
        4
    } else {
        0
    };
    one_shot(&mut commands, &sounds.steps[sample][pace.take % 2], 0.29 + 0.13 * speed, pace.take);
    pace.take += 1;
}
