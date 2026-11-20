use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::{
    Adventure, Controls, Garden, GardenFocus, Notice, Ready,
    controller::Controller,
    farm_ui::FarmMenu,
    fieldwork::FieldTool,
    garden::Crop,
    inventory,
    night::Night,
    session::Session,
    town::Dialog,
    ui::{self, UiFonts},
    world::Gardener,
};

const TOOLS: [FieldTool; 8] = [
    FieldTool::Auto,
    FieldTool::Hoe,
    FieldTool::Seeds,
    FieldTool::Water,
    FieldTool::Fertilizer,
    FieldTool::Harvest,
    FieldTool::Pickaxe,
    FieldTool::Axe,
];
const WIDTH: f32 = 424.;
const HEIGHT: f32 = 418.;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Tools,
    Seeds,
}

#[derive(Resource, Default)]
pub struct Wheel {
    page: Option<Page>,
    index: usize,
}

impl Wheel {
    pub fn open(&self) -> bool {
        self.page.is_some()
    }

    pub fn choose_seeds(&mut self, crop: Crop) {
        self.page = Some(Page::Seeds);
        self.index = crop as usize;
    }
}

impl Page {
    fn len(self) -> usize {
        match self {
            Self::Tools => TOOLS.len(),
            Self::Seeds => Crop::ALL.len(),
        }
    }

    fn name(self, index: usize) -> &'static str {
        match self {
            Self::Tools => TOOLS[index].name(),
            Self::Seeds => Crop::ALL[index].name(),
        }
    }

    fn quantity(self, index: usize, garden: &Garden) -> Option<u32> {
        match self {
            Self::Seeds => Some(garden.seed_count(Crop::ALL[index])),
            Self::Tools if TOOLS[index] == FieldTool::Fertilizer => Some(garden.fertilizer),
            Self::Tools => None,
        }
    }

    fn hint(self, index: usize, garden: &Garden) -> &'static str {
        if self == Self::Seeds {
            return if self.quantity(index, garden) == Some(0) {
                "Saatgut bekommst du bei Lina in Eichenhafen."
            } else {
                "Wähle eine Sorte und säe sie in lockere Erde."
            };
        }
        match TOOLS[index] {
            FieldTool::Auto => "A wählt vor dir den passenden Handgriff.",
            FieldTool::Hoe => "Lockert Wiesenbeete für die Aussaat.",
            FieldTool::Seeds => "Öffnet die Auswahl deiner Saatgutsorten.",
            FieldTool::Water => "Gießt trockene Beete. Wasser hilft beim Wachsen.",
            FieldTool::Fertilizer if garden.fertilizer == 0 => {
                "Dünger bekommst du bei Lina am Saatgutstand."
            }
            FieldTool::Fertilizer => "Düngt ein besätes Beet für eine bessere Ernte.",
            FieldTool::Harvest => "Sammelt reife Pflanzen ein.",
            FieldTool::Axe => "Fällt markierte Bäume für Holz.",
            FieldTool::Pickaxe => "Baut markierte Felsen und Erzadern ab.",
        }
    }
}

fn sector(direction: Vec2, count: usize) -> usize {
    let angle = direction.x.atan2(-direction.y);
    ((angle / TAU * count as f32).round() as isize).rem_euclid(count as isize) as usize
}

#[derive(Component)]
pub struct WheelRoot;
#[derive(Component)]
pub struct Slot(usize);
#[derive(Component)]
pub struct Close;
#[derive(Component)]
pub struct Picture(usize);
#[derive(Component)]
pub enum Label {
    Title,
    Detail,
    Help,
    Slot(usize),
    Amount(usize),
    Back,
}

#[derive(Resource)]
pub struct Pictures {
    tools: [Handle<Image>; 8],
    seeds: [Handle<Image>; 6],
}

impl FromWorld for Pictures {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            tools: [
                "choose",
                "items/tool_hoe_0",
                "seeds",
                "items/tool_can_0",
                "items/fertilizer",
                "carrot",
                "items/tool_pickaxe_0",
                "items/tool_axe_0",
            ]
            .map(|name| assets.load(format!("ui/{name}.png"))),
            seeds: Crop::ALL
                .map(|crop| assets.load(format!("ui/items/{}.png", inventory::crop_icon(crop)))),
        }
    }
}

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>) {
    commands
        .spawn((
            WheelRoot,
            GlobalZIndex(120),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.10, 0.22, 0.18, 0.64)),
        ))
        .with_children(|veil| {
            veil.spawn((
                Node {
                    width: Val::Px(WIDTH),
                    height: Val::Px(HEIGHT),
                    ..default()
                },
                BorderRadius::all(Val::Px(42.)),
                BackgroundColor(ui::cream()),
                ui::shadow(),
            ))
            .with_children(|ring| {
                ring.spawn(Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(122.),
                    right: Val::Px(122.),
                    top: Val::Px(147.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(8.),
                    ..default()
                })
                .with_children(|center| {
                    center.spawn((
                        Label::Title,
                        fonts.title("", 25.),
                        TextLayout::new_with_justify(JustifyText::Center),
                    ));
                    center.spawn((
                        Label::Detail,
                        fonts.body("", 15.),
                        TextLayout::new_with_justify(JustifyText::Center),
                    ));
                });
                for index in 0..TOOLS.len() {
                    ring.spawn((
                        Slot(index),
                        Button,
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Px(100.),
                            height: Val::Px(67.),
                            border: UiRect::all(Val::Px(2.)),
                            padding: UiRect::all(Val::Px(3.)),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb_u8(231, 226, 193)),
                        BorderColor(Color::srgb_u8(188, 188, 146)),
                        BorderRadius::all(Val::Px(15.)),
                    ))
                    .with_children(|slot| {
                        slot.spawn((
                            Picture(index),
                            ImageNode::default(),
                            Node {
                                width: Val::Px(33.),
                                height: Val::Px(33.),
                                ..default()
                            },
                        ));
                        slot.spawn((Label::Slot(index), fonts.bold("", 12.)));
                        slot.spawn((
                            Label::Amount(index),
                            fonts.bold("", 11.),
                            Node {
                                position_type: PositionType::Absolute,
                                right: Val::Px(5.),
                                top: Val::Px(4.),
                                ..default()
                            },
                        ));
                    });
                }
                ring.spawn(Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(11.),
                    left: Val::Px(9.),
                    right: Val::Px(9.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|footer| {
                    footer.spawn((Label::Help, fonts.body("", 13.)));
                    footer
                        .spawn((Close, ui::dialog_button(81.)))
                        .with_child((Label::Back, fonts.button("Zurück", 13.)));
                });
            });
        });
}

pub fn input(
    session: Res<Session>,
    layers: (Res<Ready>, Res<FarmMenu>, Res<Dialog>, Res<Night>),
    controller: Res<Controller>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Query<
        (&Interaction, Option<&Slot>, Option<&Close>),
        (Changed<Interaction>, Or<(With<Slot>, With<Close>)>),
    >,
    mut wheel: ResMut<Wheel>,
    mut controls: ResMut<Controls>,
    mut focus: ResMut<GardenFocus>,
    mut adventure: ResMut<Adventure>,
    player: Single<&mut Gardener>,
    garden: Res<Garden>,
    time: Res<Time>,
    mut notice: ResMut<Notice>,
) {
    let (ready, menu, dialog, night) = layers;
    let mut player = player.into_inner();
    if !session.playing() || !ready.0 || menu.open() || dialog.open() || night.active() {
        wheel.page = None;
        return;
    }
    if !wheel.open() {
        if adventure.map || !(controls.wheel || controller.wheel) {
            return;
        }
        wheel.page = Some(Page::Tools);
        wheel.index = TOOLS
            .iter()
            .position(|&tool| tool == focus.tool)
            .unwrap_or(0);
        adventure.clear_approach();
        focus.pending = None;
        player.stop();
        *controls = Controls::default();
        return;
    }
    *controls = Controls::default();
    let page = wheel.page.unwrap();
    let pressed = buttons
        .iter()
        .find(|(state, _, _)| **state == Interaction::Pressed);
    if controller.wheel {
        wheel.page = None;
        return;
    }
    if controller.back
        || keys.just_pressed(KeyCode::Escape)
        || pressed.is_some_and(|(_, _, b)| b.is_some())
    {
        wheel.page = (page == Page::Seeds).then_some(Page::Tools);
        wheel.index = 2;
        return;
    }
    let direction = controller.radial;
    let step = controller.step;
    let mut offset = (step.x + step.y).signum() as isize;
    if keys.any_just_pressed([KeyCode::ArrowLeft, KeyCode::ArrowUp]) {
        offset = -1;
    } else if keys.any_just_pressed([KeyCode::ArrowRight, KeyCode::ArrowDown]) {
        offset = 1;
    }
    let mut index = wheel.index;
    if offset != 0 {
        index = (index as isize + offset).rem_euclid(page.len() as isize) as usize;
    } else if direction.length_squared() >= 0.26 {
        index = sector(direction, page.len());
    }
    if let Some((_, Some(slot), _)) = pressed {
        index = slot.0;
    }
    if index >= page.len() {
        return;
    }
    if wheel.index != index {
        wheel.index = index;
    }
    if !(controller.accept
        || keys.any_just_pressed([KeyCode::Enter, KeyCode::Space])
        || pressed.is_some())
    {
        return;
    }
    if page.quantity(index, &garden) == Some(0) {
        return;
    }
    let tool = match page {
        Page::Tools if TOOLS[index] == FieldTool::Seeds => {
            wheel.choose_seeds(adventure.crop);
            return;
        }
        Page::Tools => TOOLS[index],
        Page::Seeds => {
            adventure.crop = Crop::ALL[index];
            FieldTool::Seeds
        }
    };
    focus.tool = tool;
    if matches!(tool, FieldTool::Axe | FieldTool::Pickaxe) {
        focus.selected = None;
    }
    focus.pad_lock = false;
    adventure.focus = None;
    wheel.page = None;
    *notice = Notice {
        message: if tool == FieldTool::Seeds {
            format!("{} · Saatgut ausgewählt", adventure.crop.name())
        } else {
            format!("{} ausgewählt", tool.name())
        },
        until: time.elapsed_secs() + Notice::DURATION,
        ..default()
    };
}

pub fn update(
    wheel: Res<Wheel>,
    controller: Res<Controller>,
    pictures: Res<Pictures>,
    garden: Res<Garden>,
    mut root: Single<&mut Node, (With<WheelRoot>, Without<Slot>)>,
    mut slots: Query<(&Slot, &mut Node, &mut BackgroundColor, &mut BorderColor)>,
    mut labels: Query<(&Label, &mut Text)>,
    mut icons: Query<(&Picture, &mut ImageNode)>,
) {
    let Some(page) = wheel.page else {
        root.display = Display::None;
        return;
    };
    root.display = Display::Flex;
    for (Slot(index), mut node, mut surface, mut edge) in &mut slots {
        let i = *index;
        if i >= page.len() {
            node.display = Display::None;
            continue;
        }
        let angle = i as f32 * TAU / page.len() as f32;
        node.display = Display::Flex;
        node.left = Val::Px(WIDTH / 2. + angle.sin() * 152. - 50.);
        node.top = Val::Px(185. - angle.cos() * 132. - 33.);
        let selected = wheel.index == i;
        surface.0 = if page.quantity(i, &garden) == Some(0) {
            Color::srgb_u8(219, 215, 199)
        } else if selected {
            Color::srgb_u8(246, 220, 158)
        } else {
            Color::srgb_u8(231, 226, 193)
        };
        edge.0 = if selected {
            Color::srgb_u8(153, 89, 42)
        } else {
            Color::srgb_u8(188, 188, 146)
        };
    }
    let assets = match page {
        Page::Tools => &pictures.tools[..],
        Page::Seeds => &pictures.seeds[..],
    };
    for (Picture(i), mut image) in &mut icons {
        if let Some(asset) = assets.get(*i) {
            if image.image != *asset {
                image.image = asset.clone();
            }
        }
    }
    for (label, mut text) in &mut labels {
        let value = match label {
            Label::Title => page.name(wheel.index).into(),
            Label::Detail => page.hint(wheel.index, &garden).into(),
            Label::Help if controller.active => {
                "Stick / Steuerkreuz · wählen   A · bestätigen".into()
            }
            Label::Help => "Tippe auf ein Werkzeug oder Saatgut.".into(),
            Label::Back if controller.active => "B Zurück".into(),
            Label::Back => "Zurück".into(),
            Label::Slot(i) if *i < page.len() => page.name(*i).into(),
            Label::Amount(i) if *i < page.len() => page
                .quantity(*i, &garden)
                .map_or_else(String::new, |n| format!("×{n}")),
            _ => String::new(),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stick_points_at_the_matching_wheel_sector() {
        assert_eq!(sector(Vec2::NEG_Y, 8), 0);
        assert_eq!(sector(Vec2::new(1., -1.), 8), 1);
        assert_eq!(sector(Vec2::X, 8), 2);
        assert_eq!(sector(Vec2::NEG_X, 8), 6);
    }
}
