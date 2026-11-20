use bevy::prelude::*;

use crate::{
    Adventure, Controls, Notice,
    adventure::commit,
    garden::{Crop, FERTILIZER_PRICE, Garden, SavePath},
    homestead::{Product, Village},
    input::Gesture,
    session::Session,
    ui::{self, UiFonts},
};

const PAGE_SIZE: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FarmPage {
    Seeds,
    SeedShop,
    Sale,
}

#[derive(Resource, Default)]
pub struct FarmMenu {
    page: Option<FarmPage>,
    list_page: usize,
    selected: Option<Product>,
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

    fn last_page(garden: &Garden) -> usize {
        // Leave one page available for an empty crate.
        Product::stocked(garden).count().saturating_sub(1) / PAGE_SIZE
    }

    fn product_at(&self, index: usize, garden: &Garden) -> Option<Product> {
        Product::at(self.list_page * PAGE_SIZE + index, garden)
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
}

#[derive(Component)]
pub enum Field {
    Title,
    Instruction,
    Message,
    CardName(usize),
    CardDetail(usize),
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
                        ] {
                            footer
                                .spawn((action, ui::dialog_button(width)))
                                .with_child(fonts.button(caption, 14.));
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
    if choice == Choice::Close || page == FarmPage::Seeds && matches!(choice, Choice::Card(_)) {
        if let Choice::Card(i) = choice {
            adventure.crop = Crop::ALL[i];
        }
        menu.close();
        *gesture = Gesture::default();
        *controls = Controls::default();
        return;
    }

    let result = match choice {
        Choice::Card(i) if page == FarmPage::SeedShop => {
            Some(Village::Seeds.work(&mut garden, Crop::ALL[i], None))
        }
        Choice::Fertilizer if page == FarmPage::SeedShop => Some(if garden.buy_fertilizer() {
            Ok("+2 Dünger".into())
        } else {
            Err("Für das Düngerpaket brauchst du 4 Münzen.".into())
        }),
        Choice::Card(i) if page == FarmPage::Sale => {
            menu.selected = menu.product_at(i, &garden);
            menu.message.clear();
            None
        }
        Choice::Back | Choice::Next if page == FarmPage::Sale => {
            menu.list_page = if choice == Choice::Back {
                menu.list_page.saturating_sub(1)
            } else {
                (menu.list_page + 1).min(FarmMenu::last_page(&garden))
            };
            menu.selected = None;
            menu.message.clear();
            None
        }
        Choice::SellOne | Choice::SellAll if page == FarmPage::Sale => menu.selected.map(|item| {
            let total = garden.stock(item);
            let quantity = if choice == Choice::SellOne { 1 } else { total };
            let coins = garden
                .sell_many(item, quantity)
                .ok_or_else(|| "Dieser Verkauf ist gerade nicht möglich.".to_string())?;
            if garden.stock(item) == 0 {
                menu.selected = None;
                menu.list_page = menu.list_page.min(FarmMenu::last_page(&garden));
            }
            Ok(format!(
                "{} × {} verkauft · +{coins} Münzen",
                quantity,
                item.name()
            ))
        }),
        _ => None,
    };
    match result {
        Some(Ok(message)) => {
            commit(
                &garden,
                &path,
                &mut notice,
                time.elapsed_secs(),
                message,
                None,
                None,
            );
            menu.message.clone_from(&notice.message);
        }
        Some(Err(message)) => menu.message = message,
        None => (),
    }
}

pub fn update(
    session: Res<Session>,
    menu: Res<FarmMenu>,
    adventure: Res<Adventure>,
    garden: Res<Garden>,
    mut veil: Single<&mut Node, With<Pane>>,
    mut fields: Query<(&Field, &mut Text)>,
    mut choices: Query<(&Choice, &Interaction, &mut Node, &mut BackgroundColor), Without<Pane>>,
) {
    let Some(page) = menu.page.filter(|_| session.playing()) else {
        veil.display = Display::None;
        return;
    };
    veil.display = Display::Flex;

    let products = core::array::from_fn::<_, PAGE_SIZE, _>(|i| menu.product_at(i, &garden));
    let pages = FarmMenu::last_page(&garden);
    let balance = garden.farm.coins;
    let selected = menu.selected;
    for (field, mut text) in &mut fields {
        let value = match *field {
            Field::Title => match page {
                FarmPage::Seeds => "Saatguttasche".into(),
                FarmPage::SeedShop => "Saatgut und Dünger".into(),
                FarmPage::Sale => "Verkaufstruhe".into(),
            },
            Field::Instruction => match page {
                FarmPage::Seeds => "Wähle das Saatgut für dein nächstes Beet.".into(),
                FarmPage::SeedShop => {
                    format!("Du hast {balance} Münzen und {} Dünger.", garden.fertilizer)
                }
                FarmPage::Sale if pages > 0 => {
                    format!(
                        "{} / {} · {} Münzen",
                        menu.list_page + 1,
                        pages + 1,
                        balance
                    )
                }
                FarmPage::Sale => format!("{} Münzen · Erlös sofort verfügbar", balance),
            },
            Field::Message if !menu.message.is_empty() => menu.message.clone(),
            Field::Message if page == FarmPage::Sale => {
                if let Some(item) = selected {
                    format!(
                        "{} · {} Stück · {} Münzen für den Stapel",
                        item.label(),
                        garden.stock(item),
                        garden.stock(item).saturating_mul(item.value()),
                    )
                } else if products[0].is_some() {
                    "Tippe auf eine Ware, um sie auszuwählen.".into()
                } else {
                    "Ernte oder sammle etwas, um es hier zu verkaufen.".into()
                }
            }
            Field::Message if page == FarmPage::Seeds => {
                format!("Gewählt · {}", adventure.crop.name())
            }
            Field::Message => String::new(),
            Field::CardName(i) if page == FarmPage::Sale => {
                products[i].map_or_else(String::new, Product::label)
            }
            Field::CardDetail(i) if page == FarmPage::Sale => {
                products[i].map_or_else(String::new, |item| {
                    format!(
                        "{} Stück · {} Münzen je Stück",
                        garden.stock(item),
                        item.value()
                    )
                })
            }
            Field::CardName(i) => Crop::ALL[i].name().into(),
            Field::CardDetail(i) if page == FarmPage::SeedShop => {
                let crop = Crop::ALL[i];
                format!(
                    "3 Samen · {} Münzen · {} im Vorrat",
                    crop.price(),
                    garden.seed_count(crop)
                )
            }
            Field::CardDetail(i) => {
                let crop = Crop::ALL[i];
                format!(
                    "{} Samen · {} gegossene Tage",
                    garden.seed_count(crop),
                    crop.days()
                )
            }
        };
        if text.0 != value {
            text.0 = value;
        }
    }
    for (&choice, &interaction, mut node, mut bg) in &mut choices {
        let (visible, enabled) = match choice {
            Choice::Card(i) if page == FarmPage::Sale => (products[i].is_some(), true),
            Choice::Card(i) if page == FarmPage::SeedShop => {
                (true, balance >= Crop::ALL[i].price())
            }
            Choice::Card(_) | Choice::Close => (true, true),
            Choice::Fertilizer => (page == FarmPage::SeedShop, balance >= FERTILIZER_PRICE),
            Choice::Back => (page == FarmPage::Sale && pages > 0, menu.list_page > 0),
            Choice::Next => (page == FarmPage::Sale && pages > 0, menu.list_page < pages),
            Choice::SellOne | Choice::SellAll => (page == FarmPage::Sale, selected.is_some()),
        };
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        bg.0 = if let Choice::Card(i) = choice {
            let highlighted = match page {
                FarmPage::Sale => selected.is_some() && selected == products[i],
                FarmPage::Seeds => adventure.crop == Crop::ALL[i],
                FarmPage::SeedShop => false,
            };
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
