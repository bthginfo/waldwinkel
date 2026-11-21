use bevy::{prelude::*, ui::FocusPolicy, window::PrimaryWindow};

use crate::{
    Adventure, Garden, Ready,
    session::Session,
    terrain,
    ui::{SafeInsets, UiFonts},
    world::{Gardener, IslandCamera},
};

pub(crate) const MAP_WIDTH: f32 = 176.;
const GLINTS: usize = 8;

#[derive(Component)]
pub enum Marker {
    QuestArea,
    Glint(usize),
    Mini,
    You,
    Objective,
}

#[derive(Component)]
pub struct Bearing;

pub fn setup(mut commands: Commands, assets: Res<AssetServer>, fonts: Res<UiFonts>) {
    let pin = |size: f32| Node {
        display: Display::None,
        position_type: PositionType::Absolute,
        width: Val::Px(size),
        height: Val::Px(size),
        ..default()
    };
    commands.spawn((
        Marker::QuestArea,
        pin(60.),
        GlobalZIndex(1),
        FocusPolicy::Pass,
        BackgroundColor(Color::srgba(0.93, 0.71, 0.28, 0.18)),
        BorderColor(Color::srgba(0.96, 0.77, 0.39, 0.84)),
        BorderRadius::MAX,
    ));
    let shine = assets.load("ui/gather_glint.png");
    for i in 0..GLINTS {
        commands.spawn((
            Marker::Glint(i),
            pin(15.),
            ImageNode::new(shine.clone()),
            GlobalZIndex(2),
            FocusPolicy::Pass,
        ));
    }
    let map_height = MAP_WIDTH * terrain::atlas_ratio();
    commands
        .spawn((
            Marker::Mini,
            GlobalZIndex(3),
            FocusPolicy::Pass,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                padding: UiRect::all(Val::Px(6.)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: Val::Px(3.),
                ..default()
            },
            ui_background(),
            BorderRadius::all(Val::Px(13.)),
        ))
        .with_children(|map| {
            map.spawn((
                ImageNode::new(assets.load("ui/island_atlas.png")),
                Node {
                    width: Val::Px(MAP_WIDTH),
                    height: Val::Px(map_height),
                    ..default()
                },
                BorderRadius::all(Val::Px(8.)),
                FocusPolicy::Pass,
            ))
            .with_children(|atlas| {
                atlas.spawn((
                    Marker::Objective,
                    pin(12.),
                    BackgroundColor(Color::srgb_u8(252, 190, 63)),
                    BorderRadius::MAX,
                    FocusPolicy::Pass,
                ));
                atlas.spawn((
                    Marker::You,
                    pin(9.),
                    BackgroundColor(Color::srgb_u8(255, 255, 242)),
                    BorderRadius::MAX,
                    FocusPolicy::Pass,
                ));
            });
            map.spawn((Bearing, fonts.button("", 12.)));
        });
}

fn ui_background() -> BackgroundColor {
    BackgroundColor(Color::srgba(0.12, 0.25, 0.21, 0.94))
}

#[derive(Default)]
pub struct Nearby {
    next: f32,
    marks: Vec<(Vec3, Color)>,
}

pub fn update(
    time: Res<Time>,
    ready: Res<Ready>,
    session: Res<Session>,
    layers: (
        Res<crate::farm_ui::FarmMenu>,
        Res<crate::town::Dialog>,
        Res<crate::wheel::Wheel>,
        Res<crate::night::Night>,
        Res<crate::decorate::Layout>,
    ),
    adventure: Res<Adventure>,
    garden: Res<Garden>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<IslandCamera>>,
    player: Single<&Gardener>,
    scale: Res<UiScale>,
    insets: Res<SafeInsets>,
    mut nearby: Local<Nearby>,
    mut markers: Query<(&Marker, &mut Node, Option<&mut ImageNode>)>,
    mut bearing: Single<&mut Text, With<Bearing>>,
) {
    let (menu, dialog, wheel, night, furnishing) = layers;
    let open = session.playing()
        && ready.0
        && !menu.open()
        && !dialog.open()
        && !wheel.open()
        && !night.active() && !furnishing.open();
    let overview = open && adventure.map && adventure.area.outdoors();
    let tablet = open
        && !adventure.map
        && adventure.area.outdoors()
        && window.width().min(window.height()) >= 590.;
    let elapsed = time.elapsed_secs();
    if !open || adventure.map {
        nearby.marks.clear();
        nearby.next = 0.;
    } else if elapsed >= nearby.next {
        nearby.marks = adventure.nearby_resources(&garden, player.position);
        nearby.next = elapsed + 0.16;
    }
    let task = (overview || tablet)
        .then(|| adventure.tracked_target(&garden))
        .flatten();
    let destination = task.map(|target| {
        let position = adventure.position(target, &garden);
        if target.is_search_area() {
            (position / 7.).round() * 7.
        } else {
            position
        }
    });
    let (camera, view) = *camera;
    let project = |point: Vec3| {
        camera
            .world_to_viewport(view, point)
            .ok()
            .map(|p| p / scale.0)
    };
    let map_height = MAP_WIDTH * terrain::atlas_ratio();
    for (mark, mut node, texture) in &mut markers {
        node.display = Display::None;
        match *mark {
            Marker::QuestArea if overview => {
                let Some(at) = destination else { continue };
                let world = |p: Vec2| Vec3::new(p.x, 0.85, p.y);
                let Some(center) = project(world(at)) else {
                    continue;
                };
                let size = project(world(at + Vec2::new(16., 0.)))
                    .map_or(Vec2::splat(36.), |edge| {
                        Vec2::new(
                            (edge.x - center.x).abs() * 2.,
                            (edge.x - center.x).abs() * 1.0,
                        )
                    })
                    .max(Vec2::new(47., 33.));
                node.display = Display::Flex;
                node.left = Val::Px(center.x - size.x * 0.5);
                node.top = Val::Px(center.y - size.y * 0.5);
                node.width = Val::Px(size.x);
                node.height = Val::Px(size.y);
                node.border = UiRect::all(Val::Px(2.));
            }
            Marker::Glint(i) => {
                let Some(&(point, tint)) = nearby.marks.get(i) else {
                    continue;
                };
                let Some(center) = project(point) else {
                    continue;
                };
                let pulse = 0.5 + 0.5 * (elapsed * 2.75 + i as f32 * 1.9).sin();
                let size = 12. + pulse * 3.;
                node.display = Display::Flex;
                node.left = Val::Px(center.x - size * 0.5);
                node.top = Val::Px(center.y - size * 0.5);
                node.width = Val::Px(size);
                node.height = Val::Px(size);
                if let Some(mut texture) = texture {
                    texture.color = tint.with_alpha(0.46 + pulse * 0.42);
                }
            }
            Marker::Mini if tablet => {
                node.display = Display::Flex;
                node.right = Val::Px(14. + insets.0[2] / window.scale_factor() / scale.0);
                node.top = Val::Px(85. + insets.0[1] / window.scale_factor() / scale.0);
            }
            Marker::You | Marker::Objective if tablet => {
                let point = if matches!(mark, Marker::You) {
                    Some(player.position)
                } else {
                    destination
                };
                let Some(point) = point else { continue };
                let center = terrain::atlas_position(point) * Vec2::new(MAP_WIDTH, map_height);
                node.display = Display::Flex;
                node.left = Val::Px(center.x - 5.);
                node.top = Val::Px(center.y - 5.);
            }
            _ => {}
        }
    }
    if tablet {
        let text = destination.map_or_else(
            || "Norden ↑".into(),
            |point| {
                let offset = point - player.position;
                if offset.length_squared() < 14. * 14. {
                    "Auftragsziel in der Nähe".into()
                } else {
                    let angle = offset.x.atan2(-offset.y);
                    let octant = ((angle / std::f32::consts::TAU * 8.).round() as isize)
                        .rem_euclid(8) as usize;
                    let arrow = ["↑", "↗", "→", "↘", "↓", "↙", "←", "↖"][octant];
                    let region = crate::travel::District::at(point)
                        .map_or("Hof", crate::travel::District::name);
                    format!("{arrow} {region}")
                }
            },
        );
        if bearing.0 != text {
            bearing.0 = text;
        }
    }
}
