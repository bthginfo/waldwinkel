use bevy::{audio::Volume, prelude::*};

use crate::{
    Adventure, Garden,
    adventure::Area,
    session::Session,
    terrain,
    weather::{Season, Sky},
    world::Gardener,
};

#[derive(Component)]
pub enum Ambience {
    Coast,
    Rain,
}

#[derive(Resource)]
pub struct Footsteps([Handle<AudioSource>; 3]);

#[derive(Default)]
pub struct Pace {
    previous: Option<(Area, Vec2)>,
    distance: f32,
    variation: usize,
}

pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    for (layer, track) in [
        (Ambience::Coast, "coast.wav"),
        (Ambience::Rain, "rain_wood.wav"),
    ] {
        commands.spawn((
            layer,
            AudioPlayer::new(assets.load(format!("audio/{track}"))),
            PlaybackSettings::LOOP.with_volume(Volume::SILENT),
        ));
    }
    commands.insert_resource(Footsteps(
        ["step_grass", "step_wood", "step_snow"]
            .map(|name| assets.load(format!("audio/{name}.wav"))),
    ));
}

pub fn play(
    mut commands: Commands,
    session: Res<Session>,
    garden: Res<Garden>,
    adventure: Res<Adventure>,
    player: Single<&Gardener>,
    time: Res<Time>,
    sounds: Res<Footsteps>,
    mut ambience: Query<(&Ambience, &mut AudioSink)>,
    mut pace: Local<Pace>,
) {
    let area = adventure.area;
    let point = player.position;
    let audible = session.playing() && !garden.sound_muted;
    let coast = if audible && area.outdoors() {
        [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y]
            .into_iter()
            .map(|direction| ((-terrain::ground(point + direction * 7.)) - 0.24).clamp(0., 1.))
            .fold(0.0_f32, f32::max)
            * 0.23
    } else {
        0.
    };
    let rain = if audible && Sky::at(garden.day) == Sky::Rain {
        match area {
            Area::Home => 0.12,
            Area::Island | Area::District(_) => 0.27,
            _ => 0.,
        }
    } else {
        0.
    };
    let blend = 1. - (-time.delta_secs() * 3.9).exp();
    for (layer, mut sink) in &mut ambience {
        let target = match layer {
            Ambience::Coast => coast,
            Ambience::Rain => rain,
        };
        let previous = sink.volume().to_linear();
        if (previous - target).abs() > 0.002 {
            sink.set_volume(Volume::Linear(previous.lerp(target, blend)));
        }
    }
    let previous = pace.previous.replace((area, point));
    let distance = previous
        .filter(|(last_area, _)| *last_area == area)
        .map_or(0., |(_, last)| last.distance(point));
    if !audible || distance > 1.65 {
        pace.distance = 0.;
        return;
    }
    pace.distance += distance;
    if pace.distance < 0.49 {
        return;
    }
    pace.distance %= 0.49;
    let wooden = area == Area::Home
        || (area.outdoors()
            && (garden.harbor.phase.deck_height(point).is_some()
                || crate::world::on_dock(point)
                || terrain::ground(point) > 0.17));
    let sample = if wooden {
        1
    } else if area.outdoors() && Season::at(garden.day) == Season::Winter {
        2
    } else {
        0
    };
    let speed = [0.96, 1.07, 1.0, 0.91][pace.variation];
    pace.variation = (pace.variation + 1) % 4;
    commands.spawn((
        AudioPlayer::new(sounds.0[sample].clone()),
        PlaybackSettings::DESPAWN
            .with_volume(Volume::Linear(0.38))
            .with_speed(speed),
    ));
}
