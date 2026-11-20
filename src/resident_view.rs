use bevy::prelude::*;

use crate::{
    adventure::{Adventure, LANDMARKS, Target, landmark_center, landmark_offset, tracked_pin},
    garden::Garden,
    town::{Npc, QuestMark},
    ui::{self, HudLayout, UiFonts},
    world::{self, IslandCamera},
};

const MAP_SIZE: Vec2 = Vec2::new(96., 32.);
const NAME_SIZE: Vec2 = Vec2::new(78., 29.);
const SPOKEN_SIZE: Vec2 = Vec2::new(30., 30.);

#[derive(Clone, Copy)]
struct Pin {
    npc: Npc,
    mark: Option<QuestMark>,
    tracked: bool,
    anchor: Vec2,
    center: Vec2,
    half: Vec2,
}

#[derive(Resource, Default)]
pub struct ResidentPins {
    pins: [Option<Pin>; Npc::ALL.len()],
    map: bool,
}

impl ResidentPins {
    pub fn pick(&self, at: Vec2, map: bool) -> Option<Npc> {
        (self.map == map)
            .then(|| {
                self.pins
                    .iter()
                    .flatten()
                    .filter(|pin| {
                        (at - pin.center)
                            .abs()
                            .cmple(pin.half + Vec2::splat(3.))
                            .all()
                    })
                    .min_by(|left, right| {
                        at.distance_squared(left.center)
                            .total_cmp(&at.distance_squared(right.center))
                    })
                    .map(|pin| pin.npc)
            })
            .flatten()
    }
}

#[derive(Component)]
pub struct Badge(Npc);

#[derive(Component)]
pub struct Leader(Npc);

#[derive(Component)]
pub struct Sign(Npc);

#[derive(Component)]
pub struct Name;

pub fn spawn(root: &mut ChildSpawnerCommands, fonts: &UiFonts) {
    for npc in Npc::ALL {
        root.spawn((
            Leader(npc),
            Node {
                position_type: PositionType::Absolute,
                height: Val::Px(3.),
                ..default()
            },
            BorderRadius::MAX,
            ZIndex(-1),
            BackgroundColor(Color::srgb_u8(54, 82, 63)),
            Visibility::Hidden,
        ));
        root.spawn((
            Badge(npc),
            Node {
                position_type: PositionType::Absolute,
                height: Val::Px(SPOKEN_SIZE.y),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(6.),
                border: UiRect::all(Val::Px(1.5)),
                ..default()
            },
            BorderRadius::all(Val::Px(10.)),
            BorderColor(ui::stroke()),
            BackgroundColor(Color::srgb_u8(231, 186, 93)),
            Visibility::Hidden,
            GlobalZIndex(4),
            ui::shadow(),
        ))
        .with_children(|chip| {
            chip.spawn((Sign(npc), fonts.bold("!", 20.)));
            chip.spawn((Name, Node::default(), fonts.bold(npc.name(), 15.)));
        });
    }
}

fn place_badge(
    desired: Vec2,
    half: Vec2,
    occupied: &[(Vec2, Vec2)],
    layout: HudLayout,
) -> Option<Vec2> {
    // First use the NPC's own point on the map. Nearby residents get a
    // separate row so each name and quest symbol stays selectable.
    for vertical in [0., -37., 37., -74., 74., -111., 111., -148., 148.] {
        for horizontal in [0., -54., 54., -107., 107.] {
            let next = desired + Vec2::new(horizontal, vertical) * layout.scale;
            if layout.on_map_badge(next, half)
                && occupied.iter().all(|&(center, other)| {
                    (next - center)
                        .abs()
                        .cmpge(half + other + Vec2::splat(4.))
                        .any()
                })
            {
                return Some(next);
            }
        }
    }
    None
}

pub fn update(
    adventure: Res<Adventure>,
    garden: Res<Garden>,
    layout: Res<HudLayout>,
    camera: Single<(&Camera, &Transform), With<IslandCamera>>,
    mut pins: ResMut<ResidentPins>,
    mut badges: Query<
        (
            &Badge,
            &mut Node,
            &mut Visibility,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Without<Leader>,
    >,
    mut leaders: Query<
        (&Leader, &mut Node, &mut Transform, &mut Visibility),
        (Without<Badge>, Without<Name>, Without<IslandCamera>),
    >,
    mut signs: Query<
        (&Sign, &mut Text, &mut Node),
        (Without<Badge>, Without<Leader>, Without<Name>),
    >,
    mut names: Query<&mut Node, (With<Name>, Without<Badge>, Without<Leader>, Without<Sign>)>,
) {
    let (camera, pose) = *camera;
    let view = GlobalTransform::from(*pose);
    let is_map = adventure.map;
    pins.pins.fill(None);
    pins.map = is_map;
    if adventure.area.outdoors() {
        let mut occupied = Vec::with_capacity(LANDMARKS.len() + Npc::ALL.len() + 1);
        if is_map {
            for (_, target) in LANDMARKS {
                if let Some(center) = landmark_center(
                    camera,
                    &view,
                    adventure.position(target, &garden),
                    layout.scale,
                    landmark_offset(target),
                ) {
                    occupied.push((center, Vec2::new(53., 13.) * layout.scale));
                }
            }
            if let Some((target, center)) =
                tracked_pin(&adventure, &garden, camera, &view, layout.scale)
                && !matches!(target, Target::Person(_))
            {
                occupied.push((center, Vec2::new(53., 14.) * layout.scale));
            }
        }
        let tracked = adventure.tracked_target(&garden);
        let marks = Npc::ALL.map(|npc| npc.quest_mark(&garden));
        let mut people = Npc::ALL;
        if is_map {
            people.sort_by_key(|&npc| {
                std::cmp::Reverse((marks[npc as usize], tracked == Some(Target::Person(npc))))
            });
        }
        for npc in people {
            let index = npc as usize;
            let mark = marks[index];
            if !npc.available(&garden) || (!is_map && mark.is_none()) {
                continue;
            }
            let size = match (is_map, mark) {
                (true, Some(_)) => MAP_SIZE,
                (true, None) => NAME_SIZE,
                _ => SPOKEN_SIZE,
            };
            let half = size * layout.scale * 0.5;
            let point = npc.position(&garden);
            let height = world::surface_height(point, &garden) + if is_map { 0. } else { 1.92 };
            let Ok(anchor) = camera.world_to_viewport(&view, Vec3::new(point.x, height, point.y))
            else {
                continue;
            };
            if !layout.on_screen(anchor) {
                continue;
            }
            let center = if is_map {
                place_badge(
                    anchor - Vec2::Y * 20. * layout.scale,
                    half,
                    &occupied,
                    *layout,
                )
            } else {
                layout.on_map_badge(anchor, half).then_some(anchor)
            };
            if let Some(center) = center {
                occupied.push((center, half));
                pins.pins[index] = Some(Pin {
                    npc,
                    mark,
                    tracked: tracked == Some(Target::Person(npc)),
                    anchor,
                    center,
                    half,
                });
            }
        }
    }
    for (leader, mut node, mut transform, mut visibility) in &mut leaders {
        let Some(pin) = pins.pins[leader.0 as usize].filter(|_| is_map) else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let delta = (pin.center - pin.anchor) / layout.scale;
        let length = delta.length() + 3.;
        let center = layout.hud_position(pin.anchor.lerp(pin.center, 0.5));
        node.set_if_neq(Node {
            left: Val::Px(center.x - length / 2.),
            top: Val::Px(center.y - 1.5),
            width: Val::Px(length),
            height: Val::Px(3.),
            position_type: PositionType::Absolute,
            ..default()
        });
        transform.rotation = Quat::from_rotation_z(delta.y.atan2(delta.x));
        visibility.set_if_neq(Visibility::Inherited);
    }
    for (badge, mut node, mut visible, mut color, mut border) in &mut badges {
        if let Some(pin) = pins.pins[badge.0 as usize] {
            let top = layout.hud_position(pin.center - pin.half);
            let size = pin.half * (2. / layout.scale);
            let fill = match pin.mark {
                Some(QuestMark::Available) => Color::srgb_u8(238, 191, 101),
                Some(QuestMark::TurnIn) => Color::srgb_u8(149, 214, 170),
                None => ui::cream(),
            };
            node.reborrow()
                .map_unchanged(|node| &mut node.left)
                .set_if_neq(Val::Px(top.x));
            node.reborrow()
                .map_unchanged(|node| &mut node.top)
                .set_if_neq(Val::Px(top.y));
            node.reborrow()
                .map_unchanged(|node| &mut node.width)
                .set_if_neq(Val::Px(size.x));
            node.reborrow()
                .map_unchanged(|node| &mut node.height)
                .set_if_neq(Val::Px(size.y));
            color.set_if_neq(BackgroundColor(fill));
            border.set_if_neq(if pin.tracked {
                BorderColor(Color::srgb_u8(60, 120, 113))
            } else {
                BorderColor(ui::stroke())
            });
            visible.set_if_neq(Visibility::Inherited);
        } else {
            visible.set_if_neq(Visibility::Hidden);
        }
    }
    for (sign, mut text, mut node) in &mut signs {
        let mark = pins.pins[sign.0 as usize].and_then(|pin| pin.mark);
        if let Some(mark) = mark {
            if text.0 != mark.symbol() {
                text.0 = mark.symbol().into();
            }
        }
        let display = if mark.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        node.reborrow()
            .map_unchanged(|node| &mut node.display)
            .set_if_neq(display);
    }
    for mut node in &mut names {
        let display = if is_map { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_map_pins_have_distinct_tap_targets() {
        let anchor = Vec2::new(620., 290.);
        let half = MAP_SIZE * 0.5;
        let mut occupied = vec![(anchor, Vec2::new(53., 13.))];
        let mut pins = ResidentPins {
            map: true,
            ..default()
        };
        for npc in [Npc::Mara, Npc::Fiete, Npc::Bo, Npc::Juna] {
            let center = place_badge(anchor, half, &occupied, HudLayout::default()).unwrap();
            occupied.push((center, half));
            pins.pins[npc as usize] = Some(Pin {
                npc,
                mark: Some(QuestMark::Available),
                tracked: false,
                anchor,
                center,
                half,
            });
        }
        for pin in pins.pins.into_iter().flatten() {
            assert_eq!(pins.pick(pin.center, true), Some(pin.npc));
            assert_eq!(pins.pick(pin.center, false), None);
        }
        assert_eq!(pins.pick(anchor, true), None);
    }
}
