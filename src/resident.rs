use bevy::{prelude::*, scene::SceneInstanceReady};

use crate::{Garden, harbor, town::Npc, world};

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
}

impl Pose {
    fn stay(point: Vec2, activity: &'static str) -> Self {
        Self {
            point,
            activity,
            direction: Vec2::new(-0.27, 1.),
            walking: false,
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
                };
            }
            distance -= length;
        }
        Self {
            point: *path.last().unwrap(),
            direction: path[path.len() - 1] - path[path.len() - 2],
            activity,
            walking: true,
        }
    }
}

pub(crate) fn pose(npc: Npc, garden: &Garden) -> Pose {
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
        (Npc::Bo, 0..780 | 1194..) => Pose::stay(BO_ROUTE[0], "Am Rathaus"),
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
                    return Pose::stay(*routine.path.last().unwrap(), routine.errand);
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
            Pose::stay(routine.path[0], routine.station)
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

pub(crate) fn bind_limbs(
    trigger: Trigger<SceneInstanceReady>,
    residents: Query<&Resident>,
    children: Query<&Children>,
    names: Query<(&Name, &Transform)>,
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
    }
}

#[derive(Default)]
pub(crate) struct Gait {
    phase: f32,
    amount: f32,
}

pub fn move_residents(
    garden: Res<Garden>,
    time: Res<Time>,
    mut residents: Query<(&Resident, &mut Transform), Without<Limb>>,
    mut limbs: Query<(&Limb, &mut Transform), Without<Resident>>,
    mut gaits: Local<[Gait; Npc::ALL.len()]>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();
    let blend = 1. - (-dt * 14.).exp();
    for (resident, mut transform) in &mut residents {
        let which = resident.0 as usize;
        let state = pose(resident.0, &garden);
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
        let y = world::surface_height(state.point, &garden) + 0.018;
        transform.translation = Vec3::new(state.point.x, y, state.point.y);
        let heading = state.direction.x.atan2(state.direction.y)
            + (which as f32 - 4.) * 0.14 * (1. - gait.amount);
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
        transform.rotation = Quat::from_rotation_x(pitch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{adventure::Adventure, day::FarmClock, regions::Frontier};

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
