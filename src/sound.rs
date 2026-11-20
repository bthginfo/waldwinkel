use bevy::{audio::Volume, prelude::*};

use crate::{
    Adventure, Garden, Notice,
    adventure::Area,
    farm_ui::FarmMenu,
    garden::Action,
    session::Session,
    terrain,
    travel::District,
    weather::{Season, Sky},
    world::Gardener,
};

#[derive(Component)]
pub enum Ambience {
    Coast,
    Rain,
    Farm,
    Forest,
    Harbor,
}

#[derive(Resource)]
pub struct Sounds {
    steps: [Handle<AudioSource>; 3],
    harvest: Handle<AudioSource>,
    impact: Handle<AudioSource>,
    support: Handle<AudioSource>,
}

#[derive(Default)]
pub struct Pace {
    previous: Option<(Area, Vec2)>,
    distance: f32,
    variation: usize,
    harvested_at: f32,
    battle: Option<(u16, u16, u16, u16, u8, usize)>,
}

pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    for (layer, track) in [
        (Ambience::Coast, "coast.wav"),
        (Ambience::Rain, "rain_wood.wav"),
        (Ambience::Farm, "farm_theme.wav"),
        (Ambience::Forest, "fernwind.wav"),
        (Ambience::Harbor, "harbor_theme.wav"),
    ] {
        commands.spawn((
            layer,
            AudioPlayer::new(assets.load(format!("audio/{track}"))),
            PlaybackSettings::LOOP.with_volume(Volume::SILENT),
        ));
    }
    commands.insert_resource(Sounds {
        steps: ["step_grass", "step_wood", "step_snow"]
            .map(|name| assets.load(format!("audio/{name}.wav"))),
        harvest: assets.load("audio/harvest_sound.wav"),
        impact: assets.load("audio/card_hit.wav"),
        support: assets.load("audio/card_shield.wav"),
    });
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
    notice: Res<Notice>,
    mut ambience: Query<(&Ambience, &mut AudioSink)>,
    mut pace: Local<Pace>,
) {
    let area = adventure.area;
    let point = player.position;
    let audible = session.playing() && !garden.sound_muted;
    let district = District::at(point);
    let calm = audible && menu.battle().is_none();
    let soundtrack = if calm && Sky::at(garden.day) == Sky::Rain {
        0.58
    } else {
        1.
    };
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
            Ambience::Farm
                if calm && (area == Area::Home || area.outdoors() && point.length() < 19.) =>
            {
                0.20 * soundtrack
            }
            Ambience::Forest
                if audible && area.outdoors() && district == Some(District::Forest) =>
            {
                0.23 * soundtrack
            }
            Ambience::Harbor
                if calm
                    && area.outdoors()
                    && district == Some(District::Town)
                    && garden.harbor.phase == crate::harbor::Phase::Open =>
            {
                0.22 * soundtrack
            }
            _ => 0.,
        };
        let previous = sink.volume().to_linear();
        if (previous - target).abs() > 0.002 {
            sink.set_volume(Volume::Linear(previous.lerp(target, blend)));
        }
    }
    let combat = menu.battle().map(|battle| {
        (
            battle.enemy_hp,
            battle.enemy_guard,
            battle.hp,
            battle.guard,
            battle.energy,
            battle.hand.len(),
        )
    });
    let cue = if notice.action == Some(Action::Harvest) && notice.until != pace.harvested_at {
        pace.harvested_at = notice.until;
        Some(&sounds.harvest)
    } else if let (
        Some((enemy, armor, life, guard, energy, hand)),
        Some((old_enemy, old_armor, old_life, old_guard, old_energy, old_hand)),
    ) = (combat, pace.battle)
    {
        if enemy < old_enemy || armor < old_armor || life < old_life {
            Some(&sounds.impact)
        } else if guard > old_guard || life > old_life || energy < old_energy || hand != old_hand {
            Some(&sounds.support)
        } else {
            None
        }
    } else {
        None
    };
    pace.battle = combat;
    if let Some(clip) = cue.filter(|_| audible) {
        commands.spawn((
            AudioPlayer::new(clip.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.42)),
        ));
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
        AudioPlayer::new(sounds.steps[sample].clone()),
        PlaybackSettings::DESPAWN
            .with_volume(Volume::Linear(0.38))
            .with_speed(speed),
    ));
}
