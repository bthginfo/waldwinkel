use bevy::prelude::*;

use crate::{
    Adventure, Controls, GardenFocus, Notice,
    adventure::{Area, commit},
    furniture::{self, Blueprint, CATALOG},
    garden::{Crop, FERTILIZER_PRICE, Garden, SavePath},
    homestead::{Product, Village},
    input::Gesture,
    processing::{Dish, Method, Preparation},
    regions::Bell,
    session::{Checkpoint, Session},
    ui::{self, UiFonts},
    world::Gardener,
};

const PAGE_SIZE: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FarmPage {
    Seeds,
    SeedShop,
    Sale,
    Workbench,
    BlueprintShop,
    Furnish,
    Kitchen,
    Travel(Bell),
    Inventory,
}

#[derive(Resource, Default)]
pub struct FarmMenu {
    page: Option<FarmPage>,
    list_page: usize,
    selected: Option<Entry>,
    preparation: Option<Preparation>,
    visible_time: f32,
    message: String,
}

impl FarmMenu {
    pub fn open(&self) -> bool {
        self.page.is_some()
    }

    pub fn show(&mut self, page: FarmPage) {
        *self = Self {
            page: Some(page),
            ..default()
        };
    }

    pub fn close(&mut self) {
        *self = Self::default();
    }

    fn entries(&self, page: FarmPage, garden: &Garden) -> Vec<Entry> {
        match page {
            FarmPage::Seeds | FarmPage::SeedShop => {
                Crop::ALL.into_iter().map(Entry::Seed).collect()
            }
            FarmPage::Sale | FarmPage::Inventory => {
                Product::stocked(garden).map(Entry::Stock).collect()
            }
            FarmPage::Travel(from) => Bell::ALL
                .into_iter()
                .filter(|&bell| bell != from && bell.ready(garden))
                .map(Entry::Destination)
                .collect(),
            FarmPage::Workbench | FarmPage::BlueprintShop | FarmPage::Furnish => CATALOG
                .iter()
                .enumerate()
                .filter(|&(index, design)| match page {
                    FarmPage::BlueprintShop => matches!(design.blueprint, Blueprint::Shop { .. }),
                    FarmPage::Furnish => garden.crafted[index],
                    _ => true,
                })
                .map(|(index, _)| Entry::Design(index))
                .collect(),
            FarmPage::Kitchen => self.preparation.as_ref().map_or_else(
                || Dish::ALL.into_iter().map(Entry::Dish).collect(),
                |game| {
                    game.buttons()
                        .iter()
                        .map(|&button| Entry::Step(button))
                        .collect()
                },
            ),
        }
    }

    fn last_page(&self, page: FarmPage, garden: &Garden) -> usize {
        self.entries(page, garden).len().saturating_sub(1) / PAGE_SIZE
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Entry {
    Seed(Crop),
    Stock(Product),
    Design(usize),
    Dish(Dish),
    Step(&'static str),
    Destination(Bell),
}

impl Entry {
    fn title(self) -> String {
        match self {
            Self::Seed(crop) => crop.name().into(),
            Self::Stock(item) => item.label(),
            Self::Design(i) => CATALOG[i].name.into(),
            Self::Dish(dish) => dish.name().into(),
            Self::Step(text) => text.into(),
            Self::Destination(bell) => bell.name().into(),
        }
    }

    fn detail(self, page: FarmPage, garden: &Garden) -> String {
        match self {
            Self::Seed(crop) if page == FarmPage::SeedShop => {
                format!(
                    "3 Samen · {} Münzen · {} im Vorrat",
                    crop.price(),
                    garden.seed_count(crop)
                )
            }
            Self::Seed(crop) => format!(
                "{} Samen · {} gegossene Tage",
                garden.seed_count(crop),
                crop.days()
            ),
            Self::Stock(item) => format!(
                "{} Stück · {} Münzen je Stück",
                garden.stock(item),
                item.value()
            ),
            Self::Design(i) if page == FarmPage::Furnish => {
                if garden.furniture.contains(&((i + 1) as u8)) {
                    "Aufgestellt"
                } else {
                    "Bereit zum Aufstellen"
                }
                .into()
            }
            Self::Design(i) if garden.crafted[i] => "Hergestellt".into(),
            Self::Design(i) if page == FarmPage::BlueprintShop && garden.blueprints[i] => {
                "Plan bereits bekannt".into()
            }
            Self::Design(i) if garden.blueprints[i] => CATALOG[i].supplies(garden),
            Self::Design(i) => CATALOG[i].source(garden),
            Self::Dish(dish) => format!("{} · {}", dish.method().title(), dish.supplies(garden)),
            Self::Step(_) => String::new(),
            Self::Destination(_) => "Wegglocke repariert".into(),
        }
    }

    fn enabled(self, page: FarmPage, garden: &Garden) -> bool {
        match self {
            Self::Seed(crop) if page == FarmPage::SeedShop => garden.farm.coins >= crop.price(),
            Self::Dish(dish) => dish.ready(garden),
            _ => true,
        }
    }
}

#[derive(Component)]
pub struct Pane;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Card(usize),
    Close,
    Back,
    Next,
    SellOne,
    SellAll,
    Fertilizer,
    Work,
}

#[derive(Component)]
pub enum Field {
    Title,
    Instruction,
    Message,
    CardName(usize),
    CardDetail(usize),
    Work,
}

#[derive(Component)]
pub enum HeatMeter {
    Track,
    Needle,
}

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>) {
    commands
        .spawn((
            Pane,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.19, 0.18, 0.47)),
            GlobalZIndex(86),
        ))
        .with_children(|veil| {
            veil.spawn((
                Node {
                    width: Val::Px(545.),
                    max_width: Val::Percent(95.),
                    padding: UiRect::all(Val::Px(22.)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(13.),
                    ..default()
                },
                BorderRadius::all(Val::Px(24.)),
                BackgroundColor(ui::cream()),
                ui::shadow(),
            ))
            .with_children(|paper| {
                paper
                    .spawn(Node {
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(12.),
                        ..default()
                    })
                    .with_children(|header| {
                        header.spawn((Field::Title, fonts.title("", 33.)));
                        header
                            .spawn((Choice::Close, ui::dialog_button(106.)))
                            .with_child(fonts.button("Schließen", 14.));
                    });
                paper.spawn((Field::Instruction, fonts.body("", 15.)));
                paper
                    .spawn((
                        HeatMeter::Track,
                        Node {
                            display: Display::None,
                            min_height: Val::Px(19.),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(8.)),
                        BackgroundColor(Color::srgb_u8(224, 205, 167)),
                    ))
                    .with_children(|track| {
                        let green = Preparation::HEAT_RANGE;
                        track.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Percent(green.start() * 100.),
                                width: Val::Percent((green.end() - green.start()) * 100.),
                                height: Val::Percent(100.),
                                ..default()
                            },
                            BackgroundColor(Color::srgb_u8(149, 179, 109)),
                        ));
                        track.spawn((
                            HeatMeter::Needle,
                            Node {
                                position_type: PositionType::Absolute,
                                width: Val::Px(5.),
                                height: Val::Percent(100.),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(3.)),
                            BackgroundColor(Color::srgb_u8(92, 65, 41)),
                        ));
                    });
                paper
                    .spawn(Node {
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(10.),
                        row_gap: Val::Px(8.),
                        ..default()
                    })
                    .with_children(|cards| {
                        for i in 0..PAGE_SIZE {
                            cards
                                .spawn((
                                    Choice::Card(i),
                                    Button,
                                    Node {
                                        width: Val::Percent(48.),
                                        min_height: Val::Px(59.),
                                        padding: UiRect::axes(Val::Px(12.), Val::Px(6.)),
                                        flex_direction: FlexDirection::Column,
                                        justify_content: JustifyContent::Center,
                                        row_gap: Val::Px(1.),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb_u8(241, 228, 191)),
                                    BorderRadius::all(Val::Px(12.)),
                                ))
                                .with_children(|card| {
                                    card.spawn((Field::CardName(i), fonts.bold("", 16.)));
                                    card.spawn((Field::CardDetail(i), fonts.body("", 13.)));
                                });
                        }
                    });
                paper.spawn((Field::Message, fonts.body("", 15.)));
                paper
                    .spawn(Node {
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(8.),
                        row_gap: Val::Px(8.),
                        ..default()
                    })
                    .with_children(|footer| {
                        for (action, caption, width) in [
                            (Choice::SellOne, "1 verkaufen", 114.),
                            (Choice::SellAll, "Stapel verkaufen", 149.),
                            (Choice::Back, "←", 53.),
                            (Choice::Next, "→", 53.),
                            (Choice::Fertilizer, "2 Dünger kaufen · 4 Münzen", 229.),
                            (Choice::Work, "Herstellen", 164.),
                        ] {
                            footer
                                .spawn((action, ui::dialog_button(width)))
                                .with_children(|button| {
                                    let mut label = button.spawn(fonts.button(caption, 14.));
                                    if action == Choice::Work {
                                        label.insert(Field::Work);
                                    }
                                });
                        }
                    });
            });
        });
}

pub fn input(
    session: Res<Session>,
    choices: Query<(&Choice, &Interaction), Changed<Interaction>>,
    mut menu: ResMut<FarmMenu>,
    mut adventure: ResMut<Adventure>,
    mut focus: ResMut<GardenFocus>,
    mut player: Single<(&mut Gardener, &mut Transform)>,
    mut garden: ResMut<Garden>,
    mut gesture: ResMut<Gesture>,
    mut controls: ResMut<Controls>,
    path: Res<SavePath>,
    mut notice: ResMut<Notice>,
    time: Res<Time>,
) {
    let Some(page) = menu.page.filter(|_| session.playing()) else {
        return;
    };
    let Some(choice) = choices
        .iter()
        .find_map(|(choice, pressed)| (*pressed == Interaction::Pressed).then_some(*choice))
    else {
        return;
    };
    let now = time.elapsed_secs();
    let result = match choice {
        Choice::Close => {
            menu.close();
            *gesture = Gesture::default();
            *controls = Controls::default();
            None
        }
        Choice::Card(index) => {
            let entry = menu
                .entries(page, &garden)
                .get(menu.list_page * PAGE_SIZE + index)
                .copied();
            match entry {
                Some(Entry::Seed(crop)) if page == FarmPage::Seeds => {
                    adventure.crop = crop;
                    menu.close();
                    *gesture = Gesture::default();
                    *controls = Controls::default();
                    None
                }
                Some(Entry::Seed(crop)) => Some(Village::Seeds.work(&mut garden, crop, None)),
                Some(Entry::Design(i)) if page == FarmPage::Furnish => {
                    adventure.furniture = i;
                    menu.close();
                    *gesture = Gesture::default();
                    *controls = Controls::default();
                    None
                }
                Some(Entry::Dish(dish)) => {
                    menu.message.clear();
                    if dish.ready(&garden) {
                        menu.preparation = Some(Preparation::new(dish, now));
                        None
                    } else {
                        Some(Err(format!(
                            "Es fehlen Zutaten · {}",
                            dish.supplies(&garden)
                        )))
                    }
                }
                Some(Entry::Destination(bell)) => {
                    let destination = bell.arrival();
                    let (player, transform) = &mut *player;
                    adventure.enter(Area::Island, player, transform, destination);
                    garden.checkpoint = Some(Checkpoint::new(Area::Island, destination));
                    *focus = GardenFocus::default();
                    *gesture = Gesture::default();
                    *controls = Controls::default();
                    menu.close();
                    Some(Ok(format!("Angekommen · {}", bell.name())))
                }
                Some(Entry::Step(_)) => {
                    let visible_time = menu.visible_time;
                    let game = menu
                        .preparation
                        .as_mut()
                        .expect("preparation offers step buttons");
                    game.try_step(index, visible_time);
                    if game.finish(&mut garden) {
                        let dish = game.dish;
                        menu.preparation = None;
                        Some(Ok(format!("+1 {}", dish.name())))
                    } else {
                        None
                    }
                }
                _ => {
                    menu.selected = entry;
                    menu.message.clear();
                    None
                }
            }
        }
        Choice::Fertilizer if page == FarmPage::SeedShop => Some(if garden.buy_fertilizer() {
            Ok("+2 Dünger".into())
        } else {
            Err("Für das Düngerpaket brauchst du 4 Münzen.".into())
        }),
        Choice::Back | Choice::Next => {
            menu.list_page = if choice == Choice::Back {
                menu.list_page.saturating_sub(1)
            } else {
                (menu.list_page + 1).min(menu.last_page(page, &garden))
            };
            menu.selected = None;
            menu.message.clear();
            None
        }
        Choice::Work if matches!(page, FarmPage::Workbench | FarmPage::BlueprintShop) => {
            let Some(Entry::Design(index)) = menu.selected else {
                return;
            };
            let result = if page == FarmPage::Workbench {
                let result = furniture::build(&mut garden, index);
                if result.is_ok() {
                    adventure.furniture = index;
                }
                result
            } else {
                furniture::buy(&mut garden, index)
            };
            Some(result)
        }
        Choice::SellOne | Choice::SellAll if page == FarmPage::Sale => {
            let Some(Entry::Stock(item)) = menu.selected else {
                return;
            };
            let total = garden.stock(item);
            let quantity = if choice == Choice::SellOne { 1 } else { total };
            Some(garden.sell_many(item, quantity).map_or_else(
                || Err("Dieser Verkauf ist gerade nicht möglich.".into()),
                |coins| {
                    if garden.stock(item) == 0 {
                        menu.selected = None;
                        menu.list_page = menu.list_page.min(menu.last_page(page, &garden));
                    }
                    Ok(format!(
                        "{quantity} × {} verkauft · +{coins} Münzen",
                        item.name()
                    ))
                },
            ))
        }
        _ => None,
    };
    match result {
        Some(Ok(message)) => {
            commit(&garden, &path, &mut notice, now, message, None, None);
            menu.message.clone_from(&notice.message);
        }
        Some(Err(message)) => menu.message = message,
        None => (),
    }
}

pub fn update(
    session: Res<Session>,
    mut menu: ResMut<FarmMenu>,
    adventure: Res<Adventure>,
    garden: Res<Garden>,
    time: Res<Time>,
    mut veil: Single<&mut Node, With<Pane>>,
    mut fields: Query<(&Field, &mut Text)>,
    mut choices: Query<(&Choice, &Interaction, &mut Node, &mut BackgroundColor), Without<Pane>>,
    mut meter: Query<(&HeatMeter, &mut Node), (Without<Pane>, Without<Choice>)>,
) {
    let Some(page) = menu.page.filter(|_| session.playing()) else {
        veil.display = Display::None;
        return;
    };
    veil.display = Display::Flex;
    menu.visible_time = time.elapsed_secs();

    let entries = menu.entries(page, &garden);
    let pages = entries.len().saturating_sub(1) / PAGE_SIZE;
    let entry_at = |i| entries.get(menu.list_page * PAGE_SIZE + i).copied();
    let balance = garden.farm.coins;
    let selected = menu.selected;
    let progress = menu.preparation.as_ref();
    let heat = progress
        .filter(|game| game.dish.method() == Method::Cook)
        .map(|game| game.heat(menu.visible_time));
    for (part, mut node) in &mut meter {
        match part {
            HeatMeter::Track => {
                node.display = if heat.is_some() {
                    Display::Flex
                } else {
                    Display::None
                }
            }
            HeatMeter::Needle => node.left = Val::Percent(100. * heat.unwrap_or(0.)),
        }
    }

    for (field, mut text) in &mut fields {
        let value = match *field {
            Field::Title if progress.is_some() => progress.unwrap().dish.name().into(),
            Field::Title => match page {
                FarmPage::Seeds => "Saatguttasche",
                FarmPage::SeedShop => "Saatgut und Dünger",
                FarmPage::Sale => "Verkaufstruhe",
                FarmPage::Workbench => "Werkbank",
                FarmPage::BlueprintShop => "Bauplanstand",
                FarmPage::Furnish => "Deine Möbel",
                FarmPage::Kitchen => "Hofküche",
                FarmPage::Travel(_) => "Wegglocken",
                FarmPage::Inventory => "Deine Tasche",
            }
            .into(),
            Field::Instruction if progress.is_some() => progress.unwrap().instructions(),
            Field::Instruction => match page {
                FarmPage::Seeds => "Wähle das Saatgut für dein nächstes Beet.".into(),
                FarmPage::SeedShop => {
                    format!("Du hast {balance} Münzen und {} Dünger.", garden.fertilizer)
                }
                FarmPage::Sale if pages > 0 => {
                    format!("{} / {} · {balance} Münzen", menu.list_page + 1, pages + 1)
                }
                FarmPage::Sale => format!("{balance} Münzen · Erlös sofort verfügbar"),
                FarmPage::Workbench => format!(
                    "Seite {} / {} · Wähle ein Möbel für die Werkbank.",
                    menu.list_page + 1,
                    pages + 1
                ),
                FarmPage::BlueprintShop => {
                    format!("{balance} Münzen · Wähle einen Bauplan für deine Werkbank.")
                }
                FarmPage::Furnish => {
                    "Wähle ein fertiges Möbel und stelle es auf einen freien Platz.".into()
                }
                FarmPage::Kitchen => {
                    "Wähle ein Rezept. Beim Fertigstellen werden die Zutaten verbraucht.".into()
                }
                FarmPage::Travel(_) if entries.is_empty() => {
                    "Repariere eine weitere Wegglocke, um dorthin zu reisen.".into()
                }
                FarmPage::Travel(_) => "Wähle eine reparierte Wegglocke als Ziel.".into(),
                FarmPage::Inventory => format!(
                    "Holz {} · Stein {} · Kupfer {} · Dünger {} · Futter {}",
                    garden.wood, garden.stone, garden.copper, garden.fertilizer, garden.farm.feed
                ),
            },
            Field::Message if progress.is_some() => {
                let game = progress.unwrap();
                let (done, total) = game.progress();
                format!(
                    "{done} / {total} · {}",
                    if game.message.is_empty() {
                        game.dish.method().title()
                    } else {
                        game.message
                    }
                )
            }
            Field::Message if !menu.message.is_empty() => menu.message.clone(),
            Field::Message if selected.is_some() => match selected.unwrap() {
                Entry::Stock(item) => format!(
                    "{} · {} Stück · {} Münzen für den Stapel",
                    item.label(),
                    garden.stock(item),
                    garden.stock(item).saturating_mul(item.value())
                ),
                Entry::Design(i) => format!(
                    "{} · {}",
                    CATALOG[i].name,
                    Entry::Design(i).detail(page, &garden)
                ),
                _ => String::new(),
            },
            Field::Message if page == FarmPage::Sale && entries.is_empty() => {
                "Ernte oder sammle etwas, um es hier zu verkaufen.".into()
            }
            Field::Message if page == FarmPage::Sale => {
                "Tippe auf eine Ware, um sie auszuwählen.".into()
            }
            Field::Message if page == FarmPage::Seeds => {
                format!("Gewählt · {}", adventure.crop.name())
            }
            Field::Message => String::new(),
            Field::CardName(i) => entry_at(i).map_or_else(String::new, Entry::title),
            Field::CardDetail(i) => {
                entry_at(i).map_or_else(String::new, |entry| entry.detail(page, &garden))
            }
            Field::Work => if page == FarmPage::BlueprintShop {
                "Bauplan kaufen"
            } else {
                "Herstellen"
            }
            .into(),
        };
        if text.0 != value {
            text.0 = value;
        }
    }

    for (&choice, &interaction, mut node, mut bg) in &mut choices {
        let (visible, enabled) = match choice {
            Choice::Card(i) => (
                entry_at(i).is_some(),
                entry_at(i).is_some_and(|entry| entry.enabled(page, &garden)),
            ),
            Choice::Close => (true, true),
            Choice::Fertilizer => (page == FarmPage::SeedShop, balance >= FERTILIZER_PRICE),
            Choice::Back => (pages > 0, menu.list_page > 0),
            Choice::Next => (pages > 0, menu.list_page < pages),
            Choice::SellOne | Choice::SellAll => (page == FarmPage::Sale, selected.is_some()),
            Choice::Work => {
                let available = match (page, selected) {
                    (FarmPage::Workbench, Some(Entry::Design(i))) => {
                        garden.blueprints[i] && !garden.crafted[i] && CATALOG[i].can_build(&garden)
                    }
                    (FarmPage::BlueprintShop, Some(Entry::Design(i))) => {
                        !garden.blueprints[i] && CATALOG[i].can_buy(&garden)
                    }
                    _ => false,
                };
                (
                    matches!(page, FarmPage::Workbench | FarmPage::BlueprintShop),
                    available,
                )
            }
        };
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        bg.0 = if let Choice::Card(i) = choice {
            let entry = entry_at(i);
            let highlighted = entry == selected && selected.is_some()
                || page == FarmPage::Seeds && entry == Some(Entry::Seed(adventure.crop));
            match (enabled, highlighted, interaction) {
                (false, _, _) => Color::srgb_u8(234, 227, 205),
                (_, true, _) | (_, _, Interaction::Pressed) => Color::srgb_u8(198, 214, 162),
                (_, _, Interaction::Hovered) => Color::srgb_u8(239, 220, 171),
                _ => Color::srgb_u8(241, 228, 191),
            }
        } else {
            ui::button_color(interaction, enabled)
        };
    }
}
