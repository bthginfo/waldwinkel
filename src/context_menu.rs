use bevy::{prelude::*, ui::FocusPolicy, window::PrimaryWindow};

use crate::{
    Adventure, Controls, Garden, GardenFocus,
    adventure::{PadTarget, Target},
    garden::{Action, Stage},
    ui::{self, Hud, HudLayout, HudPanel, UiFonts},
    world::{self, IslandCamera, PLOTS},
};

const WIDTH: f32 = 196.;
const MAIN: f32 = 56.;
const EXTRAS: f32 = 46.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Command {
    Use,
    Seeds,
    Fertilize,
    Read,
}

impl Command {
    pub fn apply(self, controls: &mut Controls) {
        match self {
            Self::Use => controls.act = true,
            Self::Seeds => controls.choice = true,
            Self::Fertilize => controls.fertilize = true,
            Self::Read => {}
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct HitArea {
    rect: Rect,
    target: PadTarget,
    primary: Command,
    extra: [Option<Command>; 2],
}

impl HitArea {
    pub fn at(self, at: Vec2, scale: f32) -> Option<(PadTarget, Command)> {
        if !self.rect.contains(at) {
            return None;
        }
        let p = (at - self.rect.min) / scale;
        let command = if p.y < MAIN {
            self.primary
        } else if self.extra[1].is_some() && p.x >= WIDTH / 2. {
            self.extra[1]?
        } else {
            self.extra[0]?
        };
        Some((self.target, command))
    }
}

#[derive(Component)]
pub struct Popup;
#[derive(Component)]
pub struct Title;
#[derive(Component)]
pub struct Verb;
#[derive(Component)]
pub struct Icon;
#[derive(Component)]
pub struct BiteBar;
#[derive(Component)]
pub struct OptionRow(Command);
#[derive(Component)]
pub struct OptionText(Command);

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>, assets: Res<AssetServer>) {
    let px = Val::Px;
    commands
        .spawn((
            Popup,
            GlobalZIndex(4),
            FocusPolicy::Pass,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: px(WIDTH),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
                ..default()
            },
            BorderRadius::all(px(13.)),
            ui::shadow(),
            BackgroundColor(Color::srgb_u8(38, 70, 57)),
        ))
        .with_children(|card| {
            card.spawn((Node {
                height: px(MAIN),
                width: Val::Percent(100.),
                align_items: AlignItems::Center,
                column_gap: px(8.),
                padding: UiRect::horizontal(px(9.)),
                ..default()
            },))
                .with_children(|main| {
                    main.spawn((
                        BiteBar,
                        Node {
                            display: Display::None,
                            position_type: PositionType::Absolute,
                            left: px(0.),
                            bottom: px(0.),
                            height: px(4.),
                            ..default()
                        },
                        BackgroundColor(Color::srgb_u8(255, 196, 82)),
                    ));
                    main.spawn((
                        Node {
                            width: px(34.),
                            height: px(34.),
                            flex_shrink: 0.,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderRadius::all(px(10.)),
                        BackgroundColor(ui::cream()),
                    ))
                    .with_child((
                        Icon,
                        ImageNode::default(),
                        Node {
                            width: px(28.),
                            height: px(28.),
                            ..default()
                        },
                    ));
                    main.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        min_width: px(0.),
                        ..default()
                    })
                    .with_children(|labels| {
                        let (title, face, _) = fonts.body("", 13.);
                        labels.spawn((
                            Title,
                            title,
                            face,
                            TextColor(Color::srgb_u8(225, 223, 191)),
                        ));
                        labels.spawn((Verb, fonts.button("", 16.)));
                    });
                });
            card.spawn((
                Node {
                    width: Val::Percent(100.),
                    ..default()
                },
                BackgroundColor(ui::cream()),
                BorderRadius::bottom(px(13.)),
            ))
            .with_children(|row| {
                for (command, icon) in [
                    (Command::Seeds, "ui/seeds.png"),
                    (Command::Fertilize, "ui/items/fertilizer.png"),
                ] {
                    row.spawn((
                        OptionRow(command),
                        Node {
                            flex_grow: 1.,
                            flex_basis: px(0.),
                            height: px(EXTRAS),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            column_gap: px(3.),
                            border: UiRect::left(px(if command == Command::Fertilize {
                                1.
                            } else {
                                0.
                            })),
                            ..default()
                        },
                        BorderColor(Color::srgb_u8(211, 202, 170)),
                    ))
                    .with_children(|cell| {
                        cell.spawn((
                            ImageNode::new(assets.load(icon)),
                            Node {
                                width: px(17.),
                                height: px(17.),
                                flex_shrink: 0.,
                                ..default()
                            },
                        ));
                        cell.spawn((OptionText(command), fonts.bold("", 13.)));
                    });
                }
            });
        });
}

fn position(
    anchor: Vec2,
    size: Vec2,
    hud: HudLayout,
    tutorial: bool,
    tablet: bool,
) -> Option<Rect> {
    if !hud.on_screen(anchor) {
        return None;
    }
    let safe = hud.safe_rect();
    let scale = hud.scale;
    let bounds = safe.size() / scale;
    let local = (anchor - safe.min) / scale;
    let gap = 15.;
    let candidates = [
        local + Vec2::new(gap, -size.y * 0.55),
        local - Vec2::new(gap + size.x, size.y * 0.55),
        local - Vec2::new(size.x * 0.5, size.y + gap),
        local + Vec2::new(-size.x * 0.5, gap),
    ];
    let rect = |x1, y1, x2, y2| Rect::new(x1, y1, x2, y2);
    for candidate in candidates {
        let corner = candidate.clamp(Vec2::splat(5.), (bounds - size - 5.).max(Vec2::splat(5.)));
        let box_ui = Rect::from_corners(corner, corner + size);
        if box_ui.contains(local) {
            continue;
        }
        let (w, h) = (bounds.x, bounds.y);
        let occupied = [
            rect(0., 0., 261., if hud.tablet_pad { 83. } else { 281. }),
            rect(w - 422., 0., w, 82.),
            rect(0., h - 140., if hud.tablet_pad { 256. } else { 145. }, h),
            rect(w - 219., h - if hud.tablet_pad { 105. } else { 227. }, w, h),
            rect(w * 0.5 - 165., h - 102., w * 0.5 + 165., h),
        ];
        if occupied
            .iter()
            .any(|other| !box_ui.intersect(*other).is_empty())
            || tutorial
                && !box_ui
                    .intersect(rect(w * 0.30, h - 216., w * 0.69, h - 127.))
                    .is_empty()
            || tablet && !box_ui.intersect(rect(w - 200., 80., w, 226.)).is_empty()
        {
            continue;
        }
        return Some(Rect::from_corners(
            safe.min + corner * scale,
            safe.min + (corner + size) * scale,
        ));
    }
    None
}

fn plot_commands(
    garden: &Garden,
    plot: usize,
    panel: &ui::Panel,
) -> (Command, [Option<Command>; 2]) {
    if garden.clock.night() {
        return (Command::Read, [None; 2]);
    }
    let seeds = garden.plots[plot].stage() == Stage::Soil;
    let primary = if panel.enabled {
        Command::Use
    } else {
        Command::Read
    };
    let mut extras = [None; 2];
    if seeds && panel.fallback != "Saatgut wählen" {
        extras[0] = Some(Command::Seeds);
    }
    if garden.fertilizable(plot) && garden.fertilizer > 0 && panel.action != Some(Action::Fertilize)
    {
        let i = usize::from(extras[0].is_some());
        extras[i] = Some(Command::Fertilize);
    }
    (primary, extras)
}

fn icon_name(target: PadTarget, action: Option<Action>) -> &'static str {
    match target {
        PadTarget::Plot(_) => match action {
            Some(Action::Till) => "items/tool_hoe_0",
            Some(Action::Sow) => "seeds",
            Some(Action::Water) => "items/tool_can_0",
            Some(Action::Fertilize) => "items/fertilizer",
            Some(Action::Harvest) => "carrot",
            None => "sprout",
        },
        PadTarget::Place(t) => match t {
            Target::Tree(_) => "items/tool_axe_0",
            Target::Rock(_) | Target::MineRock(_) => "items/tool_pickaxe_0",
            Target::Bed | Target::Door | Target::Exit => "items/plan_room",
            Target::Pier | Target::HarborFishing(_) => "items/fish",
            Target::Person(_) | Target::HarborBoard | Target::Watch(_) => "journal",
            Target::Kitchen => "station_cook",
            Target::SaleChest | Target::EveningMarket | Target::Village(_) => "coin",
            Target::Binder | Target::DuskFoe => "card_strike",
            Target::Find(_) | Target::SeasonFind(_) | Target::HarborDriftwood => "bag",
            _ => "choose",
        },
    }
}

pub fn update(
    adventure: Res<Adventure>,
    garden: Res<Garden>,
    focus: Res<GardenFocus>,
    panel: Res<HudPanel>,
    mut hud: ResMut<HudLayout>,
    visible: Single<&Visibility, With<Hud>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<IslandCamera>>,
    assets: Res<AssetServer>,
    mut previous_icon: Local<Option<&'static str>>,
    time: Res<Time>,
    mut popup: Single<&mut Node, (With<Popup>, Without<OptionRow>)>,
    mut rows: Query<(&OptionRow, &mut Node), (Without<Popup>, Without<BiteBar>)>,
    mut bite_bar: Single<&mut Node, (With<BiteBar>, Without<Popup>)>,
    mut labels: Query<
        (
            Option<&Title>,
            Option<&Verb>,
            Option<&OptionText>,
            &mut Text,
        ),
        Or<(With<Title>, With<Verb>, With<OptionText>)>,
    >,
    mut picture: Single<&mut ImageNode, With<Icon>>,
) {
    let selection = focus
        .selected
        .map(PadTarget::Plot)
        .or(adventure.focus.map(PadTarget::Place));
    let display = || {
        let panel = panel.0.as_ref()?;
        let target = selection?;
        if **visible == Visibility::Hidden || adventure.map {
            return None;
        }
        let (p, height, primary, extras) = match target {
            PadTarget::Plot(i) => {
                let (primary, extras) = plot_commands(&garden, i, panel);
                (PLOTS[i], 0.34, primary, extras)
            }
            PadTarget::Place(t) => (
                adventure.position(t, &garden),
                t.height(),
                if panel.enabled {
                    Command::Use
                } else {
                    Command::Read
                },
                [None; 2],
            ),
        };
        let ground = if adventure.area.outdoors() {
            world::surface_height(p, &garden)
        } else {
            0.
        };
        let anchor = camera
            .0
            .world_to_viewport(camera.1, Vec3::new(p.x, ground + height, p.y))
            .ok()?;
        let rect = position(
            anchor,
            Vec2::new(WIDTH, MAIN + if extras[0].is_some() { EXTRAS } else { 0. }),
            *hud,
            garden.prologue.is_some(),
            adventure.area.outdoors() && window.width().min(window.height()) >= 590.,
        )?;
        Some((
            HitArea {
                rect,
                target,
                primary,
                extra: extras,
            },
            panel,
        ))
    };
    let Some((area, panel)) = display() else {
        popup.display = Display::None;
        hud.bubble = None;
        return;
    };
    hud.bubble = Some(area);
    popup.display = Display::Flex;
    popup.left = Val::Px(area.rect.min.x / hud.scale);
    popup.top = Val::Px(area.rect.min.y / hud.scale);
    for (row, mut node) in &mut rows {
        node.display = if area.extra.contains(&Some(row.0)) {
            Display::Flex
        } else {
            Display::None
        };
    }
    if let Some(remaining) = adventure.bite_remaining(time.elapsed_secs()) {
        bite_bar.display = Display::Flex;
        bite_bar.width = Val::Percent(100. * remaining);
    } else {
        bite_bar.display = Display::None;
    }
    let name = icon_name(area.target, panel.action);
    if *previous_icon != Some(name) {
        picture.image = assets.load(format!("ui/{name}.png"));
        *previous_icon = Some(name);
    }
    for (title, verb, option, mut text) in &mut labels {
        if title.is_some() {
            if text.0 != panel.title {
                text.0.clone_from(&panel.title);
            }
            continue;
        }
        let label = if verb.is_some() {
            panel.action.map_or(panel.fallback, Action::label)
        } else {
            match option.map(|i| i.0) {
                Some(Command::Seeds) => {
                    if panel.pad {
                        "LT · Saatgut"
                    } else {
                        "Saatgut"
                    }
                }
                Some(Command::Fertilize) => {
                    if panel.pad {
                        "RT · Düngen"
                    } else {
                        "Düngen"
                    }
                }
                _ => "",
            }
        };
        if verb.is_some() && panel.pad && area.primary == Command::Use {
            if text.0.strip_prefix("A · ") != Some(label) {
                text.0 = format!("A · {label}");
            }
        } else if text.0 != label {
            text.0 = label.into();
        }
    }
}

pub fn compact_dock(hud: Res<HudLayout>, mut fields: Query<(&ui::Field, &mut Node)>) {
    for (field, mut node) in &mut fields {
        if matches!(field, ui::Field::Context) {
            let display = if hud.bubble.is_some() {
                Display::None
            } else {
                Display::Flex
            };
            if node.display != display {
                node.display = display;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacent_farm_choices_and_scaled_touch_targets_agree() {
        for scale in [0.83, 1., 1.48] {
            let target = PadTarget::Plot(3);
            let origin = Vec2::new(327., 191.);
            let area = HitArea {
                rect: Rect::from_corners(origin, origin + Vec2::new(WIDTH, MAIN + EXTRAS) * scale),
                target,
                primary: Command::Use,
                extra: [Some(Command::Seeds), Some(Command::Fertilize)],
            };
            for (x, y, command) in [
                (35., 20., Command::Use),
                (45., 75., Command::Seeds),
                (133., 75., Command::Fertilize),
            ] {
                assert_eq!(
                    area.at(origin + Vec2::new(x, y) * scale, scale),
                    Some((target, command))
                );
            }
            assert_eq!(area.at(origin - Vec2::ONE, scale), None);
        }
    }

    #[test]
    fn context_sits_next_to_target_and_inside_screen() {
        let hud = HudLayout::default();
        for anchor in [
            Vec2::new(640., 350.),
            Vec2::new(1254., 372.),
            Vec2::new(25., 362.),
        ] {
            let rect = position(anchor, Vec2::new(WIDTH, MAIN + EXTRAS), hud, true, true).unwrap();
            assert!(!rect.contains(anchor));
            assert!(hud.on_screen(rect.min) && hud.on_screen(rect.max));
        }
        assert!(position(Vec2::splat(-1.), Vec2::new(WIDTH, MAIN), hud, false, false).is_none());
    }
}
