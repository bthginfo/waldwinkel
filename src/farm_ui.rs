use bevy::prelude::*;

use crate::{
    Adventure, Controls, GardenFocus, Notice,
    adventure::{Area, commit},
    cinematic::Clip,
    duel::{Battle, CAMP_INTERVAL, Card, DECK_SIZE, Gear, Talent},
    furniture::{self, Blueprint, CATALOG},
    garden::{Crop, FERTILIZER_PRICE, Garden, SavePath},
    harbor::{self, Phase},
    homestead::{Product, Village},
    input::Gesture,
    inventory::{self, Material},
    onboarding::Onboarding,
    processing::{Dish, Method, Preparation},
    progression::{self, CARDS_FOR_SALE, GearSource},
    regions::Bell,
    seasons::{self, CONTRIBUTION, MEETING_START, PRIZE},
    session::{Checkpoint, Session},
    story::{Guide, GuideTab},
    ui::{self, UiFonts},
    weather::Season,
    workshop::{BuildingPlan, Project, ToolKind},
    world::Gardener,
};

const PAGE_SIZE: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FarmPage {
    Seeds,
    SeedShop,
    Sale,
    Workbench,
    Projects,
    Tools { at_forge: bool },
    BlueprintShop,
    Furnish,
    Kitchen,
    Travel(Bell),
    Inventory,
    Binder,
    Talents,
    CardShop,
    Gear(GearSource),
    Expedition,
    Journal(GuideTab),
    Calendar,
    Harbor { at_site: bool },
    Cinematic(Clip),
    SeasonMeeting,
    Battle,
}

#[derive(Resource, Default)]
pub struct FarmMenu {
    page: Option<FarmPage>,
    list_page: usize,
    selected: Option<Entry>,
    preparation: Option<Preparation>,
    visible_time: f32,
    clip_time: f32,
    message: String,
    battle: Option<Battle>,
    deck_draft: Vec<Card>,
}

impl FarmMenu {
    pub fn open(&self) -> bool {
        self.page.is_some()
    }

    pub fn clip(&self) -> Option<(Clip, f32)> {
        match self.page {
            Some(FarmPage::Cinematic(clip)) => Some((clip, self.clip_time)),
            _ => None,
        }
    }

    pub fn advance_clip(&mut self, delta: f32) {
        self.clip_time += delta;
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

    pub fn show_battle(&mut self, battle: Battle) {
        self.show(FarmPage::Battle);
        self.battle = Some(battle);
    }

    pub fn battle(&self) -> Option<&Battle> {
        self.battle.as_ref()
    }

    pub fn battle_mut(&mut self) -> Option<&mut Battle> {
        self.battle.as_mut()
    }

    pub fn load_deck(&mut self, garden: &Garden) {
        self.deck_draft = garden.rpg.deck().to_vec();
    }

    fn navigate(&mut self, page: FarmPage) {
        self.page = Some(page);
        self.list_page = 0;
        self.selected = None;
        self.message.clear();
    }

    fn copies(&self, card: Card) -> usize {
        self.deck_draft.iter().filter(|&&copy| copy == card).count()
    }

    fn entries(&self, page: FarmPage, garden: &Garden) -> Vec<Entry> {
        match page {
            FarmPage::Seeds | FarmPage::SeedShop => {
                Crop::ALL.into_iter().map(Entry::Seed).collect()
            }
            FarmPage::Sale => Product::stocked(garden).map(Entry::Stock).collect(),
            FarmPage::Inventory => Material::stocked(garden)
                .map(Entry::Material)
                .chain(Product::stocked(garden).map(Entry::Stock))
                .collect(),
            FarmPage::Travel(from) => Bell::ALL
                .into_iter()
                .filter(|&bell| bell != from && bell.ready(garden))
                .map(Entry::Destination)
                .collect(),
            FarmPage::Projects => Project::available(garden).map(Entry::Project).collect(),
            FarmPage::Tools { .. } => ToolKind::ALL.into_iter().map(Entry::Tool).collect(),
            FarmPage::Workbench | FarmPage::BlueprintShop | FarmPage::Furnish => (page
                == FarmPage::BlueprintShop)
                .then_some(BuildingPlan::ALL)
                .into_iter()
                .flatten()
                .map(Entry::BuildingPlan)
                .chain(
                    (page == FarmPage::Workbench && garden.harbor.phase == Phase::Landing)
                        .then_some(Entry::Fittings),
                )
                .chain(
                    CATALOG
                        .iter()
                        .enumerate()
                        .filter(|&(index, design)| match page {
                            FarmPage::BlueprintShop => {
                                matches!(design.blueprint, Blueprint::Shop { .. })
                            }
                            FarmPage::Furnish => garden.crafted[index],
                            _ => garden.blueprints[index],
                        })
                        .map(|(index, _)| Entry::Design(index)),
                )
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
            FarmPage::Binder => Card::ALL
                .into_iter()
                .filter(|&card| garden.rpg.knows(card))
                .map(Entry::CombatCard)
                .collect(),
            FarmPage::Talents => Talent::ALL.into_iter().map(Entry::Talent).collect(),
            FarmPage::CardShop => CARDS_FOR_SALE.into_iter().map(Entry::CombatCard).collect(),
            FarmPage::Gear(_) => Gear::ALL.into_iter().map(Entry::Gear).collect(),
            FarmPage::Expedition => std::iter::once(1)
                .chain((1..=garden.rpg.camp() / CAMP_INTERVAL).map(|index| index * CAMP_INTERVAL))
                .map(Entry::Level)
                .collect(),
            FarmPage::Journal(tab) => Guide::entries(tab).map(Entry::Guide).collect(),
            FarmPage::Calendar => Season::ALL.into_iter().map(Entry::Season).collect(),
            FarmPage::SeasonMeeting | FarmPage::Harbor { .. } | FarmPage::Cinematic(_) => {
                Vec::new()
            }
            FarmPage::Battle => Vec::new(),
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
    Material(Material),
    Fittings,
    Project(Project),
    Tool(ToolKind),
    BuildingPlan(BuildingPlan),
    Design(usize),
    Dish(Dish),
    Step(&'static str),
    Destination(Bell),
    CombatCard(Card),
    Talent(Talent),
    Gear(Gear),
    Level(u32),
    Guide(Guide),
    Season(Season),
}

impl Entry {
    fn title(self, garden: &Garden) -> String {
        match self {
            Self::Seed(crop) => crop.name().into(),
            Self::Stock(item) => item.label(),
            Self::Material(item) => item.name(),
            Self::Fittings => "Kupferbeschläge".into(),
            Self::Project(project) => project.name().into(),
            Self::Tool(tool) => format!("{} · {}", tool.name(), tool.tier(garden)),
            Self::BuildingPlan(plan) => plan.name().into(),
            Self::Design(i) => CATALOG[i].name.into(),
            Self::Dish(dish) => dish.name().into(),
            Self::Step(text) => text.into(),
            Self::Destination(bell) => bell.name().into(),
            Self::CombatCard(card) => card.name().into(),
            Self::Talent(talent) => talent.name().into(),
            Self::Gear(gear) => gear.name().into(),
            Self::Level(1) => "Ebene 1".into(),
            Self::Level(level) => format!("Glockenrast {level}"),
            Self::Guide(guide) => guide.title(garden).into(),
            Self::Season(season) if season == Season::at(garden.day) => {
                format!("{} · Jetzt", season.label())
            }
            Self::Season(season) => season.label().into(),
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
            Self::Stock(item) if page == FarmPage::Inventory => format!("×{}", garden.stock(item)),
            Self::Stock(item) => format!(
                "{} Stück · {} Münzen je Stück",
                garden.stock(item),
                item.value()
            ),
            Self::Material(item) => format!("×{}", item.amount(garden)),
            Self::Fittings if garden.harbor.fittings => "Fertig · Zum Bautisch bringen".into(),
            Self::Fittings => harbor::fitting_supplies(garden),
            Self::Project(project) => project.detail(garden),
            Self::Tool(tool) => {
                tool.requirements(garden, page == FarmPage::Tools { at_forge: true })
            }
            Self::BuildingPlan(plan) => plan.detail(garden),
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
            Self::CombatCard(card) if page == FarmPage::CardShop => {
                let cost = if garden.rpg.knows(card) {
                    "In deiner Sammlung".into()
                } else {
                    format!(
                        "{} Münzen",
                        progression::card_price(card).unwrap_or_default()
                    )
                };
                format!("{cost} · {}", card.description())
            }
            Self::CombatCard(card) => format!("Kraft {} · {}", card.cost(), card.description()),
            Self::Talent(talent) => format!(
                "Rang {} · {}",
                garden.rpg.ranks[talent as usize],
                talent.text()
            ),
            Self::Gear(gear) => match page {
                FarmPage::Gear(source) => source.detail(garden, gear),
                _ => String::new(),
            },
            Self::Level(1) => "Am Eingang beginnen".into(),
            Self::Level(_) => "Hier beginnen und Leben auffüllen".into(),
            Self::Guide(guide) => guide.detail(garden),
            Self::Season(season) => {
                let discovery = if garden.seasons.discovered[season as usize] {
                    season.find_name()
                } else {
                    "Fund offen"
                };
                format!("{} · {discovery}", season.meeting())
            }
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
    AddCopy,
    RemoveCopy,
    SaveDeck,
    Talents,
    Gear,
    Projects,
    Tools,
    Recipes,
    Return,
    Guide(GuideTab),
    Calendar,
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
pub struct ItemPicture(usize);

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
                    width: Val::Px(590.),
                    max_width: Val::Percent(95.),
                    padding: UiRect::all(Val::Px(22.)),
                    border: UiRect::all(Val::Px(1.)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(13.),
                    ..default()
                },
                BorderRadius::all(Val::Px(24.)),
                BackgroundColor(ui::cream()),
                BorderColor(ui::stroke()),
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
                                        flex_direction: FlexDirection::Row,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(9.),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb_u8(241, 228, 191)),
                                    BorderRadius::all(Val::Px(12.)),
                                ))
                                .with_children(|card| {
                                    card.spawn((
                                        ItemPicture(i),
                                        ImageNode::default(),
                                        Node {
                                            display: Display::None,
                                            width: Val::Px(43.),
                                            height: Val::Px(43.),
                                            flex_shrink: 0.,
                                            ..default()
                                        },
                                    ));
                                    card.spawn(Node {
                                        flex_direction: FlexDirection::Column,
                                        flex_grow: 1.,
                                        row_gap: Val::Px(1.),
                                        ..default()
                                    })
                                    .with_children(|text| {
                                        text.spawn((Field::CardName(i), fonts.bold("", 16.)));
                                        text.spawn((Field::CardDetail(i), fonts.body("", 13.)));
                                    });
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
                            (Choice::AddCopy, "+ Karte", 84.),
                            (Choice::RemoveCopy, "− Karte", 84.),
                            (Choice::SaveDeck, "Deck speichern", 129.),
                            (Choice::Talents, "Talente", 84.),
                            (Choice::Gear, "Ausrüstung", 106.),
                            (Choice::Projects, "Bauprojekte", 110.),
                            (Choice::Tools, "Werkzeuge", 106.),
                            (Choice::Recipes, "Möbel", 84.),
                            (Choice::Return, "Zurück", 84.),
                            (Choice::Guide(GuideTab::Orders), "Aufträge", 104.),
                            (Choice::Guide(GuideTab::Story), "Geschichte", 112.),
                            (Choice::Guide(GuideTab::Town), "Dorf", 76.),
                            (Choice::Calendar, "Kalender", 104.),
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
    let Some(page) = menu
        .page
        .filter(|&page| session.playing() && page != FarmPage::Battle)
    else {
        return;
    };
    let Some(choice) = choices
        .iter()
        .find_map(|(choice, pressed)| (*pressed == Interaction::Pressed).then_some(*choice))
        .or_else(|| {
            menu.clip()
                .filter(|(clip, seconds)| *seconds >= clip.duration())
                .map(|_| Choice::Close)
        })
    else {
        return;
    };
    let now = time.elapsed_secs();
    let result = match choice {
        Choice::Close => {
            menu.close();
            *gesture = Gesture::default();
            *controls = Controls::default();
            if page == FarmPage::Cinematic(Clip::Arrival)
                && garden.onboarding == Onboarding::Welcome
            {
                garden.onboarding = Onboarding::Till;
                Some(Ok("Auftrag gestartet · Dein erster Morgen".into()))
            } else {
                None
            }
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
                Some(Entry::Level(level)) => {
                    let (player, transform) = &mut *player;
                    adventure.enter_duel(level, level != 1, &mut garden, player, transform);
                    garden.checkpoint = Some(Checkpoint::new(adventure.area, player.position));
                    *focus = GardenFocus::default();
                    *gesture = Gesture::default();
                    *controls = Controls::default();
                    menu.close();
                    Some(Ok(format!("Dämmergänge · Ebene {level}")))
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
        Choice::Guide(tab) if matches!(page, FarmPage::Journal(_) | FarmPage::Calendar) => {
            menu.navigate(FarmPage::Journal(tab));
            None
        }
        Choice::Calendar if matches!(page, FarmPage::Journal(_) | FarmPage::SeasonMeeting) => {
            menu.navigate(FarmPage::Calendar);
            None
        }
        Choice::Talents if page == FarmPage::Binder => {
            menu.navigate(FarmPage::Talents);
            None
        }
        Choice::Projects
            if page == FarmPage::Workbench || page == (FarmPage::Tools { at_forge: false }) =>
        {
            menu.navigate(FarmPage::Projects);
            None
        }
        Choice::Tools
            if matches!(page, FarmPage::Workbench | FarmPage::Projects)
                && garden.workshop.restored =>
        {
            menu.navigate(FarmPage::Tools { at_forge: false });
            None
        }
        Choice::Recipes
            if matches!(
                page,
                FarmPage::Projects | FarmPage::Tools { at_forge: false }
            ) && garden.workshop.restored =>
        {
            menu.navigate(FarmPage::Workbench);
            None
        }
        Choice::Gear if page == FarmPage::Workbench || page == FarmPage::CardShop => {
            let source = if page == FarmPage::Workbench {
                GearSource::Workbench
            } else {
                GearSource::Smith
            };
            menu.navigate(FarmPage::Gear(source));
            None
        }
        Choice::Return => {
            match page {
                FarmPage::Talents => menu.navigate(FarmPage::Binder),
                FarmPage::Gear(GearSource::Workbench) => menu.navigate(FarmPage::Workbench),
                FarmPage::Gear(GearSource::Smith) => menu.navigate(FarmPage::CardShop),
                _ => {}
            }
            None
        }
        Choice::AddCopy | Choice::RemoveCopy if page == FarmPage::Binder => {
            let Some(Entry::CombatCard(card)) = menu.selected else {
                return;
            };
            if choice == Choice::AddCopy
                && menu.deck_draft.len() < DECK_SIZE
                && menu.copies(card) < 2
            {
                menu.deck_draft.push(card);
            } else if choice == Choice::RemoveCopy
                && let Some(index) = menu.deck_draft.iter().rposition(|&copy| copy == card)
            {
                menu.deck_draft.remove(index);
            }
            menu.message.clear();
            None
        }
        Choice::SaveDeck if page == FarmPage::Binder && adventure.area == Area::Home => Some(
            garden
                .rpg
                .save_deck(&menu.deck_draft)
                .map(|()| "Dein Kampfdeck ist gespeichert.".into())
                .map_err(str::to_owned),
        ),
        Choice::Work if page == FarmPage::CardShop => {
            let Some(Entry::CombatCard(card)) = menu.selected else {
                return;
            };
            Some(progression::buy_card(&mut garden, card))
        }
        Choice::Work if page == FarmPage::Talents => {
            let Some(Entry::Talent(talent)) = menu.selected else {
                return;
            };
            Some(if garden.rpg.train(talent) {
                Ok(format!(
                    "{} · Rang {}",
                    talent.name(),
                    garden.rpg.ranks[talent as usize]
                ))
            } else {
                Err("Für diesen Rang fehlen Punkte, oder das Talent ist bereits ausgebaut.".into())
            })
        }
        Choice::Work if matches!(page, FarmPage::Harbor { .. }) && !garden.harbor.started => {
            Some(harbor::begin(&mut garden))
        }
        Choice::Work if page == (FarmPage::Harbor { at_site: true }) => {
            let result = harbor::build(&mut garden);
            if result.is_ok() && garden.harbor.phase == Phase::Open {
                adventure.map = false;
                menu.show(FarmPage::Cinematic(Clip::Opening));
            }
            Some(result)
        }
        Choice::Work
            if matches!(
                page,
                FarmPage::Journal(_) | FarmPage::Calendar | FarmPage::Harbor { at_site: false }
            ) =>
        {
            let guide = match (page, menu.selected) {
                (FarmPage::Calendar, _) => Guide::SeasonMeeting,
                (FarmPage::Harbor { .. }, _) => Guide::Pier,
                (_, Some(Entry::Guide(guide))) => guide,
                _ => return,
            };
            if guide == Guide::Farm {
                Some(if garden.claim() {
                    Ok("Hofaufgabe erfüllt · +5 Münzen und Saatgut".into())
                } else {
                    Err("Die Hofaufgabe ist noch offen.".into())
                })
            } else if let Some(target) = guide.target(&garden) {
                adventure.tracked = Some(guide);
                if adventure.area.outdoors() {
                    adventure.map = true;
                    adventure.focus = Some(target);
                    *focus = GardenFocus::default();
                    *gesture = Gesture::default();
                    *controls = Controls::default();
                    menu.close();
                    None
                } else {
                    Some(Err(
                        "Ziel markiert. Draußen kannst du es auf der Weltkarte sehen.".into(),
                    ))
                }
            } else {
                None
            }
        }
        Choice::Work if page == FarmPage::SeasonMeeting => Some(seasons::contribute(&mut garden)),
        Choice::Work if matches!(page, FarmPage::Gear(_)) => {
            let (FarmPage::Gear(source), Some(Entry::Gear(gear))) = (page, menu.selected) else {
                return;
            };
            Some(source.equip(&mut garden, gear))
        }
        Choice::Work if page == FarmPage::Projects => {
            let Some(Entry::Project(project)) = menu.selected else {
                return;
            };
            let result = project.build(&mut garden);
            if result.is_ok() {
                if project == Project::Restore {
                    menu.navigate(FarmPage::Workbench);
                } else {
                    menu.selected = None;
                }
            }
            Some(result)
        }
        Choice::Work if matches!(page, FarmPage::Tools { .. }) => {
            let Some(Entry::Tool(tool)) = menu.selected else {
                return;
            };
            Some(tool.upgrade(&mut garden, page == FarmPage::Tools { at_forge: true }))
        }
        Choice::Work
            if page == FarmPage::BlueprintShop
                && matches!(menu.selected, Some(Entry::BuildingPlan(_))) =>
        {
            let Some(Entry::BuildingPlan(plan)) = menu.selected else {
                unreachable!()
            };
            Some(plan.buy(&mut garden))
        }
        Choice::Work if matches!(page, FarmPage::Workbench | FarmPage::BlueprintShop) => {
            if page == FarmPage::Workbench && menu.selected == Some(Entry::Fittings) {
                let result = harbor::make_fittings(&mut garden);
                if let Ok(message) = &result {
                    commit(
                        &garden,
                        &path,
                        &mut notice,
                        now,
                        message.clone(),
                        None,
                        None,
                    );
                }
                menu.message = result.unwrap_or_else(|error| error);
                return;
            }
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
    assets: Res<AssetServer>,
    mut veil: Single<&mut Node, With<Pane>>,
    mut fields: Query<(&Field, &mut Text)>,
    mut choices: Query<(&Choice, &Interaction, &mut Node, &mut BackgroundColor), Without<Pane>>,
    mut meter: Query<(&HeatMeter, &mut Node), (Without<Pane>, Without<Choice>)>,
    mut pictures: Query<
        (&ItemPicture, &mut ImageNode, &mut Node),
        (Without<Pane>, Without<Choice>, Without<HeatMeter>),
    >,
) {
    let Some(page) = menu.page.filter(|&page| {
        session.playing() && !matches!(page, FarmPage::Battle | FarmPage::Cinematic(_))
    }) else {
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
    for (picture, mut image, mut node) in &mut pictures {
        let entry = entry_at(picture.0);
        let slug = match entry {
            Some(Entry::Stock(item)) => Some(inventory::product_icon(item)),
            Some(Entry::Material(item)) => Some(item.icon()),
            Some(Entry::Seed(crop)) => Some(inventory::crop_icon(crop)),
            Some(Entry::Fittings) => Some("fittings"),
            Some(Entry::BuildingPlan(BuildingPlan::Barn(_))) => Some("plan_barn"),
            Some(Entry::BuildingPlan(BuildingPlan::Room(_)) | Entry::Project(Project::Room(_))) => {
                Some("plan_room")
            }
            Some(Entry::Design(_) | Entry::Project(_)) => Some("plan_furniture"),
            _ => None,
        }
        .map(str::to_owned)
        .or_else(|| {
            if let Some(Entry::Tool(tool)) = entry {
                Some(tool.icon(&garden))
            } else {
                None
            }
        });
        node.display = if slug.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if let Some(slug) = slug {
            let next = assets.load(format!("ui/items/{slug}.png"));
            if image.image != next {
                image.image = next;
            }
        }
    }
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
                FarmPage::Workbench => "Hofwerkbank · Möbel",
                FarmPage::Projects if !garden.workshop.restored => "Die alte Hofwerkbank",
                FarmPage::Projects => "Hofwerkbank · Bauprojekte",
                FarmPage::Tools { at_forge: false } => "Hofwerkbank · Werkzeuge",
                FarmPage::Tools { at_forge: true } => "Schmiedetisch",
                FarmPage::BlueprintShop => "Bauplanstand",
                FarmPage::Furnish => "Deine Möbel",
                FarmPage::Kitchen => "Hofküche",
                FarmPage::Travel(_) => "Wegglocken",
                FarmPage::Inventory => "Deine Tasche",
                FarmPage::Binder => "Kartenbinder",
                FarmPage::Talents => "Kampftalente",
                FarmPage::CardShop => "Runas Karten",
                FarmPage::Gear(_) => "Ausrüstung",
                FarmPage::Expedition => "Dämmergänge",
                FarmPage::Journal(GuideTab::Story) => "Das Licht der Wegglocken",
                FarmPage::Journal(GuideTab::Orders) => "Aufträge",
                FarmPage::Journal(GuideTab::Town) => "Eichenhafen",
                FarmPage::Harbor { .. } => "Der alte Fischersteg",
                FarmPage::Cinematic(_) => "",
                FarmPage::Calendar => "Jahreszeitenkalender",
                FarmPage::SeasonMeeting => Season::at(garden.day).meeting(),
                FarmPage::Battle => "",
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
                    "{} gelernte Baupläne · Seite {}/{}",
                    garden.blueprints.iter().filter(|&&known| known).count(),
                    menu.list_page + 1,
                    pages + 1
                ),
                FarmPage::Projects if !garden.workshop.restored => {
                    "Repariere die Werkbank und lerne deine ersten drei Möbelbaupläne.".into()
                }
                FarmPage::Projects => format!(
                    "Wohnraum · {} von 14 Möbelplätzen. Baupläne gibt es am Bauplanstand.",
                    garden.workshop.slots()
                ),
                FarmPage::Tools { at_forge: false } => {
                    "Verbessere deine Werkzeuge auf Kupfer. Die Silberstufe fertigt der Schmiedetisch.".into()
                }
                FarmPage::Tools { at_forge: true } => {
                    "Hier kannst du Werkzeuge mit Silber aus der tiefen Mine verbessern.".into()
                }
                FarmPage::BlueprintShop => {
                    format!("{balance} Münzen · Baupläne für Ställe, Anbauten und Möbel.")
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
                    "{} Stapel · Seite {}/{}",
                    entries.len(),
                    menu.list_page + 1,
                    pages + 1
                ),
                FarmPage::Binder => format!(
                    "{}/{} Karten im Deck · {} in der Sammlung · Seite {}/{}",
                    menu.deck_draft.len(),
                    DECK_SIZE,
                    entries.len(),
                    menu.list_page + 1,
                    pages + 1
                ),
                FarmPage::Talents => format!(
                    "{} Talentpunkte · {} / {} Leben · Siege geben Talentpunkte",
                    garden.rpg.points,
                    garden.rpg.hp,
                    garden.rpg.max_hp()
                ),
                FarmPage::CardShop => format!(
                    "Du hast {balance} Münzen. Gekaufte Karten findest du im Binder daheim."
                ),
                FarmPage::Gear(GearSource::Workbench) => {
                    format!("{balance} Münzen · Fertige eigene Ausrüstung.")
                }
                FarmPage::Gear(GearSource::Smith) => {
                    format!("{balance} Münzen · Kaufe Ausrüstung bei Runa.")
                }
                FarmPage::Expedition => format!(
                    "{} / 25 Hüter besiegt · {} / {} Leben",
                    garden.rpg.cleared,
                    garden.rpg.hp,
                    garden.rpg.max_hp()
                ),
                FarmPage::Journal(GuideTab::Story) => format!(
                    "{}/3 Klangsteine · Wähle ein Ziel, um es auf der Karte zu sehen.",
                    garden.story.voices()
                ),
                FarmPage::Journal(GuideTab::Orders) => {
                    "Hof, Eichenhafen und die Außenregionen".into()
                }
                FarmPage::Journal(GuideTab::Town) => {
                    "Wähle eine Person oder den Fischersteg für die Karte.".into()
                }
                FarmPage::Harbor { .. } => garden.harbor.phase.label().into(),
                FarmPage::Cinematic(_) => String::new(),
                FarmPage::Calendar => {
                    let season = Season::at(garden.day);
                    let day = Season::day(garden.day);
                    let left = Season::LENGTH + 1 - day;
                    let next = Season::ALL[(season as usize + 1) % Season::ALL.len()];
                    let change = if left == 1 {
                        "morgen".into()
                    } else {
                        format!("in {left} Tagen")
                    };
                    format!(
                        "{} · Tag {day}/{} · {} beginnt {change}.",
                        season.label(),
                        Season::LENGTH,
                        next.label()
                    )
                }
                FarmPage::SeasonMeeting => {
                    let season = Season::at(garden.day);
                    format!(
                        "{} lädt ein. {}",
                        season.host().name(),
                        seasons::meeting_status(&garden)
                    )
                }
                FarmPage::Battle => String::new(),
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
            Field::Message if matches!(page, FarmPage::Harbor { .. }) => {
                let phase = garden.harbor.phase;
                let needs = harbor::supplies(&garden);
                format!("{}\n{needs}", phase.instructions())
            }
            Field::Message if page == FarmPage::Calendar => {
                let season = match selected {
                    Some(Entry::Season(season)) => season,
                    _ => Season::at(garden.day),
                };
                let item = garden.stock(Product::Seasonal(season));
                let status = if garden.seasons.celebrated[season as usize] {
                    "Teilgenommen"
                } else {
                    "Treffen offen"
                };
                format!(
                    "{} × {} · {} · Funde {}/4 · Dorftreffen {}/4\n{} Treffen an Tag {MEETING_START}–7.",
                    item,
                    season.find_name(),
                    status,
                    garden.seasons.collection(),
                    garden.seasons.meetings(),
                    season.find_hint()
                )
            }
            Field::Message if page == FarmPage::SeasonMeeting => {
                let season = Season::at(garden.day);
                let supplies = garden.stock(Product::Seasonal(season));
                format!(
                    "{}\nBeitrag {supplies}/{CONTRIBUTION} {} · Belohnung {PRIZE} Münzen und 3 Samen.\nDas Jahreszeitenregal gibt es nach allen vier Treffen.",
                    season.invitation(),
                    season.find_name()
                )
            }
            Field::Message if selected.is_some() => match selected.unwrap() {
                Entry::Tool(tool) => tool.effect().into(),
                Entry::BuildingPlan(BuildingPlan::Barn(_)) => "Den Stall baust du auf der markierten Hoffläche.".into(),
                Entry::BuildingPlan(BuildingPlan::Room(_)) => "Den Anbau baust du an deiner Hofwerkbank.".into(),
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
                Entry::CombatCard(card) if page == FarmPage::Binder => {
                    format!(
                        "{} · {} von 2 möglichen Kopien im Deck",
                        card.name(),
                        menu.copies(card)
                    )
                }
                Entry::Talent(talent) => match garden.rpg.train_cost(talent) {
                    Some(cost) => format!("Nächster Rang · {cost} Talentpunkte"),
                    None => "Höchster Rang erreicht".into(),
                },
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
            Field::Message if page == FarmPage::Binder => {
                "Wähle eine Karte. Schließen verwirft ungespeicherte Änderungen.".into()
            }
            Field::Message => String::new(),
            Field::CardName(i) => {
                entry_at(i).map_or_else(String::new, |entry| entry.title(&garden))
            }
            Field::CardDetail(i) => entry_at(i).map_or_else(String::new, |entry| {
                let extra = entry.detail(page, &garden);
                match entry {
                    Entry::CombatCard(card) if page == FarmPage::Binder => {
                        format!("{} im Deck · {extra}", menu.copies(card))
                    }
                    _ => extra,
                }
            }),
            Field::Work => match (page, selected) {
                (FarmPage::BlueprintShop, _) => "Bauplan kaufen",
                (FarmPage::Projects, Some(Entry::Project(Project::Restore))) => "Werkbank reparieren",
                (FarmPage::Projects, _) => "Bauen",
                (FarmPage::Tools { .. }, _) => "Verbessern",
                (FarmPage::CardShop, _) => "Karte kaufen",
                (FarmPage::Talents, _) => "Rang steigern",
                (FarmPage::Gear(_), Some(Entry::Gear(gear)))
                    if garden.rpg.gear_owned[gear as usize] =>
                {
                    "Ausrüsten"
                }
                (FarmPage::Gear(GearSource::Smith), _) => "Kaufen & anlegen",
                (FarmPage::Gear(GearSource::Workbench), _) => "Bauen & anlegen",
                (FarmPage::Journal(_), Some(Entry::Guide(Guide::Farm))) => "Belohnung holen",
                (FarmPage::Journal(_), _) => "Auf Karte zeigen",
                (FarmPage::Calendar, _) => "Treffpunkt zeigen",
                (FarmPage::Harbor { .. }, _) if !garden.harbor.started => "Auftrag annehmen",
                (FarmPage::Harbor { at_site: true }, _) => garden.harbor.phase.action(),
                (FarmPage::Harbor { at_site: false }, _) => "Auf Karte zeigen",
                (FarmPage::Workbench, Some(Entry::Fittings)) => "Beschläge fertigen",
                (FarmPage::SeasonMeeting, _) => "Beitrag abgeben",
                _ => "Herstellen",
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
            Choice::Talents => (page == FarmPage::Binder, true),
            Choice::Projects => (
                page == FarmPage::Workbench || page == (FarmPage::Tools { at_forge: false }),
                true,
            ),
            Choice::Tools => (
                matches!(page, FarmPage::Workbench | FarmPage::Projects)
                    && garden.workshop.restored,
                true,
            ),
            Choice::Recipes => (
                matches!(
                    page,
                    FarmPage::Projects | FarmPage::Tools { at_forge: false }
                ) && garden.workshop.restored,
                true,
            ),
            Choice::Gear => (
                matches!(page, FarmPage::Workbench | FarmPage::CardShop),
                true,
            ),
            Choice::Return => (matches!(page, FarmPage::Talents | FarmPage::Gear(_)), true),
            Choice::Guide(tab) => (
                page == FarmPage::Calendar
                    || matches!(page, FarmPage::Journal(current) if current != tab),
                true,
            ),
            Choice::Calendar => (
                matches!(page, FarmPage::Journal(_) | FarmPage::SeasonMeeting),
                true,
            ),
            Choice::AddCopy => (
                page == FarmPage::Binder,
                matches!(selected, Some(Entry::CombatCard(card)) if menu.copies(card) < 2)
                    && menu.deck_draft.len() < DECK_SIZE,
            ),
            Choice::RemoveCopy => (
                page == FarmPage::Binder,
                matches!(selected, Some(Entry::CombatCard(card)) if menu.copies(card) > 0),
            ),
            Choice::SaveDeck => (page == FarmPage::Binder, menu.deck_draft.len() == DECK_SIZE),
            Choice::Work => {
                let available = match (page, selected) {
                    (FarmPage::Projects, Some(Entry::Project(project))) => {
                        project.can_build(&garden)
                    }
                    (FarmPage::Tools { at_forge }, Some(Entry::Tool(tool))) => {
                        tool.can_upgrade(&garden, at_forge)
                    }
                    (FarmPage::BlueprintShop, Some(Entry::BuildingPlan(plan))) => {
                        plan.can_buy(&garden)
                    }
                    (FarmPage::Workbench, Some(Entry::Design(i))) => {
                        garden.blueprints[i] && !garden.crafted[i] && CATALOG[i].can_build(&garden)
                    }
                    (FarmPage::BlueprintShop, Some(Entry::Design(i))) => {
                        !garden.blueprints[i] && CATALOG[i].can_buy(&garden)
                    }
                    (FarmPage::CardShop, Some(Entry::CombatCard(card))) => {
                        !garden.rpg.knows(card)
                            && progression::card_price(card).is_some_and(|price| balance >= price)
                    }
                    (FarmPage::Talents, Some(Entry::Talent(talent))) => garden
                        .rpg
                        .train_cost(talent)
                        .is_some_and(|cost| garden.rpg.points >= cost),
                    (FarmPage::Gear(source), Some(Entry::Gear(gear))) => {
                        source.ready(&garden, gear)
                    }
                    (FarmPage::Journal(_), Some(Entry::Guide(Guide::Farm))) => garden.task().1,
                    (FarmPage::Journal(_), Some(Entry::Guide(guide))) => {
                        guide.target(&garden).is_some()
                    }
                    (FarmPage::Calendar, _) => true,
                    (FarmPage::SeasonMeeting, _) => seasons::can_contribute(&garden),
                    (FarmPage::Harbor { .. }, _) if !garden.harbor.started => true,
                    (FarmPage::Harbor { at_site: true }, _) => harbor::can_build(&garden),
                    (FarmPage::Harbor { at_site: false }, _) => true,
                    (FarmPage::Workbench, Some(Entry::Fittings)) => {
                        harbor::can_make_fittings(&garden)
                    }
                    _ => false,
                };
                (
                    matches!(
                        page,
                        FarmPage::Workbench
                            | FarmPage::Projects
                            | FarmPage::Tools { .. }
                            | FarmPage::BlueprintShop
                            | FarmPage::CardShop
                            | FarmPage::Talents
                            | FarmPage::Gear(_)
                            | FarmPage::Journal(_)
                            | FarmPage::Calendar
                            | FarmPage::SeasonMeeting
                            | FarmPage::Harbor { .. }
                    ),
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
                _ if let Some(Entry::Season(season)) = entry => match season {
                    Season::Spring => Color::srgb_u8(242, 215, 207),
                    Season::Summer => Color::srgb_u8(247, 230, 181),
                    Season::Autumn => Color::srgb_u8(242, 208, 169),
                    Season::Winter => Color::srgb_u8(215, 233, 228),
                },
                _ => Color::srgb_u8(241, 228, 191),
            }
        } else {
            ui::button_color(interaction, enabled)
        };
    }
}
