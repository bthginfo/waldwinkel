use bevy::{prelude::*, scene::SceneInstanceReady};

use crate::{
    Garden, evening_market, harbor,
    town::{Dialog, Npc},
    world::{self, Gardener},
};

const BO_ROUTE: &[Vec2] = &[
    Vec2::new(50.50, 1.50),
    Vec2::new(56.00, 7.75),
    Vec2::new(57.25, 8.25),
    Vec2::new(57.50, 15.50),
    Vec2::new(57.25, 24.00),
    Vec2::new(56.75, 25.00),
];
const PIER_ROUTE: &[Vec2] = &[
    Vec2::new(56.75, 25.00),
    Vec2::new(58.00, 26.60),
    Vec2::new(58.00, 30.90),
    Vec2::new(57.70, 33.25),
    Vec2::new(57.50, 35.78),
];
const FIETE_SHORE: &[Vec2] = &[
    Vec2::new(54.75, 25.75),
    Vec2::new(50.00, 10.25),
    Vec2::new(52.75, 7.25),
];
const FIETE_PIER: &[Vec2] = &[
    Vec2::new(52.75, 7.25),
    Vec2::new(56.75, 7.75),
    Vec2::new(57.25, 8.25),
    Vec2::new(58.50, 25.25),
];
const FIETE_HOME: &[Vec2] = &[Vec2::new(58.50, 25.25), Vec2::new(54.75, 25.75)];

const MARA_MARKET: &[Vec2] = &[
    Vec2::new(42.16, 5.92),
    Vec2::new(45.65, 5.84),
    Vec2::new(45.94, 7.04),
    Vec2::new(48.50, 7.14),
    Vec2::new(49.10, 6.02),
    Vec2::new(50.18, 7.18),
];
const LINA_MARKET: &[Vec2] = &[
    Vec2::new(43.50, 2.45),
    Vec2::new(44.53, 3.48),
    Vec2::new(45.55, 4.88),
    Vec2::new(45.94, 7.04),
    Vec2::new(48.50, 7.14),
    Vec2::new(51.20, 8.45),
];
const MIRO_MARKET: &[Vec2] = &[
    Vec2::new(54.81, 2.50),
    Vec2::new(53.96, 4.15),
    Vec2::new(53.35, 6.20),
    Vec2::new(52.92, 7.86),
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Duty {
    #[default]
    Idle,
    Stir,
    Water,
    Sort,
    Write,
    Forge,
}

impl Duty {
    fn prop(self) -> Option<&'static str> {
        match self {
            Self::Stir => Some("resident_spoon"),
            Self::Water => Some("watering_can"),
            Self::Sort => Some("resident_parcel"),
            Self::Write => Some("resident_notebook"),
            Self::Forge => Some("resident_hammer"),
            Self::Idle => None,
        }
    }

    fn arms(self, time: f32, side: f32) -> (f32, f32, f32) {
        let wave = time.sin();
        let opposite = (time + 1.6).sin();
        let right = side > 0.;
        match self {
            Self::Stir if right => (-1.31 + 0.16 * wave, 0.19 * opposite, -0.21 + 0.15 * wave),
            Self::Stir => (-0.91, -0.16, 0.23),
            Self::Water if right => (-0.95 - 0.28 * wave.max(0.), 0.13 * opposite, -0.24),
            Self::Sort => (
                -0.87 + 0.27 * (time * 0.57 + side).sin(),
                -side * 0.13,
                -side * 0.18,
            ),
            Self::Write => (
                -1.02 + 0.055 * (time * 2.95 + side).sin(),
                -side * 0.23,
                -side * 0.13,
            ),
            Self::Forge if right => (-1.25 - 0.59 * wave.max(0.), 0.08, -0.18),
            Self::Forge => (-0.80, -0.19, 0.18),
            _ => (0., 0., 0.),
        }
    }
}

struct Routine {
    path: &'static [Vec2],
    shift: [(u16, u16); 2],
    travel: u16,
    station: &'static str,
    errand: &'static str,
}

fn routine(npc: Npc) -> Option<Routine> {
    Some(match npc {
        Npc::Lina => Routine {
            path: const {
                &[
                    Vec2::new(43.50, 2.45),
                    Vec2::new(43.70, 4.40),
                    Vec2::new(44.05, 5.90),
                    Vec2::new(44.75, 6.19),
                ]
            },
            shift: [(365, 437), (920, 964)],
            travel: 5,
            station: "Sortiert Saatgut am Markt",
            errand: "Pflegt die Pflanzkübel am Markt",
        },
        Npc::Miro => Routine {
            path: const {
                &[
                    Vec2::new(54.81, 2.50),
                    Vec2::new(54.38, 4.70),
                    Vec2::new(53.86, 5.80),
                    Vec2::new(53.20, 8.20),
                ]
            },
            shift: [(393, 463), (1002, 1059)],
            travel: 7,
            station: "Handelt am Marktstand",
            errand: "Zählt Lieferungen am Hafentor",
        },
        Npc::Mara => Routine {
            path: const {
                &[
                    Vec2::new(42.16, 5.92),
                    Vec2::new(42.80, 5.20),
                    Vec2::new(44.70, 5.20),
                    Vec2::new(45.60, 6.35),
                ]
            },
            shift: [(567, 641), (1084, 1170)],
            travel: 5,
            station: "Bereitet Essen vor ihrem Haus zu",
            errand: "Sucht frische Kräuter am Marktplatz",
        },
        Npc::Juna => Routine {
            path: const {
                &[
                    Vec2::new(56.26, 6.56),
                    Vec2::new(55.75, 6.40),
                    Vec2::new(54.79, 6.20),
                    Vec2::new(51.15, 6.05),
                ]
            },
            shift: [(499, 553), (834, 903)],
            travel: 6,
            station: "Zeichnet Karten am Hafenweg",
            errand: "Vergleicht Wegzeichen am Brunnen",
        },
        Npc::Runa => Routine {
            path: const {
                &[
                    Vec2::new(124.70, 5.65),
                    Vec2::new(124.70, 4.80),
                    Vec2::new(124.40, 3.80),
                    Vec2::new(126.30, 3.80),
                ]
            },
            shift: [(462, 533), (872, 946)],
            travel: 5,
            station: "Arbeitet vor der Schmiede",
            errand: "Prüft die Erzvorräte in der Kupferklamm",
        },
        Npc::Nele => Routine {
            path: const {
                &[
                    Vec2::new(15.10, 117.90),
                    Vec2::new(14.50, 116.80),
                    Vec2::new(13.00, 116.30),
                    Vec2::new(12.00, 115.40),
                ]
            },
            shift: [(425, 509), (768, 887)],
            travel: 5,
            station: "Sortiert Lieferungen auf den Windweiden",
            errand: "Kontrolliert die Pflanzen am Weideweg",
        },
        Npc::Edda => Routine {
            path: const {
                &[
                    Vec2::new(-142.20, -6.05),
                    Vec2::new(-143.20, -5.80),
                    Vec2::new(-144.23, -4.95),
                    Vec2::new(-146.00, -4.20),
                ]
            },
            shift: [(537, 682), (1139, 1269)],
            travel: 5,
            station: "Hält Wache am Moorweg",
            errand: "Prüft die Markierungen am Ufer",
        },
        Npc::Bo | Npc::Fiete => return None,
    })
}

pub(crate) struct Pose {
    pub point: Vec2,
    pub activity: &'static str,
    direction: Vec2,
    walking: bool,
    duty: Duty,
}

impl Pose {
    fn stay(point: Vec2, activity: &'static str) -> Self {
        Self {
            point,
            activity,
            direction: Vec2::new(-0.27, 1.),
            walking: false,
            duty: Duty::Idle,
        }
    }

    fn travel(path: &[Vec2], progress: f32, reverse: bool, activity: &'static str) -> Self {
        let progress = if reverse { 1. - progress } else { progress }.clamp(0., 1.);
        let mut distance = progress
            * path
                .windows(2)
                .map(|pair| pair[0].distance(pair[1]))
                .sum::<f32>();
        for pair in path.windows(2) {
            let offset = pair[1] - pair[0];
            let length = offset.length();
            if distance < length {
                return Self {
                    point: pair[0] + offset * (distance / length),
                    activity,
                    direction: if reverse { -offset } else { offset },
                    walking: true,
                    duty: Duty::Idle,
                };
            }
            distance -= length;
        }
        Self {
            point: *path.last().unwrap(),
            direction: path[path.len() - 1] - path[path.len() - 2],
            activity,
            walking: true,
            duty: Duty::Idle,
        }
    }

    fn work(mut self, duty: Duty) -> Self {
        self.duty = duty;
        self
    }
}

fn market_pose(npc: Npc, garden: &Garden) -> Option<Pose> {
    if !evening_market::scheduled(garden) {
        return None;
    }
    let (route, duration, duty, role) = match npc {
        Npc::Mara => (MARA_MARKET, 10, Duty::Stir, "Kocht am Abendmarkt"),
        Npc::Lina => (
            LINA_MARKET,
            12,
            Duty::Sort,
            "Legt Saatgut am Abendmarkt aus",
        ),
        Npc::Miro => (MIRO_MARKET, 8, Duty::Sort, "Sortiert Kisten am Abendmarkt"),
        _ => return None,
    };
    let start = evening_market::OPENS - duration;
    let end = evening_market::CLOSES;
    let minute = garden.clock.precise_minute();
    let duration = f32::from(duration);
    if (f32::from(start)..f32::from(evening_market::OPENS)).contains(&minute) {
        return Some(Pose::travel(
            route,
            (minute - f32::from(start)) / duration,
            false,
            "Unterwegs zum Abendmarkt",
        ));
    }
    if evening_market::hours(garden) {
        return Some(Pose::stay(*route.last().unwrap(), role).work(duty));
    }
    if (f32::from(end)..f32::from(end) + duration).contains(&minute) {
        return Some(Pose::travel(
            route,
            (minute - f32::from(end)) / duration,
            true,
            "Auf dem Heimweg vom Abendmarkt",
        ));
    }
    None
}

pub(crate) fn pose(npc: Npc, garden: &Garden) -> Pose {
    if npc == Npc::Bo {
        if garden.fish_supper.at_farm() || garden.fish_supper.waiting_at_farm(garden.day, garden.clock.minute()) {
            return Pose::stay(crate::adventure::KITCHEN + Vec2::new(1.35, 0.35), "Zu Besuch an deiner Hofküche")
                .work(if garden.fish_supper.learned() { Duty::Stir } else { Duty::Sort });
        }
        if garden.fish_supper.offered_today(garden.day) && evening_market::hours(garden) {
            return Pose::stay(evening_market::PLACE + Vec2::new(1.25, 0.63), "Serviert Hafenbrühe am Abendmarkt").work(Duty::Stir);
        }
    }
    if let Some(state) = market_pose(npc, garden) {
        return state;
    }
    let minute = garden.clock.minute();
    let now = garden.clock.precise_minute();
    let walk = |path, from, duration, reverse, activity| {
        Pose::travel(path, (now - f32::from(from)) / duration, reverse, activity)
    };
    if garden.harbor.celebrating(garden.day) && !garden.clock.night() {
        match npc {
            Npc::Bo => {
                return Pose::stay(
                    Vec2::new(57.50, 35.78),
                    "Bei der Wiedereröffnung am Fischersteg",
                );
            }
            Npc::Fiete => {
                return Pose::stay(
                    Vec2::new(58.69, 34.12),
                    "Bei der Wiedereröffnung am Fischersteg",
                );
            }
            _ => (),
        }
    }
    let pier_open = garden.harbor.phase == harbor::Phase::Open;
    match (npc, minute) {
        (Npc::Bo, 0..780 | 1194..) => Pose::stay(BO_ROUTE[0], "Am Rathaus").work(Duty::Write),
        (Npc::Bo, 780..804) => walk(BO_ROUTE, 780, 24., false, "Unterwegs zum Fischersteg"),
        (Npc::Bo, 804..820) if pier_open => {
            walk(PIER_ROUTE, 804, 16., false, "Unterwegs zur Seeseite")
        }
        (Npc::Bo, 820..1154) if pier_open => Pose::stay(
            *PIER_ROUTE.last().unwrap(),
            "Auf der Seeseite des Fischerstegs",
        ),
        (Npc::Bo, 1154..1170) if pier_open => {
            walk(PIER_ROUTE, 1154, 16., true, "Unterwegs vom Fischersteg")
        }
        (Npc::Bo, 804..1170) => Pose::stay(*BO_ROUTE.last().unwrap(), "Am Fischersteg"),
        (Npc::Bo, _) => walk(BO_ROUTE, 1170, 24., true, "Unterwegs zum Rathaus"),
        (Npc::Fiete, 0..600 | 1207..) => {
            Pose::stay(FIETE_SHORE[0], "Sammelt Muscheln am Hafenufer")
        }
        (Npc::Fiete, 600..619) => walk(FIETE_SHORE, 600, 19., false, "Unterwegs zum Marktplatz"),
        (Npc::Fiete, 619..1020) => Pose::stay(FIETE_PIER[0], "Am Marktplatz"),
        (Npc::Fiete, 1020..1040) => walk(FIETE_PIER, 1020, 20., false, "Unterwegs zum Fischersteg"),
        (Npc::Fiete, 1040..1200) => Pose::stay(FIETE_HOME[0], "Am Fischersteg"),
        (Npc::Fiete, _) => walk(FIETE_HOME, 1200, 7., false, "Unterwegs zum Hafenufer"),
        _ => {
            let routine = routine(npc).expect("resident has a routine");
            for (leave, back) in routine.shift {
                if (leave..leave + routine.travel).contains(&minute) {
                    return walk(
                        routine.path,
                        leave,
                        f32::from(routine.travel),
                        false,
                        "Unterwegs zum nächsten Halt",
                    );
                }
                if (leave + routine.travel..back).contains(&minute) {
                    let duty = match npc {
                        Npc::Lina | Npc::Nele => Duty::Water,
                        Npc::Miro | Npc::Runa | Npc::Mara => Duty::Sort,
                        _ => Duty::Write,
                    };
                    return Pose::stay(*routine.path.last().unwrap(), routine.errand).work(duty);
                }
                if (back..back + routine.travel).contains(&minute) {
                    return walk(
                        routine.path,
                        back,
                        f32::from(routine.travel),
                        true,
                        "Auf dem Rückweg",
                    );
                }
            }
            let duty = match npc {
                Npc::Mara => Duty::Stir,
                Npc::Runa => Duty::Forge,
                Npc::Juna | Npc::Edda => Duty::Write,
                _ => Duty::Sort,
            };
            Pose::stay(routine.path[0], routine.station).work(duty)
        }
    }
}

#[derive(Component)]
pub struct Resident(pub Npc);

#[derive(Component)]
pub(crate) struct Limb {
    resident: Npc,
    side: f32,
    arm: bool,
    height: f32,
}

#[derive(Component)]
pub(crate) struct WorkProp(Npc, Duty);

pub(crate) fn bind_limbs(
    trigger: Trigger<SceneInstanceReady>,
    residents: Query<&Resident>,
    children: Query<&Children>,
    names: Query<(&Name, &Transform)>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    let Ok(resident) = residents.get(trigger.target()) else {
        return;
    };
    for entity in children.iter_descendants(trigger.target()) {
        let Ok((name, pose)) = names.get(entity) else {
            continue;
        };
        let part = name.as_str().strip_prefix(resident.0.model());
        let (arm, side) = match part {
            Some("_leg_l") => (false, -1.),
            Some("_leg_r") => (false, 1.),
            Some("_arm_l") => (true, -1.),
            Some("_arm_r") => (true, 1.),
            _ => continue,
        };
        commands.entity(entity).insert(Limb {
            resident: resident.0,
            arm,
            side,
            height: pose.translation.y,
        });
        if arm && side > 0. {
            let duties: &[Duty] = match resident.0 {
                Npc::Mara => &[Duty::Stir, Duty::Sort],
                Npc::Lina | Npc::Nele => &[Duty::Water, Duty::Sort],
                Npc::Miro => &[Duty::Sort],
                Npc::Bo | Npc::Juna | Npc::Edda => &[Duty::Write],
                Npc::Runa => &[Duty::Forge, Duty::Sort],
                _ => &[],
            };
            for &duty in duties {
                if let Some(model) = duty.prop() {
                    commands.entity(entity).with_child((
                        WorkProp(resident.0, duty),
                        SceneRoot(assets.load(format!("models/{model}.glb#Scene0"))),
                        Transform::from_xyz(0., -0.268, 0.046)
                            .with_rotation(Quat::from_rotation_x(-0.65))
                            .with_scale(Vec3::splat(if duty == Duty::Water { 0.60 } else { 1. })),
                        Visibility::Hidden,
                    ));
                }
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct Gait {
    phase: f32,
    amount: f32,
    duty: Duty,
    work: f32,
}

pub fn move_residents(
    garden: Res<Garden>,
    time: Res<Time>,
    dialog: Res<Dialog>,
    player: Single<&Gardener>,
    mut residents: Query<(&Resident, &mut Transform), Without<Limb>>,
    mut limbs: Query<(&Limb, &mut Transform), Without<Resident>>,
    mut props: Query<(&WorkProp, &mut Visibility)>,
    mut gaits: Local<[Gait; Npc::ALL.len()]>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();
    let blend = 1. - (-dt * 14.).exp();
    for (resident, mut transform) in &mut residents {
        let which = resident.0 as usize;
        let state = pose(resident.0, &garden);
        let speaking = dialog.person() == Some(resident.0);
        let distance = state.point.distance(transform.translation.xz());
        let walking = state.walking && distance < 1.2;
        let stride = 0.66;
        let mut amount = 0.;
        let gait = &mut gaits[which];
        if walking && dt > 0. {
            gait.phase = (gait.phase + distance / stride).fract();
            amount = (distance / (dt * 1.2)).clamp(0., 1.);
        }
        gait.amount = gait.amount.lerp(amount, blend);
        gait.duty = if speaking || walking {
            Duty::Idle
        } else {
            state.duty
        };
        gait.work = gait
            .work
            .lerp(if gait.duty == Duty::Idle { 0. } else { 1. }, blend);
        let y = world::surface_height(state.point, &garden) + 0.018;
        transform.translation = Vec3::new(state.point.x, y, state.point.y);
        let direction = if speaking {
            player.position - state.point
        } else {
            state.direction
        };
        let heading = direction.x.atan2(direction.y)
            + if speaking {
                0.
            } else {
                (which as f32 - 4.) * 0.14 * (1. - gait.amount)
            };
        transform.rotation = transform
            .rotation
            .slerp(Quat::from_rotation_y(heading), blend);
    }
    for (limb, mut transform) in &mut limbs {
        let gait = &gaits[limb.resident as usize];
        let phase = world::step_phase(gait.phase, limb.side);
        let (pitch, lift) = if limb.arm {
            let idle = ((elapsed * 0.79 + limb.resident as usize as f32 * 1.7 + limb.side).sin()
                * 0.5
                + 0.5)
                .powi(5);
            (
                -0.29 * (phase * std::f32::consts::TAU).cos() * gait.amount
                    - 0.18 * idle * (1. - gait.amount),
                0.,
            )
        } else {
            let (swing, rise) = world::foot_pose(phase);
            let pitch = -swing * 0.49 * gait.amount;
            (pitch, rise * 0.55 * gait.amount - (1. - pitch.cos()) * 0.38)
        };
        transform.translation.y = limb.height + lift;
        let (motion, yaw, roll) = if limb.arm {
            gait.duty
                .arms(elapsed * 2.45 + limb.resident as usize as f32, limb.side)
        } else {
            (0., 0., 0.)
        };
        let target = Quat::from_euler(
            EulerRot::XYZ,
            pitch + motion * gait.work,
            yaw * gait.work,
            roll * gait.work,
        );
        transform.rotation = transform.rotation.slerp(target, blend);
    }
    for (prop, mut visible) in &mut props {
        let gait = &gaits[prop.0 as usize];
        visible.set_if_neq(if gait.duty == prop.1 && gait.work > 0.40 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{adventure::Adventure, day::FarmClock, regions::Frontier};

    #[test]
    fn market_guests_walk_there_and_back_on_passable_ground() {
        let mut garden = Garden::default();
        garden.evening_market.stage = evening_market::Stage::Open;
        let adventure = Adventure::default();
        for npc in [Npc::Mara, Npc::Lina, Npc::Miro] {
            let mut previous: Option<Vec2> = None;
            for tick in 1050 * 2..=1310 * 2 {
                garden.clock = FarmClock::from_minutes(tick as f32 * 0.5);
                let state = pose(npc, &garden);
                assert!(
                    adventure.walkable_without_residents(state.point, &garden),
                    "{npc:?} at {} {:?}",
                    garden.clock.display(),
                    state.point
                );
                if let Some(point) = previous {
                    assert!(
                        point.distance(state.point) < 1.25,
                        "{npc:?} jumps at {}",
                        garden.clock.display()
                    );
                }
                if evening_market::hours(&garden) {
                    assert!(state.point.distance(evening_market::PLACE) < 1.8);
                }
                previous = Some(state.point);
            }
        }
    }

    #[test]
    fn residents_change_places_and_keep_to_walkable_ground() {
        let mut garden: Garden = serde_json::from_str(r#"{"regions":{"gates":[2,2,2]}}"#).unwrap();
        let adventure = Adventure::default();
        for npc in Npc::ALL {
            assert!(npc.available(&garden));
            for phase in [harbor::Phase::Damaged, harbor::Phase::Open] {
                garden.harbor.phase = phase;
                let mut previous = None::<Vec2>;
                let mut moved = 0.0;
                for step in 720..=2878 {
                    garden.clock = FarmClock::from_minutes(step as f32 / 2.);
                    let state = pose(npc, &garden);
                    assert!(
                        adventure.walkable_without_residents(state.point, &garden),
                        "{npc:?}, {} {:?} ({phase:?})",
                        garden.clock.display(),
                        state.point
                    );
                    if let Some(last) = previous {
                        let distance = state.point.distance(last);
                        assert!(distance < 1.15, "{npc:?}, {}", garden.clock.display());
                        moved += distance;
                    }
                    previous = Some(state.point);
                }
                assert!(moved > 5., "{npc:?}");
            }
        }
        garden = Garden::default();
        assert!(
            Frontier::ALL
                .into_iter()
                .all(|area| !area.unlocked(&garden))
        );
        assert!(!Npc::Runa.available(&garden));
    }
}
