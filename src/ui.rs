use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    GardenFocus, Notice, Ready,
    adventure::{
        Adventure, Area, LANDMARKS, Target, landmark_center, landmark_offset, tracked_pin,
    },
    garden::{Action, Garden, Stage},
    homestead,
    input::Gesture,
    session::Session,
    town::Dialog,
    travel::District,
    weather::{Season, Sky},
    world::{Gardener, IslandCamera, PLOTS, work_reach},
};

const MARGIN: f32 = 23.0;
const STICK_SIZE: f32 = 108.0;
const BUTTON_WIDTH: f32 = 188.0;
const BUTTON_HEIGHT: f32 = 62.0;
const BUTTON_BOTTOM: f32 = 29.0;
const FERTILIZER_BOTTOM: f32 = BUTTON_BOTTOM + BUTTON_HEIGHT + 10.0;
const FERTILIZER_HEIGHT: f32 = 52.0;
const FERTILIZER_WIDTH: f32 = 119.0;
const CONTEXT_WIDTH: f32 = 316.0;
const TOOLS_TOP: f32 = 88.;
const TOOL_WIDTH: f32 = 113.;
const TOOL_HEIGHT: f32 = 58.;
const TOOL_GAP: f32 = 7.;
const TOOL_STEP: f32 = TOOL_HEIGHT + TOOL_GAP;
const FIELD_TOOL_TOP: f32 = TOOLS_TOP + 2. * TOOL_STEP;
const FIELD_TOOL_WIDTH: f32 = 162.;
const ZOOM_WIDTH: f32 = 56.;
const MENU_WIDTH: f32 = 94.;

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
    FieldTool,
}
impl Utility {
    const TOOLBAR: [[(Self, &'static str); 2]; 2] = [
        [(Self::Map, "Karte"), (Self::Task, "Journal")],
        [(Self::Choice, "Saatgut"), (Self::Inventory, "Tasche")],
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
            Self::FieldTool => controls.wheel = true,
        }
    }
}

#[derive(Component)]
pub struct UtilityLabel(Utility);

pub(crate) fn cream() -> Color {
    Color::srgb_u8(252, 245, 225)
}
fn ink() -> Color {
    Color::srgb_u8(34, 59, 48)
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
    origin: Vec2,
    pub scale: f32,
    pub(crate) bubble: Option<crate::context_menu::HitArea>,
    context_height: f32,
    fertilizer: bool,
    field_tools: bool,
    large: bool,
}

impl Default for HudLayout {
    fn default() -> Self {
        Self {
            size: Vec2::new(1280.0, 720.0),
            origin: Vec2::ZERO,
            scale: 1.0,
            bubble: None,
            context_height: 65.,
            fertilizer: false,
            field_tools: false,
            large: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct SafeInsets(pub [f32; 4]);

impl HudLayout {
    pub(crate) fn hud_position(self, viewport: Vec2) -> Vec2 {
        (viewport - self.origin) / self.scale
    }

    pub(crate) fn on_screen(self, point: Vec2) -> bool {
        (point - self.origin).cmpge(Vec2::ZERO).all()
            && (point - self.origin).cmple(self.size).all()
    }

    pub(crate) fn on_map_badge(self, center: Vec2, half: Vec2) -> bool {
        self.on_screen(center - half - Vec2::splat(6.))
            && self.on_screen(center + half + Vec2::splat(6.))
            && [center, center - half, center + half]
                .into_iter()
                .all(|at| !self.on_other_ui(at) && !self.on_action(at) && !self.on_stick(at))
    }

    pub(crate) fn safe_rect(self) -> Rect {
        Rect::from_corners(self.origin, self.origin + self.size)
    }

    pub fn stick_center(self) -> Vec2 {
        self.origin
            + Vec2::new(
                (MARGIN + STICK_SIZE * 0.5) * self.scale,
                self.size.y - (MARGIN + STICK_SIZE * 0.5) * self.scale,
            )
    }

    pub fn on_stick(self, at: Vec2) -> bool {
        at.distance_squared(self.stick_center()) <= ((STICK_SIZE * 0.5 + 5.0) * self.scale).powi(2)
    }

    pub fn on_action(self, at: Vec2) -> bool {
        let at = at - self.origin;
        let right = (self.size.x - at.x) / self.scale;
        let bottom = (self.size.y - at.y) / self.scale;
        (MARGIN..=MARGIN + BUTTON_WIDTH).contains(&right)
            && (BUTTON_BOTTOM..=BUTTON_BOTTOM + BUTTON_HEIGHT).contains(&bottom)
    }

    pub fn utility(self, at: Vec2) -> Option<Utility> {
        let at = at - self.origin;
        let right = (self.size.x - at.x) / self.scale;
        let bottom = (self.size.y - at.y) / self.scale;
        let top = at.y / self.scale;
        if (MARGIN..=MARGIN + MENU_WIDTH).contains(&right)
            && (MARGIN..=MARGIN + TOOL_HEIGHT).contains(&top)
        {
            return Some(Utility::Menu);
        }
        if self.fertilizer
            && (MARGIN..=MARGIN + FERTILIZER_WIDTH).contains(&right)
            && (FERTILIZER_BOTTOM..=FERTILIZER_BOTTOM + FERTILIZER_HEIGHT).contains(&bottom)
        {
            return Some(Utility::Fertilize);
        }
        if (self.zoom_bottom()..=self.zoom_bottom() + TOOL_HEIGHT).contains(&bottom) {
            if (MARGIN..=MARGIN + ZOOM_WIDTH).contains(&right) {
                return Some(Utility::In);
            }
            if (MARGIN + ZOOM_WIDTH + TOOL_GAP..=MARGIN + ZOOM_WIDTH * 2. + TOOL_GAP)
                .contains(&right)
            {
                return Some(Utility::Out);
            }
        }
        let touch = at / self.scale;
        if self.field_tools
            && (MARGIN..=MARGIN + FIELD_TOOL_WIDTH).contains(&touch.x)
            && (FIELD_TOOL_TOP..=FIELD_TOOL_TOP + TOOL_HEIGHT).contains(&touch.y)
        {
            return Some(Utility::FieldTool);
        }
        let point = touch - Vec2::new(MARGIN, TOOLS_TOP);
        if point.x < 0. || point.y < 0. {
            return None;
        }
        let row = (point.y / TOOL_STEP) as usize;
        if point.y % TOOL_STEP > TOOL_HEIGHT {
            return None;
        }
        let col_width = TOOL_WIDTH + TOOL_GAP;
        let col = (point.x / col_width) as usize;
        if point.x % col_width > TOOL_WIDTH {
            return None;
        }
        Utility::TOOLBAR
            .get(row)?
            .get(col)
            .map(|&(command, _)| command)
    }

    fn zoom_bottom(self) -> f32 {
        if self.fertilizer {
            FERTILIZER_BOTTOM + FERTILIZER_HEIGHT + 9.
        } else {
            FERTILIZER_BOTTOM
        }
    }

    pub fn on_other_ui(self, at: Vec2) -> bool {
        if self.utility(at).is_some() {
            return true;
        }
        let at = at - self.origin;
        let top = at.y / self.scale;
        let left = at.x / self.scale;
        let right = (self.size.x - at.x) / self.scale;
        let bottom = (self.size.y - at.y) / self.scale;
        ((MARGIN..=MARGIN + 55.0).contains(&top)
            && ((MARGIN..=MARGIN + 188.0).contains(&left)
                || (MARGIN..=MARGIN + MENU_WIDTH + 298.0).contains(&right)))
            || ((BUTTON_BOTTOM..=BUTTON_BOTTOM + self.context_height).contains(&bottom)
                && (at.x - self.size.x * 0.5).abs() < CONTEXT_WIDTH * self.scale * 0.5)
    }
}

pub fn resize(
    focus: Res<GardenFocus>,
    garden: Res<Garden>,
    adventure: Res<Adventure>,
    window: Single<&Window, With<PrimaryWindow>>,
    insets: Res<SafeInsets>,
    mut layout: ResMut<HudLayout>,
    mut scale: ResMut<UiScale>,
    mut hud: Single<&mut Node, With<Hud>>,
    player: Single<&Gardener>,
    controller: Res<crate::controller::Controller>,
    mut field_button: Single<&mut Visibility, With<FieldToolButton>>,
) {
    let [left, top, right, bottom] = insets.0.map(|v| v / window.scale_factor());
    let origin = Vec2::new(left, top);
    let size = Vec2::new(
        window.width() - left - right,
        window.height() - top - bottom,
    )
    .max(Vec2::ONE);
    let context_height = 65.;
    let fertilizer = !adventure.map
        && focus.selected.is_some_and(|i| garden.fertilizable(i))
        && !garden.clock.night();
    let field_tools = !adventure.map
        && (controller.active || adventure.area == Area::Island && player.position.length() < 17.);
    field_button.set_if_neq(if field_tools {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    if size != layout.size
        || origin != layout.origin
        || layout.context_height != context_height
        || layout.fertilizer != fertilizer
        || layout.field_tools != field_tools
        || layout.large != garden.large_ui
    {
        let preference = if garden.large_ui { 1.16 } else { 1.0 };
        let fit = ((size.y / 680.0).max(0.83) * preference)
            .min(size.y / 505.0)
            .min(size.x / 728.0)
            .clamp(0.42, 1.48);
        *layout = HudLayout {
            size,
            origin,
            scale: fit,
            bubble: None,
            context_height,
            fertilizer,
            field_tools,
            large: garden.large_ui,
        };
        scale.0 = fit;
        hud.position_type = PositionType::Absolute;
        hud.left = px(origin.x / fit);
        hud.top = px(origin.y / fit);
        hud.width = px(size.x / fit);
        hud.height = px(size.y / fit);
    }
}

#[derive(Component)]
pub struct Landmark(Target);

#[derive(Component)]
pub struct TrackedPin;

pub fn map_labels(
    adventure: Res<Adventure>,
    garden: Res<Garden>,
    layout: Res<HudLayout>,
    camera: Single<(&Camera, &Transform), With<IslandCamera>>,
    mut labels: Query<(&Landmark, &mut Node, &mut Visibility, &mut BackgroundColor)>,
    mut pin: Single<(&mut Node, &mut Visibility), (With<TrackedPin>, Without<Landmark>)>,
) {
    let (camera, pose) = *camera;
    let view = GlobalTransform::from(*pose);
    for (label, mut node, mut visible, mut background) in &mut labels {
        let position = if adventure.map && adventure.area.outdoors() {
            landmark_center(
                camera,
                &view,
                adventure.position(label.0, &garden),
                layout.scale,
                landmark_offset(label.0),
            )
        } else {
            None
        };
        if let Some(p) = position {
            let p = p - layout.origin;
            *visible = Visibility::Inherited;
            node.left = px(p.x / layout.scale - 53.);
            node.top = px(p.y / layout.scale - 12.);
            let active = adventure.map_focus == Some(crate::adventure::map_destination(label.0, &garden));
            background.set_if_neq(BackgroundColor(if active {
                Color::srgb_u8(145, 95, 37)
            } else {
                Color::srgba(0.19, 0.33, 0.26, 0.87)
            }));
        } else {
            *visible = Visibility::Hidden;
        }
    }
    let (node, visible) = &mut *pin;
    let point = tracked_pin(&adventure, &garden, camera, &view, &layout)
        .filter(|(target, _)| {
            !matches!(target, Target::Person(npc)
                if adventure.map || npc.quest_mark(&garden).is_some())
        })
        .map(|(_, point)| point);
    if let Some(p) = point {
        let p = p - layout.origin;
        **visible = Visibility::Inherited;
        node.left = px(p.x / layout.scale - 53.);
        node.top = px(p.y / layout.scale - 14.);
    } else {
        **visible = Visibility::Hidden;
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
pub struct FieldToolButton;

#[derive(Component)]
pub struct FertilizerButton;

#[derive(Component)]
pub struct ZoomTools;

#[derive(Component)]
pub struct ProgressTrack;

#[derive(Component)]
pub struct Progress;

#[derive(Component)]
pub struct ActionIcon;

#[derive(Component)]
pub enum Field {
    Place,
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
    map: Handle<Image>,
    journal: Handle<Image>,
    bag: Handle<Image>,
    menu: Handle<Image>,
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
            map: assets.load("ui/map.png"),
            journal: assets.load("ui/journal.png"),
            bag: assets.load("ui/bag.png"),
            menu: assets.load("ui/menu.png"),
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
        Interaction::Pressed => Color::srgb_u8(34, 59, 48),
        Interaction::Hovered => Color::srgb_u8(64, 94, 70),
        Interaction::None => Color::srgb_u8(43, 73, 57),
    }
}

pub fn dialog_button(width: f32) -> (Button, Node, BorderRadius, BackgroundColor, crate::controller::FocusEnabled) {
    (
        Button,
        Node {
            width: px(width),
            min_height: px(58.),
            padding: UiRect::axes(px(8.), px(4.)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BorderRadius::all(px(12.)),
        BackgroundColor(button_color(Interaction::None, true)),
        crate::controller::FocusEnabled(true),
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
                BackgroundColor(Color::srgb_u8(43, 73, 57)),
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
                brand.spawn((Field::Place, text("Waldwinkel", &title, 23.0, cream())));
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
            root.spawn((
                TrackedPin,
                Visibility::Hidden,
                Node {
                    position_type: PositionType::Absolute,
                    width: px(106.),
                    height: px(27.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BorderRadius::all(px(9.)),
                BackgroundColor(Color::srgb_u8(230, 181, 98)),
                GlobalZIndex(3),
            ))
            .with_child(text("Markiertes Ziel", &bold, 13., ink()));

            crate::resident_view::spawn(root, &fonts);

            root.spawn(Node {
                position_type: PositionType::Absolute,
                right: px(MARGIN + MENU_WIDTH + 8.),
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
                    align_items: AlignItems::FlexStart,
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

            for (row, labels) in Utility::TOOLBAR.into_iter().enumerate() {
                root.spawn(Node {
                    position_type: PositionType::Absolute,
                    left: px(MARGIN),
                    top: px(TOOLS_TOP + row as f32 * TOOL_STEP),
                    width: px(TOOL_WIDTH * 2. + TOOL_GAP),
                    height: px(TOOL_HEIGHT),
                    column_gap: px(TOOL_GAP),
                    ..default()
                })
                .with_children(|group| {
                    for (utility, label) in labels {
                        let image = match utility {
                            Utility::Map => &icons.map,
                            Utility::Task => &icons.journal,
                            Utility::Inventory => &icons.bag,
                            _ => &icons.seeds,
                        };
                        group
                            .spawn((
                                Node {
                                    width: px(TOOL_WIDTH),
                                    height: px(TOOL_HEIGHT),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    column_gap: px(3.),
                                    border: UiRect::all(px(1.)),
                                    ..default()
                                },
                                BorderRadius::all(px(15.)),
                                BackgroundColor(cream()),
                                BorderColor(Color::srgb_u8(218, 202, 165)),
                                shadow(),
                            ))
                            .with_children(|tool| {
                                tool.spawn(icon(image, 24.));
                                tool.spawn((UtilityLabel(utility), text(label, &bold, 15., ink())));
                            });
                    }
                });
            }
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: px(MARGIN),
                    top: px(MARGIN),
                    width: px(MENU_WIDTH),
                    height: px(TOOL_HEIGHT),
                    column_gap: px(3.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(cream()),
                BorderRadius::all(px(15.)),
                shadow(),
            ))
            .with_children(|menu| {
                menu.spawn(icon(&icons.menu, 25.));
                menu.spawn(text("Menü", &bold, 15., ink()));
            });
            root.spawn((
                FieldToolButton,
                Visibility::Hidden,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(MARGIN),
                    top: px(FIELD_TOOL_TOP),
                    width: px(FIELD_TOOL_WIDTH),
                    height: px(TOOL_HEIGHT),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: px(5.),
                    ..default()
                },
                BorderRadius::all(px(15.)),
                BackgroundColor(cream()),
                shadow(),
            ))
            .with_children(|button| {
                button.spawn(icon(&icons.hoe, 24.));
                button.spawn((
                    UtilityLabel(Utility::FieldTool),
                    text("Werkzeug", &bold, 15., ink()),
                ));
            });
            root.spawn((
                ZoomTools,
                Node {
                    position_type: PositionType::Absolute,
                    right: px(MARGIN),
                    bottom: px(FERTILIZER_BOTTOM),
                    column_gap: px(TOOL_GAP),
                    ..default()
                },
            ))
            .with_children(|zoom| {
                for label in ["−", "+"] {
                    zoom.spawn((
                        Node {
                            width: px(ZOOM_WIDTH),
                            height: px(TOOL_HEIGHT),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BorderRadius::all(px(15.)),
                        BackgroundColor(cream()),
                        shadow(),
                    ))
                    .with_child(text(label, &bold, 23., ink()));
                }
            });
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
    overlays: (
        Res<crate::farm_ui::FarmMenu>,
        Res<crate::wheel::Wheel>,
        Res<crate::decorate::Layout>,
        Res<crate::night::Night>,
    ),
    keys: Res<ButtonInput<KeyCode>>,
    mut show: Local<Option<bool>>,
    mut hud: Single<&mut Visibility, (With<Hud>, Without<Loading>)>,
    mut loading: Single<&mut Visibility, (With<Loading>, Without<Hud>)>,
) {
    if keys.just_pressed(KeyCode::F10) {
        *show = Some(!show.unwrap_or(true));
    }
    let (farm_menu, wheel, layout, night) = overlays;
    **hud = if ready.0
        && session.playing()
        && !dialog.open()
        && !farm_menu.open()
        && !wheel.open()
        && !layout.open()
        && !night.active()
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
    mut button: Single<
        (&mut Node, &mut BackgroundColor),
        (With<FertilizerButton>, Without<ZoomTools>),
    >,
    mut zoom: Single<&mut Node, With<ZoomTools>>,
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
    zoom.bottom = px(layout.zoom_bottom());
}

#[derive(Resource, Default)]
pub(crate) struct HudPanel(pub Option<Panel>);

#[derive(PartialEq)]
pub(crate) struct Panel {
    pub(crate) action: Option<Action>,
    pub(crate) beds: usize,
    pub(crate) pad: bool,
    pub(crate) enabled: bool,
    pressed: bool,
    pub(crate) title: String,
    place: &'static str,
    detail: String,
    pub(crate) fallback: &'static str,
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
    input: (Res<Gesture>, Res<crate::controller::Controller>),
    notice: Res<Notice>,
    icons: Res<Icons>,
    time: Res<Time>,
    mut fields: Query<(&Field, &mut Text)>,
    mut labels: Query<(&UtilityLabel, &mut Text), Without<Field>>,
    mut action_surface: Single<(&mut BackgroundColor, &mut Visibility), With<ActionSurface>>,
    mut action_icon: Single<&mut ImageNode, With<ActionIcon>>,
    mut progress: Single<&mut Node, (With<Progress>, Without<ProgressTrack>, Without<Thumb>)>,
    mut progress_track: Single<&mut Node, (With<ProgressTrack>, Without<Progress>, Without<Thumb>)>,
    mut thumb: Single<&mut Node, (With<Thumb>, Without<ProgressTrack>, Without<Progress>)>,
    mut old_panel: ResMut<HudPanel>,
) {
    let (gesture, controller) = input;
    thumb.left = px(30.0 + gesture.stick_axis.x * 32.0);
    thumb.top = px(30.0 - gesture.stick_axis.y * 32.0);
    let now = time.elapsed_secs();
    let (title, detail, fallback, enabled) = adventure.context(&garden, player.position, now);
    let mut progress_value = None;
    let mut panel = Panel {
        action: None,
        beds: 1,
        pad: controller.active,
        enabled,
        pressed: gesture.action_held(),
        title,
        place: if adventure.area.outdoors() {
            District::at(player.position).map_or("Waldwinkel", District::name)
        } else {
            adventure.area.name()
        },
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
        let tool = focus.tool.for_bed(&garden, index, adventure.crop);
        panel.action = tool.action(&garden, index, adventure.crop);
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
                "{} · {} Saatgut · {} Gießtage",
                adventure.crop.name(),
                garden.seed_count(adventure.crop),
                adventure.crop.days()
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
        if matches!(stage, Stage::Sprout | Stage::Young | Stage::Ripe) {
            let nearby = garden.companion(index);
            if nearby.shade {
                panel.detail.push_str(" · Kürbisschatten hält Wasser");
            } else if nearby.flowers && matches!(plot.crop(), crate::garden::Crop::Strawberry | crate::garden::Crop::Tomato) {
                panel.detail.push_str(" · Bestäubt");
            }
        }
        if tool != crate::fieldwork::FieldTool::Auto
            && let Some(action) = panel.action
        {
            let stroke =
                crate::fieldwork::stroke(&garden, index, adventure.crop, action, tool);
            panel.beds = stroke.targets().len();
            if panel.beds > 1 {
                panel.detail = format!("{} markierte Beete · {}", panel.beds, focus.tool.name());
            }
            if let Some(next) = stroke.next {
                panel
                    .detail
                    .push_str(&format!(" · Danach Beet {}", next + 1));
            }
        }
        panel.fallback = if garden.clock.night() {
            panel.action = None;
            panel.enabled = false;
            panel.detail = "Zeit zum Schlafen. Im Bett beginnt der nächste Tag.".into();
            "Morgen weiter"
        } else if focus.pending.is_some_and(|(action, _)| {
            !work_reach(PLOTS[index], action, &garden)
                .reached(player.position, &|p| adventure.walkable(p, &garden))
        }) {
            panel.detail = "Auf dem Weg zum Beet …".into();
            "Zum Beet"
        } else if stage == Stage::Soil
            && garden.seed_count(adventure.crop) == 0
            && matches!(
                tool,
                crate::fieldwork::FieldTool::Auto | crate::fieldwork::FieldTool::Seeds
            )
        {
            panel.enabled = true;
            "Saatgut wählen"
        } else if plot.remaining().is_some_and(|(left, wet)| left > 0 && wet) {
            "Heute gegossen"
        } else {
            "Wächst …"
        };
    } else if garden.clock.night() && adventure.focus.is_none() && adventure.area.outdoors() {
        panel.detail = "Die Feldarbeit ruht bis morgen. Schlaf in deiner Hütte.".into();
    }
    if focus.selected.is_none() && adventure.focus.is_none() && !garden.clock.night() {
        let step = crate::campaign::next(&garden);
        if adventure.area.outdoors()
            && !matches!(step, crate::campaign::Step::Done | crate::campaign::Step::Tutorial(_))
        {
            panel.title = step.title().into();
            panel.detail = step.hint(&garden, panel.pad);
        } else if panel.pad {
            panel.detail = "A · Interagieren   X · Werkzeugrad   ↑ · Journal".into();
        }
    }
    if adventure.bite_remaining(now).is_none() && notice.toast.is_none()
        && (notice.index.is_none() || notice.index == focus.selected) && now < notice.until
    {
        panel.detail.clone_from(&notice.message);
    }
    let (action_color, action_visible) = &mut *action_surface;
    let island_map = adventure.map && adventure.area.outdoors();
    action_visible.set_if_neq(if island_map {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    });
    if island_map {
        panel.title = adventure.tracked_target(&garden).map_or_else(
            || "! Neuer Auftrag · ? Abgabebereit".into(),
            |target| format!("Auftragsgebiet · {}", District::at(adventure.position(target, &garden)).map_or("Hof", District::name)),
        );
        panel.detail = if panel.pad {
            if let Some(target) = adventure.map_focus {
                panel.title = target.name().into();
                "← → Ziel wechseln · A Hinlaufen · B Zurück".into()
            } else {
                "← → Ziel wählen · B Zurück".into()
            }
        } else {
            "Tippe einen Namen an, um zur Figur zu laufen.".into()
        };
        progress_value = None;
    } else if player.finding_route() {
        panel.detail = if panel.pad {
            "Weg wird gesucht … · B bricht ab".into()
        } else {
            "Weg wird gesucht …".into()
        };
    }
    for (utility, mut label) in &mut labels {
        let next = match utility.0 {
            Utility::Choice if panel.pad && adventure.area == Area::Home => "LT Einrichten",
            Utility::Choice if panel.pad => "LT Saatgut",
            Utility::Task if panel.pad => "↑ Journal",
            Utility::Map if panel.pad && adventure.map => "Y Nahansicht",
            Utility::Map if panel.pad => "Y Karte",
            Utility::Inventory if panel.pad => "View Tasche",
            Utility::FieldTool if panel.pad => "X Werkzeuge",
            Utility::Choice if adventure.area == Area::Home => "Einrichten",
            Utility::Choice => {
                homestead::choice_label(adventure.area, adventure.focus, adventure.offer(&garden))
                    .unwrap_or("Saatgut")
            }
            Utility::Task => "Journal",
            Utility::Map if adventure.map => "Nahansicht",
            Utility::Map if adventure.area.outdoors() => "Karte",
            Utility::Map => "Übersicht",
            Utility::Inventory => "Tasche",
            Utility::FieldTool if focus.tool == crate::fieldwork::FieldTool::Auto => "Werkzeuge",
            Utility::FieldTool => focus.tool.name(),
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
    if old_panel.0.as_ref() == Some(&panel) {
        return;
    }
    for (label, mut value) in &mut fields {
        value.0 = match label {
            Field::Place => panel.place.into(),
            Field::Day => format!("Tag {} · {}", panel.counters.0, garden.clock.display()),
            Field::Coins => panel.counters.1.to_string(),
            Field::Weather => format!(
                "{} · {}",
                if garden.clock.dusk() > 0. {
                    garden.clock.label()
                } else {
                    panel.season.label()
                },
                if garden.clock.night() && panel.sky == Sky::Sun {
                    "Klar"
                } else {
                    panel.sky.label()
                }
            ),
            Field::Context => panel.title.clone(),
            Field::Detail => panel.detail.clone(),
            Field::Action => {
                let label = panel.action.map_or(panel.fallback, Action::label);
                let label = if panel.pad && label == "Ort wählen" { "Interagieren" } else { label };
                let prefix = if panel.pad && (panel.enabled || label == "Interagieren") {
                    "A · "
                } else {
                    ""
                };
                if panel.beds > 1 {
                    format!("{prefix}{label} · {}", panel.beds)
                } else {
                    format!("{prefix}{label}")
                }
            }
        };
    }
    action_color.0 = if panel.enabled && panel.pressed {
        Color::srgb_u8(217, 180, 110)
    } else if panel.enabled {
        Color::srgb_u8(244, 219, 160)
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
    old_panel.0 = Some(panel);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_toolbar_and_zoom_hit_boxes_fit_phone_and_tablet_layouts() {
        for (size, scale) in [
            (Vec2::new(1000., 560.), 0.83),
            (Vec2::new(1280., 800.), 1.18),
            (Vec2::new(1280., 800.), 1.36),
        ] {
            let layout = HudLayout {
                size,
                scale,
                fertilizer: true,
                ..default()
            };
            for (row, tools) in Utility::TOOLBAR.into_iter().enumerate() {
                for (column, (expected, _)) in tools.into_iter().enumerate() {
                    let center = Vec2::new(
                        MARGIN + TOOL_WIDTH / 2. + column as f32 * (TOOL_WIDTH + TOOL_GAP),
                        TOOLS_TOP + row as f32 * TOOL_STEP + TOOL_HEIGHT / 2.,
                    ) * scale;
                    assert!(layout.utility(center) == Some(expected));
                    assert!(!layout.on_stick(center) && !layout.on_action(center));
                }
            }
            let zoom = size
                - Vec2::new(
                    MARGIN + ZOOM_WIDTH / 2.,
                    layout.zoom_bottom() + TOOL_HEIGHT / 2.,
                ) * scale;
            assert!(layout.utility(zoom) == Some(Utility::In));
            assert!(!layout.on_action(zoom));
            let pause = Vec2::new(
                size.x - (MARGIN + MENU_WIDTH / 2.) * scale,
                (MARGIN + TOOL_HEIGHT / 2.) * scale,
            );
            assert!(layout.utility(pause) == Some(Utility::Menu));
        }
    }
}
