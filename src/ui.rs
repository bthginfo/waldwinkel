use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    GardenFocus, Notice, Ready,
    adventure::{Adventure, Area, LANDMARKS, Target, landmark_center},
    garden::{Action, Crop, Garden, Stage},
    homestead::{self, Product},
    input::Gesture,
    session::Session,
    town::{self, Dialog},
    travel::District,
    weather::{Season, Sky},
    world::{Gardener, IslandCamera, PLOTS, can_work},
};

const MARGIN: f32 = 23.0;
const STICK_SIZE: f32 = 108.0;
const BUTTON_WIDTH: f32 = 188.0;
const BUTTON_HEIGHT: f32 = 62.0;
const BUTTON_BOTTOM: f32 = 29.0;
const FERTILIZER_BOTTOM: f32 = BUTTON_BOTTOM + BUTTON_HEIGHT + 10.0;
const FERTILIZER_HEIGHT: f32 = 41.0;
const FERTILIZER_WIDTH: f32 = 119.0;
const CONTEXT_WIDTH: f32 = 316.0;
const TOOLS_TOP: f32 = 88.;
const TOOL_WIDTH: f32 = 105.;
const TOOL_HEIGHT: f32 = 41.;
const TOOL_STEP: f32 = 47.;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Utility {
    In,
    Out,
    Map,
    Choice,
    Task,
    Inventory,
    Menu,
    Fertilize,
}
impl Utility {
    const ALL: [Self; 7] = [
        Self::In,
        Self::Out,
        Self::Map,
        Self::Choice,
        Self::Task,
        Self::Inventory,
        Self::Menu,
    ];
    pub fn apply(self, controls: &mut crate::Controls) {
        match self {
            Self::In => controls.zoom += 0.21,
            Self::Out => controls.zoom -= 0.21,
            Self::Map => controls.map = true,
            Self::Choice => controls.choice = true,
            Self::Task => controls.task = true,
            Self::Inventory => controls.inventory = true,
            Self::Menu => controls.menu = true,
            Self::Fertilize => controls.fertilize = true,
        }
    }
}

#[derive(Component)]
pub struct UtilityLabel(Utility);

pub(crate) fn cream() -> Color {
    Color::srgb_u8(252, 240, 213)
}
fn ink() -> Color {
    Color::srgb_u8(43, 67, 52)
}
fn muted() -> Color {
    Color::srgb_u8(86, 100, 71)
}
fn sage() -> Color {
    Color::srgb_u8(220, 224, 184)
}
pub(crate) fn stroke() -> Color {
    Color::srgba(0.99, 0.95, 0.84, 0.78)
}
pub(crate) fn shadow() -> BoxShadow {
    BoxShadow::new(
        Color::srgba(0.14, 0.26, 0.19, 0.21),
        px(0.0),
        px(3.0),
        px(1.0),
        px(9.0),
    )
}
fn px(val: f32) -> Val {
    Val::Px(val)
}

#[derive(Clone, Copy, Resource)]
pub struct HudLayout {
    size: Vec2,
    pub scale: f32,
    context_height: f32,
    fertilizer: bool,
}

impl Default for HudLayout {
    fn default() -> Self {
        Self {
            size: Vec2::new(1280.0, 720.0),
            scale: 1.0,
            context_height: 65.,
            fertilizer: false,
        }
    }
}

impl HudLayout {
    pub fn stick_center(self) -> Vec2 {
        Vec2::new(
            (MARGIN + STICK_SIZE * 0.5) * self.scale,
            self.size.y - (MARGIN + STICK_SIZE * 0.5) * self.scale,
        )
    }

    pub fn on_stick(self, at: Vec2) -> bool {
        at.distance_squared(self.stick_center()) <= ((STICK_SIZE * 0.5 + 5.0) * self.scale).powi(2)
    }

    pub fn on_action(self, at: Vec2) -> bool {
        let right = (self.size.x - at.x) / self.scale;
        let bottom = (self.size.y - at.y) / self.scale;
        (MARGIN..=MARGIN + BUTTON_WIDTH).contains(&right)
            && (BUTTON_BOTTOM..=BUTTON_BOTTOM + BUTTON_HEIGHT).contains(&bottom)
    }

    pub fn utility(self, at: Vec2) -> Option<Utility> {
        let right = (self.size.x - at.x) / self.scale;
        let bottom = (self.size.y - at.y) / self.scale;
        if self.fertilizer
            && (MARGIN..=MARGIN + FERTILIZER_WIDTH).contains(&right)
            && (FERTILIZER_BOTTOM..=FERTILIZER_BOTTOM + FERTILIZER_HEIGHT).contains(&bottom)
        {
            return Some(Utility::Fertilize);
        }
        let point = at / self.scale - Vec2::new(MARGIN, TOOLS_TOP);
        if point.x < 0. || point.y < 0. {
            return None;
        }
        let row = (point.y / TOOL_STEP) as usize;
        if point.y % TOOL_STEP > TOOL_HEIGHT {
            return None;
        }
        if row == 0 && point.x <= TOOL_WIDTH {
            return Some(if point.x < TOOL_WIDTH / 2. {
                Utility::Out
            } else {
                Utility::In
            });
        }
        if point.x > TOOL_WIDTH {
            return None;
        }
        Utility::ALL.get(row + 1).copied()
    }

    pub fn on_other_ui(self, at: Vec2) -> bool {
        if self.utility(at).is_some() {
            return true;
        }
        let top = at.y / self.scale;
        let left = at.x / self.scale;
        let right = (self.size.x - at.x) / self.scale;
        let bottom = (self.size.y - at.y) / self.scale;
        ((MARGIN..=MARGIN + 55.0).contains(&top)
            && ((MARGIN..=MARGIN + 188.0).contains(&left)
                || (MARGIN..=MARGIN + 300.0).contains(&right)))
            || ((BUTTON_BOTTOM..=BUTTON_BOTTOM + self.context_height).contains(&bottom)
                && (at.x - self.size.x * 0.5).abs() < CONTEXT_WIDTH * self.scale * 0.5)
    }
}

pub fn resize(
    adventure: Res<Adventure>,
    focus: Res<GardenFocus>,
    garden: Res<Garden>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut layout: ResMut<HudLayout>,
    mut scale: ResMut<UiScale>,
) {
    let size = Vec2::new(window.width(), window.height());
    let context_height = if adventure.inventory {
        166.
    } else if adventure.journal {
        109.
    } else {
        65.
    };
    let fertilizer =
        focus.selected.is_some_and(|i| garden.fertilizable(i)) && !garden.clock.night();
    if size != layout.size
        || layout.context_height != context_height
        || layout.fertilizer != fertilizer
    {
        let fit = (size.y / 680.0)
            .max(0.78)
            .min(size.y / 505.0)
            .min(size.x / 728.0)
            .clamp(0.42, 1.48);
        *layout = HudLayout {
            size,
            scale: fit,
            context_height,
            fertilizer,
        };
        scale.0 = fit;
    }
}

#[derive(Component)]
pub struct Landmark(Target);

pub fn map_labels(
    adventure: Res<Adventure>,
    layout: Res<HudLayout>,
    camera: Single<(&Camera, &GlobalTransform), With<IslandCamera>>,
    mut labels: Query<(&Landmark, &mut Node, &mut Visibility)>,
) {
    let (camera, view) = *camera;
    for (label, mut node, mut visible) in &mut labels {
        let position = if adventure.map && adventure.area.outdoors() {
            landmark_center(camera, view, adventure.position(label.0), layout.scale)
        } else {
            None
        };
        if let Some(p) = position {
            *visible = Visibility::Inherited;
            node.left = px(p.x / layout.scale - 53.);
            node.top = px(p.y / layout.scale - 12.);
        } else {
            *visible = Visibility::Hidden;
        }
    }
}

#[derive(Component)]
pub struct Hud;

#[derive(Component)]
pub struct Loading;

#[derive(Component)]
pub struct Thumb;

#[derive(Component)]
pub struct ActionSurface;

#[derive(Component)]
pub struct FertilizerButton;

#[derive(Component)]
pub struct ProgressTrack;

#[derive(Component)]
pub struct Progress;

#[derive(Component)]
pub struct ActionIcon;

#[derive(Component)]
pub enum Field {
    Day,
    Coins,
    Weather,
    Context,
    Detail,
    Action,
}

#[derive(Resource)]
pub struct Icons {
    sprout: Handle<Image>,
    hoe: Handle<Image>,
    choose: Handle<Image>,
    carrot: Handle<Image>,
    seeds: Handle<Image>,
    watering: Handle<Image>,
    fertilizer: Handle<Image>,
    day: Handle<Image>,
    coin: Handle<Image>,
    stick: Handle<Image>,
}

impl FromWorld for Icons {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            sprout: assets.load("ui/sprout.png"),
            hoe: assets.load("ui/hoe.png"),
            choose: assets.load("ui/choose.png"),
            carrot: assets.load("ui/carrot.png"),
            seeds: assets.load("ui/seeds.png"),
            watering: assets.load("ui/watering.png"),
            fertilizer: assets.load("ui/fertilizer.png"),
            day: assets.load("ui/day.png"),
            coin: assets.load("ui/coin.png"),
            stick: assets.load("ui/stick.png"),
        }
    }
}

fn icon(texture: &Handle<Image>, size: f32) -> (ImageNode, Node) {
    (
        ImageNode::new(texture.clone()),
        Node {
            width: px(size),
            height: px(size),
            flex_shrink: 0.0,
            ..default()
        },
    )
}

fn text(value: &str, font: &Handle<Font>, size: f32, color: Color) -> (Text, TextFont, TextColor) {
    (
        Text::new(value),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(color),
    )
}

#[derive(Resource)]
pub struct UiFonts {
    serif: Handle<Font>,
    regular: Handle<Font>,
    bold: Handle<Font>,
}

impl FromWorld for UiFonts {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            serif: assets.load("fonts/DMSerifText-Regular.ttf"),
            regular: assets.load("fonts/DMSans-Medium.ttf"),
            bold: assets.load("fonts/DMSans-Bold.ttf"),
        }
    }
}

impl UiFonts {
    pub fn title(&self, value: &str, size: f32) -> (Text, TextFont, TextColor) {
        text(value, &self.serif, size, ink())
    }
    pub fn body(&self, value: &str, size: f32) -> (Text, TextFont, TextColor) {
        text(value, &self.regular, size, ink())
    }
    pub fn bold(&self, value: &str, size: f32) -> (Text, TextFont, TextColor) {
        text(value, &self.bold, size, ink())
    }
    pub fn button(&self, value: &str, size: f32) -> (Text, TextFont, TextColor) {
        text(value, &self.bold, size, cream())
    }
}

pub fn button_color(interaction: Interaction, enabled: bool) -> Color {
    if !enabled {
        return Color::srgb_u8(130, 130, 108);
    }
    match interaction {
        Interaction::Pressed => Color::srgb_u8(45, 67, 52),
        Interaction::Hovered => Color::srgb_u8(79, 109, 79),
        Interaction::None => Color::srgb_u8(65, 94, 70),
    }
}

pub fn dialog_button(width: f32) -> (Button, Node, BorderRadius, BackgroundColor) {
    (
        Button,
        Node {
            width: px(width),
            min_height: px(44.),
            padding: UiRect::axes(px(8.), px(4.)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BorderRadius::all(px(12.)),
        BackgroundColor(button_color(Interaction::None, true)),
    )
}

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>, icons: Res<Icons>) {
    let title = fonts.serif.clone();
    let regular = fonts.regular.clone();
    let bold = fonts.bold.clone();
    commands
        .spawn((
            Loading,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(51.0),
                width: px(252.0),
                height: px(58.0),
                margin: UiRect {
                    left: px(-126.0),
                    ..default()
                },
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderRadius::all(px(22.0)),
            BackgroundColor(cream()),
            shadow(),
        ))
        .with_child(text("Waldwinkel lädt …", &title, 22.0, ink()));
    commands
        .spawn((
            Hud,
            Visibility::Hidden,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(MARGIN),
                    top: px(MARGIN),
                    width: px(188.0),
                    height: px(54.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(px(10.0), px(0.0)),
                    column_gap: px(8.0),
                    border: UiRect::all(px(1.0)),
                    ..default()
                },
                BorderRadius::all(px(19.0)),
                BackgroundColor(cream()),
                BorderColor(stroke()),
                shadow(),
            ))
            .with_children(|brand| {
                brand
                    .spawn((
                        Node {
                            width: px(34.0),
                            height: px(34.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(sage()),
                        BorderRadius::MAX,
                    ))
                    .with_child(icon(&icons.sprout, 28.0));
                brand.spawn(text("Waldwinkel", &title, 24.0, ink()));
            });

            for (name, target) in LANDMARKS {
                root.spawn((
                    Landmark(target),
                    Visibility::Hidden,
                    Node {
                        position_type: PositionType::Absolute,
                        width: px(106.),
                        height: px(24.),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BorderRadius::all(px(9.)),
                    BackgroundColor(Color::srgba(0.19, 0.33, 0.26, 0.87)),
                ))
                .with_child(text(name, &bold, 13., cream()));
            }

            root.spawn(Node {
                position_type: PositionType::Absolute,
                right: px(MARGIN),
                top: px(MARGIN),
                height: px(54.0),
                column_gap: px(8.0),
                ..default()
            })
            .with_children(|stats| {
                for (which_icon, label, value, field) in [
                    (&icons.day, "Frühling · Sonne", "Tag 1", Field::Day),
                    (&icons.coin, "Münzen", "20", Field::Coins),
                ] {
                    stats
                        .spawn((
                            Node {
                                min_width: px(116.0),
                                height: px(54.0),
                                padding: UiRect::axes(px(9.0), px(0.0)),
                                column_gap: px(5.0),
                                align_items: AlignItems::Center,
                                border: UiRect::all(px(1.0)),
                                ..default()
                            },
                            BackgroundColor(cream()),
                            BorderRadius::all(px(18.0)),
                            BorderColor(stroke()),
                            shadow(),
                        ))
                        .with_children(|counter| {
                            counter.spawn(icon(which_icon, 31.0));
                            counter
                                .spawn(Node {
                                    flex_direction: FlexDirection::Column,
                                    ..default()
                                })
                                .with_children(|row| {
                                    let mut heading =
                                        row.spawn(text(label, &regular, 12.0, muted()));
                                    if matches!(field, Field::Day) {
                                        heading.insert(Field::Weather);
                                    }
                                    row.spawn((field, text(value, &bold, 20.0, ink())));
                                });
                        });
                }
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(MARGIN),
                    bottom: px(MARGIN),
                    width: px(STICK_SIZE),
                    height: px(STICK_SIZE),
                    border: UiRect::all(px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.97, 0.94, 0.80, 0.72)),
                BorderRadius::MAX,
                BorderColor(stroke()),
                shadow(),
            ))
            .with_children(|stick| {
                stick.spawn((
                    ImageNode::new(icons.stick.clone()),
                    Node {
                        position_type: PositionType::Absolute,
                        width: px(104.0),
                        height: px(104.0),
                        left: px(0.0),
                        top: px(0.0),
                        ..default()
                    },
                ));
                stick.spawn((
                    Thumb,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(30.0),
                        top: px(30.0),
                        width: px(44.0),
                        height: px(44.0),
                        border: UiRect::all(px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb_u8(101, 128, 86)),
                    BorderRadius::MAX,
                    BorderColor(Color::srgb_u8(183, 193, 146)),
                    BoxShadow::new(
                        Color::srgba(0.10, 0.21, 0.16, 0.21),
                        px(0.0),
                        px(2.0),
                        px(0.0),
                        px(5.0),
                    ),
                ));
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    bottom: px(BUTTON_BOTTOM),
                    margin: UiRect {
                        left: px(-CONTEXT_WIDTH / 2.0),
                        ..default()
                    },
                    width: px(CONTEXT_WIDTH),
                    min_height: px(54.0),
                    padding: UiRect::axes(px(16.0), px(7.0)),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.126, 0.285, 0.239, 0.93)),
                BorderRadius::all(px(18.0)),
                BorderColor(Color::srgba(0.92, 0.94, 0.81, 0.26)),
                shadow(),
            ))
            .with_children(|hint| {
                hint.spawn((Field::Context, text("Dein Garten", &bold, 16.0, cream())));
                hint.spawn((
                    Field::Detail,
                    text("Tippe auf einen Platz im Garten.", &regular, 13.0, sage()),
                ));
                hint.spawn((
                    ProgressTrack,
                    Node {
                        display: Display::None,
                        margin: UiRect {
                            top: px(4.0),
                            ..default()
                        },
                        width: Val::Percent(78.0),
                        height: px(4.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.88, 0.92, 0.74, 0.26)),
                    BorderRadius::all(px(3.0)),
                ))
                .with_child((
                    Progress,
                    Node {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb_u8(229, 207, 136)),
                    BorderRadius::all(px(3.0)),
                ));
            });

            for (row, labels) in [
                vec![(Utility::Out, "−"), (Utility::In, "+")],
                vec![(Utility::Map, "Weltkarte")],
                vec![(Utility::Choice, "Saatgut")],
                vec![(Utility::Task, "Aufgaben")],
                vec![(Utility::Inventory, "Tasche")],
                vec![(Utility::Menu, "Menü")],
            ]
            .into_iter()
            .enumerate()
            {
                root.spawn(Node {
                    position_type: PositionType::Absolute,
                    left: px(MARGIN),
                    top: px(TOOLS_TOP + row as f32 * TOOL_STEP),
                    width: px(TOOL_WIDTH),
                    height: px(TOOL_HEIGHT),
                    column_gap: px(4.),
                    ..default()
                })
                .with_children(|group| {
                    for (utility, label) in labels {
                        group
                            .spawn((
                                Node {
                                    flex_grow: 1.,
                                    flex_basis: px(0.),
                                    height: px(TOOL_HEIGHT),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BorderRadius::all(px(14.)),
                                BackgroundColor(Color::srgba(0.98, 0.94, 0.82, 0.9)),
                            ))
                            .with_child((UtilityLabel(utility), text(label, &bold, 14., ink())));
                    }
                });
            }
            root.spawn((
                FertilizerButton,
                Node {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    right: px(MARGIN),
                    bottom: px(FERTILIZER_BOTTOM),
                    width: px(FERTILIZER_WIDTH),
                    height: px(FERTILIZER_HEIGHT),
                    column_gap: px(5.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BorderRadius::all(px(15.)),
                BackgroundColor(cream()),
                shadow(),
            ))
            .with_children(|button| {
                button.spawn(icon(&icons.fertilizer, 26.));
                button.spawn(text("Düngen", &bold, 15., ink()));
            });
            root.spawn((
                ActionSurface,
                Node {
                    position_type: PositionType::Absolute,
                    right: px(MARGIN),
                    bottom: px(BUTTON_BOTTOM),
                    width: px(BUTTON_WIDTH),
                    height: px(BUTTON_HEIGHT),
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(px(10.0), px(0.0)),
                    column_gap: px(7.0),
                    border: UiRect::all(px(1.0)),
                    ..default()
                },
                BorderRadius::all(px(20.0)),
                BackgroundColor(cream()),
                BorderColor(stroke()),
                shadow(),
            ))
            .with_children(|button| {
                button
                    .spawn((
                        Node {
                            width: px(39.0),
                            height: px(39.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(sage()),
                        BorderRadius::MAX,
                    ))
                    .with_child((ActionIcon, icon(&icons.choose, 34.0)));
                button.spawn((Field::Action, text("Beet wählen", &bold, 17.0, ink())));
            });
        });
}

pub fn sync_visibility(
    ready: Res<Ready>,
    session: Res<Session>,
    dialog: Res<Dialog>,
    farm_menu: Res<crate::farm_ui::FarmMenu>,
    keys: Res<ButtonInput<KeyCode>>,
    mut show: Local<Option<bool>>,
    mut hud: Single<&mut Visibility, (With<Hud>, Without<Loading>)>,
    mut loading: Single<&mut Visibility, (With<Loading>, Without<Hud>)>,
) {
    if keys.just_pressed(KeyCode::F10) {
        *show = Some(!show.unwrap_or(true));
    }
    **hud = if ready.0
        && session.playing()
        && !dialog.open()
        && !farm_menu.open()
        && show.unwrap_or(true)
    {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    **loading = if ready.0 || !session.playing() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
}

pub fn update_fertilizer(
    layout: Res<HudLayout>,
    garden: Res<Garden>,
    mut button: Single<(&mut Node, &mut BackgroundColor), With<FertilizerButton>>,
) {
    let (node, color) = &mut *button;
    node.display = if layout.fertilizer {
        Display::Flex
    } else {
        Display::None
    };
    color.0 = if garden.fertilizer == 0 {
        Color::srgba(0.97, 0.94, 0.82, 0.70)
    } else {
        cream()
    };
}

#[derive(PartialEq)]
pub(crate) struct Panel {
    action: Option<Action>,
    enabled: bool,
    pressed: bool,
    title: String,
    detail: String,
    fallback: &'static str,
    counters: (u32, u32),
    minute: u16,
    sky: Sky,
    season: Season,
}

pub fn update(
    garden: Res<Garden>,
    adventure: Res<Adventure>,
    focus: Res<GardenFocus>,
    player: Single<&Gardener>,
    gesture: Res<Gesture>,
    notice: Res<Notice>,
    icons: Res<Icons>,
    time: Res<Time>,
    mut fields: Query<(&Field, &mut Text)>,
    mut labels: Query<(&UtilityLabel, &mut Text), Without<Field>>,
    mut action_surface: Single<&mut BackgroundColor, With<ActionSurface>>,
    mut action_icon: Single<&mut ImageNode, With<ActionIcon>>,
    mut progress: Single<&mut Node, (With<Progress>, Without<ProgressTrack>, Without<Thumb>)>,
    mut progress_track: Single<&mut Node, (With<ProgressTrack>, Without<Progress>, Without<Thumb>)>,
    mut thumb: Single<&mut Node, (With<Thumb>, Without<ProgressTrack>, Without<Progress>)>,
    mut old_panel: Local<Option<Panel>>,
) {
    thumb.left = px(30.0 + gesture.stick_axis.x * 32.0);
    thumb.top = px(30.0 - gesture.stick_axis.y * 32.0);
    let now = time.elapsed_secs();
    let (title, detail, fallback, enabled) = adventure.context(&garden, player.position, now);
    let mut progress_value = None;
    let mut panel = Panel {
        action: None,
        enabled,
        pressed: gesture.action_held(),
        title,
        detail,
        fallback,
        counters: (garden.day, garden.farm.coins),
        minute: garden.clock.minute(),
        sky: Sky::at(garden.day),
        season: Season::at(garden.day),
    };
    if let Some(index) = focus.selected {
        let plot = garden.plots[index];
        let stage = plot.stage();
        panel.action = garden.action(index, adventure.crop);
        panel.enabled = panel.action.is_some();
        let label = match stage {
            Stage::Grass => "Wiesenplatz",
            Stage::Soil => "Lockere Erde",
            Stage::Sprout | Stage::Young => plot.crop().name(),
            Stage::Ripe => "Erntereif",
        };
        panel.title = format!("Beet {} · {label}", index + 1);
        panel.detail = match stage {
            Stage::Grass => "Lockere die Erde.".into(),
            Stage::Soil => format!(
                "{} · {} Saatgut",
                adventure.crop.name(),
                garden.seed_count(adventure.crop)
            ),
            Stage::Ripe => format!("{} sind reif.", plot.crop().name()),
            _ => "Die Pflanzen wachsen.".into(),
        };
        if let Some((left, wet)) = plot.remaining().filter(|(left, _)| *left > 0) {
            panel.detail = format!(
                "Noch {left} {} · {}",
                if left == 1 { "Tag" } else { "Tage" },
                if wet { "gegossen" } else { "braucht Wasser" }
            );
            progress_value = Some(1. - f32::from(left) / f32::from(plot.crop().days()));
        }
        if garden.fertilized[index] {
            panel.detail.push_str(" · gedüngt");
        } else if garden.fertilizable(index) && garden.fertilizer > 0 {
            panel
                .detail
                .push_str(&format!(" · Dünger {}", garden.fertilizer));
        }
        panel.fallback = if garden.clock.night() {
            panel.detail = "Zeit zum Schlafen. Im Bett beginnt der nächste Tag.".into();
            "Morgen weiter"
        } else if focus.pending.is_some()
            && !can_work(player.position, PLOTS[index], &adventure, &garden)
        {
            panel.detail = "Auf dem Weg zum Beet …".into();
            "Zum Beet"
        } else if stage == Stage::Soil && panel.action.is_none() {
            panel.enabled = true;
            "Saatgut wählen"
        } else {
            "Wächst …"
        };
    } else if garden.clock.night() && adventure.focus.is_none() && adventure.area.outdoors() {
        panel.detail = "Die Feldarbeit ruht bis morgen. Schlaf in deiner Hütte.".into();
    }
    if (notice.index.is_none() || notice.index == focus.selected) && now < notice.until {
        panel.detail.clone_from(&notice.message);
    }
    if adventure.inventory {
        panel.title = "Tasche".into();
        panel.detail = format!(
            "Holz {} · Stein {} · Kupfer {}\nFutter {} · Dünger {}\nMöhren {} · Rüben {} · Kürbisse {}\nErdbeeren {} · Tomaten {} · Sonnenblumen {}\nÄpfel {} · Beeren {} · Eier {} · Wolle {}\nFisch {} · Silber {} · Kristalle {}\nPilze {} · Haselnüsse {} · Muscheln {}",
            garden.wood,
            garden.stone,
            garden.copper,
            garden.farm.feed,
            garden.fertilizer,
            garden.crop_total(Crop::Carrot),
            garden.crop_total(Crop::Turnip),
            garden.crop_total(Crop::Pumpkin),
            garden.crop_total(Crop::Strawberry),
            garden.crop_total(Crop::Tomato),
            garden.crop_total(Crop::Sunflower),
            garden.stock(Product::Apple),
            garden.stock(Product::Berry),
            garden.stock(Product::Egg),
            garden.stock(Product::Wool),
            garden.fish,
            garden.stock(Product::Silver),
            garden.stock(Product::Crystal),
            garden.stock(Product::Mushroom),
            garden.stock(Product::Hazel),
            garden.stock(Product::Shell),
        );
    } else if adventure.journal {
        panel.title = "Stadtaufträge".into();
        panel.detail = town::journal_summary(&garden);
    }
    for (utility, mut label) in &mut labels {
        let next = match utility.0 {
            Utility::Choice if adventure.area == Area::Home => adventure.furniture_name(),
            Utility::Choice => {
                homestead::choice_label(adventure.area, adventure.focus, adventure.offer(&garden))
                    .unwrap_or("Saatgut")
            }
            Utility::Task if adventure.journal => "Schließen",
            Utility::Task
                if adventure.area.outdoors() && District::at(player.position).is_some() =>
            {
                "Stadtaufträge"
            }
            Utility::Task if garden.task().1 => "Belohnung",
            Utility::Task => "Aufgaben",
            Utility::Map if adventure.map => "Nahansicht",
            Utility::Map if adventure.area.outdoors() => "Weltkarte",
            Utility::Map => "Übersicht",
            Utility::Inventory if adventure.inventory => "Schließen",
            Utility::Inventory => "Tasche",
            _ => continue,
        };
        if label.0 != next {
            label.0 = next.into();
        }
    }
    if let Some(percent) = progress_value {
        progress_track.display = Display::Flex;
        progress.width = Val::Percent(percent * 100.0);
    } else {
        progress_track.display = Display::None;
    }
    if old_panel.as_ref() == Some(&panel) {
        return;
    }
    for (label, mut value) in &mut fields {
        value.0 = match label {
            Field::Day => format!("Tag {} · {}", panel.counters.0, garden.clock.display()),
            Field::Coins => panel.counters.1.to_string(),
            Field::Weather => format!(
                "{} · {}",
                if garden.clock.dusk() > 0. {
                    garden.clock.label()
                } else {
                    panel.season.label()
                },
                panel.sky.label()
            ),
            Field::Context => panel.title.clone(),
            Field::Detail => panel.detail.clone(),
            Field::Action => panel.action.map_or(panel.fallback, Action::label).into(),
        };
    }
    action_surface.0 = if panel.enabled && panel.pressed {
        Color::srgb_u8(229, 223, 185)
    } else if panel.enabled {
        cream()
    } else {
        Color::srgba(0.97, 0.94, 0.82, 0.79)
    };
    action_icon.image = match panel.action {
        Some(Action::Till) => icons.hoe.clone(),
        Some(Action::Sow) => icons.seeds.clone(),
        Some(Action::Water) => icons.watering.clone(),
        Some(Action::Fertilize) => icons.fertilizer.clone(),
        Some(Action::Harvest) => icons.carrot.clone(),
        None => icons.choose.clone(),
    };
    *old_panel = Some(panel);
}
