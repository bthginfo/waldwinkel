use bevy::prelude::*;

use crate::{
    Adventure, Controls, GardenFocus, Notice,
    adventure::{Area, commit},
    building_plan::Plan,
    cinematic::Clip,
    commissions::Order,
    controller::{Controller, FocusEnabled},
    craft::{Bench, Recipe},
    duel::{Battle, CAMP_INTERVAL, Card, DECK_SIZE, Enemy, Gear, Ration, Style, Talent},
    evening_market,
    furniture::{self, Blueprint, CATALOG},
    garden::{Crop, FERTILIZER_PRICE, Garden, SavePath},
    harbor::{self, Phase},
    homestead::{Product, Village},
    input::Gesture,
    inventory::{self, Material},
    logbook::Path,
    onboarding::Onboarding,
    processing::{Dish, Method, Preparation},
    progression::{self, CARDS_FOR_SALE, GearSource},
    regions::Bell,
    seasons::{self, CONTRIBUTION, MEETING_START, PRIZE},
    session::{Checkpoint, Session},
    story::{Guide, GuideTab},
    ui::{self, UiFonts},
    waywatch::{self, Stage as WatchStage, Watch},
    weather::Season,
    workshop::{BuildingPlan, Project, ToolKind},
    world::Gardener,
};

const PAGE_SIZE: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FarmPage {
    SeedShop,
    EveningMarket,
    Sale,
    MarketSale,
    Workbench,
    Projects,
    Materials(Bench),
    Commissions,
    Watch(Watch),
    WatchFinale,
    Tools { at_forge: bool },
    BlueprintShop,
    Furnish,
    Yard,
    FurniturePreview,
    Kitchen,
    Travel(Bell),
    Inventory,
    Binder,
    Styles,
    Talents,
    CardShop,
    Gear(GearSource),
    Expedition,
    Journal(GuideTab),
    Logbook,
    Calendar,
    Harbor { at_site: bool },
    Cinematic(Clip),
    SeasonMeeting,
    Battle,
}

impl FarmPage {
    fn selling(self) -> bool {
        matches!(self, Self::Sale | Self::MarketSale)
    }

    fn arranging(self) -> bool {
        matches!(self, Self::Furnish | Self::Yard)
    }
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
    batches: u32,
    abandon_confirm: bool,
    battle: Option<Battle>,
    deck_draft: Vec<Card>,
    preview: Option<RoomPreview>,
    history: Vec<ReturnPoint>,
}

#[derive(Clone, Copy)]
struct ReturnPoint {
    page: FarmPage,
    list_page: usize,
    selected: Option<Entry>,
    batches: u32,
}

#[derive(Clone, Copy)]
pub struct RoomPreview {
    pub design: usize,
    pub slot: usize,
    from: FarmPage,
}

impl FarmMenu {
    pub fn room_preview(&self) -> Option<RoomPreview> {
        self.preview
            .filter(|_| self.page == Some(FarmPage::FurniturePreview))
    }
    #[cfg(target_os = "android")]
    pub fn page(&self) -> Option<FarmPage> {
        self.page
    }

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

    pub fn cook_with_bo(&mut self, garden: &Garden, now: f32) {
        self.show(FarmPage::Kitchen);
        self.selected = Some(Entry::Dish(Dish::HarborBroth));
        if Dish::HarborBroth.ready(garden) {
            self.preparation = Some(Preparation::new(Dish::HarborBroth, now));
        } else {
            self.message = format!("Bo hält den Topf warm · {}", Dish::HarborBroth.supplies(garden));
        }
    }

    pub fn show_battle(&mut self, battle: Battle) {
        if battle.training && self.page == Some(FarmPage::Binder) {
            self.open_child(FarmPage::Battle);
        } else {
            self.show(FarmPage::Battle);
        }
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
        self.batches = 1;
        self.abandon_confirm = false;
    }

    fn open_child(&mut self, next: FarmPage) {
        let Some(page) = self.page else {
            self.show(next);
            return;
        };
        if page == next {
            return;
        }
        if let Some(index) = self.history.iter().rposition(|point| point.page == next) {
            self.history.truncate(index + 1);
            self.back();
            return;
        }
        self.history.push(ReturnPoint {
            page,
            list_page: self.list_page,
            selected: self.selected,
            batches: self.batches,
        });
        self.navigate(next);
    }

    pub fn back(&mut self) {
        if self.preparation.take().is_some() {
            self.message.clear();
            return;
        }
        if let Some(view) = self.preview.take() {
            self.page = Some(view.from);
            return;
        }
        self.battle = None;
        if let Some(point) = self.history.pop() {
            self.navigate(point.page);
            self.list_page = point.list_page;
            self.selected = point.selected;
            self.batches = point.batches;
        } else {
            self.close();
        }
    }

    fn copies(&self, card: Card) -> usize {
        self.deck_draft.iter().filter(|&&copy| copy == card).count()
    }

    fn entries(&self, page: FarmPage, garden: &Garden) -> Vec<Entry> {
        match page {
            FarmPage::SeedShop => Crop::ALL.into_iter().map(Entry::Seed)
                .chain([Entry::Material(Material::Fertilizer)]).collect(),
            FarmPage::Sale | FarmPage::MarketSale => Product::stocked(garden).map(Entry::Stock).collect(),
            FarmPage::EveningMarket => garden.fish_supper.offered_today(garden.day).then_some(Entry::Ration(Ration::Broth)).into_iter()
                .chain(Ration::MARKET.into_iter().filter(|_| evening_market::trading(garden)).map(Entry::Ration)).collect(),
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
            FarmPage::Materials(bench) => Recipe::ALL
                .into_iter()
                .filter(|recipe| recipe.sort() == bench)
                .map(Entry::Part)
                .collect(),
            FarmPage::Commissions => Order::offers(garden).map(Entry::Commission).collect(),
            FarmPage::Watch(_) | FarmPage::WatchFinale => Vec::new(),
            FarmPage::Tools { .. } => ToolKind::ALL.into_iter().map(Entry::Tool).collect(),
            FarmPage::Workbench | FarmPage::BlueprintShop | FarmPage::Furnish | FarmPage::Yard => (page
                == FarmPage::BlueprintShop)
                .then_some(BuildingPlan::ALL)
                .into_iter()
                .flatten()
                .map(Entry::BuildingPlan)
                .chain(
                    (page == FarmPage::Workbench && garden.harbor.phase == Phase::Landing)
                        .then_some(Entry::Fittings),
                )
                .chain(crate::decorate::Building::ALL.into_iter()
                    .filter(|building| page == FarmPage::Yard && building.built(garden))
                    .map(Entry::Building))
                .chain(
                    CATALOG
                        .iter()
                        .enumerate()
                        .filter(|&(index, design)| match page {
                            FarmPage::BlueprintShop => {
                                matches!(design.blueprint, Blueprint::Shop { .. })
                            }
                            FarmPage::Furnish | FarmPage::Yard => garden.crafted[index],
                            _ => garden.blueprints[index],
                        })
                        .map(|(index, _)| Entry::Design(index)),
                )
                .collect(),
            FarmPage::Kitchen => self.preparation.as_ref().map_or_else(
                || garden.fish_supper.learned().then_some(Entry::Dish(Dish::HarborBroth)).into_iter()
                    .chain(Dish::ALL.into_iter().filter(|dish| *dish != Dish::HarborBroth).map(Entry::Dish)).collect(),
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
            FarmPage::Styles => Style::ALL.into_iter().map(Entry::Style).collect(),
            FarmPage::CardShop => CARDS_FOR_SALE.into_iter().map(Entry::CombatCard).collect(),
            FarmPage::Gear(_) => Gear::ALL.into_iter().map(Entry::Gear).collect(),
            FarmPage::Expedition => std::iter::once(1)
                .chain((1..=garden.rpg.camp() / CAMP_INTERVAL).map(|index| index * CAMP_INTERVAL))
                .map(Entry::Level)
                .collect(),
            FarmPage::Journal(tab) => Guide::entries(tab)
                .filter(|entry| !matches!(entry, Guide::Resident(npc) if !npc.available(garden)))
                .filter(|entry| *entry != Guide::FirstMorning || garden.prologue.is_none())
                .filter(|entry| *entry != Guide::BuildPlan || garden.plan.is_some())
                .filter(|entry| *entry != Guide::FishSupper || garden.fish_supper.stage != crate::fish_supper::Stage::Unseen)
                .filter(|entry| *entry != Guide::EveningMarket || evening_market::invited(garden))
                .map(Entry::Guide)
                .collect(),
            FarmPage::Calendar => Season::ALL.into_iter().map(Entry::Season).collect(),
            FarmPage::Logbook => Path::ALL.into_iter().map(Entry::Practice).collect(),
            FarmPage::SeasonMeeting | FarmPage::Harbor { .. } | FarmPage::Cinematic(_) => {
                Vec::new()
            }
            FarmPage::Battle | FarmPage::FurniturePreview => Vec::new(),
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
    Part(Recipe),
    Commission(Order),
    Tool(ToolKind),
    BuildingPlan(BuildingPlan),
    Design(usize),
    Building(crate::decorate::Building),
    Dish(Dish),
    Ration(Ration),
    Step(&'static str),
    Practice(Path),
    Destination(Bell),
    CombatCard(Card),
    Style(Style),
    Talent(Talent),
    Gear(Gear),
    Level(u32),
    Guide(Guide),
    Season(Season),
}

impl Entry {
    fn plan(self, page: FarmPage, garden: &Garden, batches: u32) -> Option<Plan> {
        match (page, self) {
            (FarmPage::Workbench | FarmPage::BlueprintShop, Self::Design(index)) => {
                Some(Plan::Furniture(index))
            }
            (FarmPage::Projects, Self::Project(project)) => Some(Plan::Project(project)),
            (FarmPage::Tools { .. }, Self::Tool(tool)) if tool.level(garden) < 2 => {
                Some(Plan::Tool {
                    tool,
                    tier: tool.level(garden) + 1,
                })
            }
            (FarmPage::Materials(_), Self::Part(recipe)) => {
                let stock = garden.stock(Product::Crafted(recipe));
                if let Some(
                    plan @ Plan::Material {
                        recipe: pinned,
                        batches: count,
                        quantity,
                    },
                ) = garden.plan
                    && pinned == recipe
                    && u32::from(count) == batches
                    && stock < quantity
                {
                    return Some(plan);
                }
                let quantity = stock.checked_add(recipe.yield_count() * batches)?;
                Some(Plan::Material {
                    recipe,
                    batches: batches as u8,
                    quantity,
                })
            }
            (FarmPage::Journal(_), Self::Guide(Guide::BuildPlan)) => garden.plan,
            _ => None,
        }
    }

    fn title(self, garden: &Garden) -> String {
        match self {
            Self::Building(building) => building.name().into(),
            Self::Seed(crop) => crop.name().into(),
            Self::Stock(item) => item.label(),
            Self::Material(item) => item.name(),
            Self::Fittings => "Kupferbeschläge".into(),
            Self::Project(project) => project.name().into(),
            Self::Part(recipe) => recipe.name().into(),
            Self::Commission(order) => order.name().into(),
            Self::Tool(tool) => format!("{} · {}", tool.name(), tool.tier(garden)),
            Self::BuildingPlan(plan) => plan.name().into(),
            Self::Design(i) => CATALOG[i].name.into(),
            Self::Dish(dish) => dish.name().into(),
            Self::Ration(ration) => ration.dish().name().into(),
            Self::Step(text) => text.into(),
            Self::Practice(path) => format!("{} · Rang {}", path.title(), path.rank(garden)),
            Self::Destination(bell) => bell.name().into(),
            Self::CombatCard(card) => card.name().into(),
            Self::Style(style) => style.name().into(),
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

    fn detail(self, page: FarmPage, garden: &Garden, batches: u32) -> String {
        match self {
            Self::Building(_) => "Im Hofraster verschieben und drehen".into(),
            Self::Material(Material::Fertilizer) if page == FarmPage::SeedShop =>
                format!("2 Portionen · {FERTILIZER_PRICE} Münzen · {} im Vorrat", garden.fertilizer),
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
            Self::Part(recipe) => recipe.describe(garden, batches),
            Self::Commission(order) => order.detail(garden),
            Self::Tool(tool) => {
                tool.requirements(garden, page == FarmPage::Tools { at_forge: true })
            }
            Self::BuildingPlan(plan) => plan.detail(garden),
            Self::Design(i) if page.arranging() => {
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
            Self::Ration(ration) if evening_market::stocked(garden, ration) => format!(
                "{} Münzen · {}",
                evening_market::price(ration),
                ration.effect()
            ),
            Self::Ration(_) => "Heute verkauft".into(),
            Self::Step(_) => String::new(),
            Self::Practice(path) => path.detail(garden),
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
            Self::Style(style) => style.detail().into(),
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

    fn enabled(self, _page: FarmPage, garden: &Garden) -> bool {
        match self {
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
    Purchase,
    MarketBuy,
    MarketSell,
    Work,
    AddCopy,
    RemoveCopy,
    SaveDeck,
    Training,
    PinPlan,
    PreviewFurniture,
    PlaceFurniture,
    Arrange,
    PreviewSlot(i8),
    Styles,
    Talents,
    Gear,
    Projects,
    Tools,
    Recipes,
    Materials,
    Batches,
    Commissions,
    Abandon,
    Return,
    Guide(GuideTab),
    Logbook,
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
    PinPlan,
    Batches,
    StationProgress,
}

#[derive(Component)]
pub struct ItemPicture(usize);

#[derive(Component)]
pub enum HeatMeter {
    Track,
    Target,
    Needle,
}

#[derive(Component)]
pub struct StationArt;

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>) {
    commands
        .spawn((
            Pane,
            crate::controller::Panel::Farm,
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
                paper.spawn((
                    StationArt, ImageNode::default(),
                    Node {
                        display: Display::None,
                        align_self: AlignSelf::Center,
                        width: Val::Px(146.), height: Val::Px(89.),
                        ..default()
                    },
                    ui::shadow(),
                ));
                paper.spawn((Field::StationProgress, fonts.bold("", 13.)));
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
                        track.spawn((
                            HeatMeter::Target,
                            Node {
                                position_type: PositionType::Absolute,
                                height: Val::Percent(100.),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(4.)),
                            BackgroundColor(Color::srgb_u8(112, 156, 106)),
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
                                    FocusEnabled(true),
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
                            (Choice::MarketBuy, "Einkaufen", 102.),
                            (Choice::MarketSell, "Verkaufen", 103.),
                            (Choice::Purchase, "Auswahl kaufen", 144.),
                            (Choice::Work, "Herstellen", 164.),
                            (Choice::PinPlan, "Anheften", 127.),
                            (Choice::PreviewFurniture, "Im Haus ansehen", 155.),
                            (Choice::PlaceFurniture, "Platzieren", 113.),
                            (Choice::Arrange, "Hof einrichten", 135.),
                            (Choice::PreviewSlot(-1), "← Platz", 105.),
                            (Choice::PreviewSlot(1), "Platz →", 105.),
                            (Choice::AddCopy, "+ Karte", 84.),
                            (Choice::RemoveCopy, "− Karte", 84.),
                            (Choice::SaveDeck, "Deck speichern", 129.),
                            (Choice::Training, "Üben", 84.),
                            (Choice::Styles, "Kampfstile", 105.),
                            (Choice::Talents, "Talente", 84.),
                            (Choice::Gear, "Ausrüstung", 106.),
                            (Choice::Projects, "Bauprojekte", 110.),
                            (Choice::Tools, "Werkzeuge", 106.),
                            (Choice::Recipes, "Möbel", 84.),
                            (Choice::Materials, "Materialien", 108.),
                            (Choice::Batches, "Menge ×1", 112.),
                            (Choice::Commissions, "Hafenaufträge", 126.),
                            (Choice::Abandon, "Aufgeben", 100.),
                            (Choice::Return, "Zurück", 84.),
                            (Choice::Guide(GuideTab::Orders), "Aufträge", 104.),
                            (Choice::Guide(GuideTab::Story), "Geschichte", 112.),
                            (Choice::Guide(GuideTab::Town), "Dorf", 76.),
                            (Choice::Guide(GuideTab::Watches), "Wegwachten", 117.),
                            (Choice::Calendar, "Kalender", 104.),
                            (Choice::Logbook, "Hofbuch", 96.),
                        ] {
                            footer
                                .spawn((action, ui::dialog_button(width)))
                                .with_children(|button| {
                                    let mut label = button.spawn(fonts.button(caption, 14.));
                                    if action == Choice::Work {
                                        label.insert(Field::Work);
                                    } else if action == Choice::PinPlan {
                                        label.insert(Field::PinPlan);
                                    } else if action == Choice::Batches {
                                        label.insert(Field::Batches);
                                    }
                                });
                        }
                    });
            });
        });
}

pub fn input(
    session: Res<Session>,
    controller: Res<Controller>,
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
    mut sounds: EventWriter<crate::sound::Cue>,
    mut decor: ResMut<crate::decorate::Layout>,
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
            if menu.preparation.is_some() || menu.clip().is_some() {
                return None;
            }
            match controller.page_step {
                -1 if menu.list_page > 0 => Some(Choice::Back),
                1 if menu.list_page < menu.last_page(page, &garden) => Some(Choice::Next),
                _ => None,
            }
        })
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
        Choice::PinPlan => {
            let Some(plan) = menu
                .selected
                .and_then(|entry| entry.plan(page, &garden, menu.batches.max(1)))
            else {
                return;
            };
            if garden.plan == Some(plan) {
                garden.plan = None;
                if adventure.tracked == Some(Guide::BuildPlan) {
                    adventure.tracked = None;
                }
                Some(Ok("Bauplan gelöst".into()))
            } else if !plan.finished(&garden) {
                garden.plan = Some(plan);
                adventure.tracked = Some(Guide::BuildPlan);
                Some(Ok(format!(
                    "{} angeheftet · Journal → Aufträge",
                    plan.title()
                )))
            } else {
                None
            }
        }
        Choice::PreviewFurniture => {
            if let Some(Entry::Design(design)) = menu.selected
                && matches!(
                    page,
                    FarmPage::Workbench | FarmPage::BlueprintShop | FarmPage::Furnish
                )
            {
                let slots = garden.workshop.slots();
                let slot = garden.furniture[..slots]
                    .iter()
                    .position(|&item| item == 0)
                    .unwrap_or(0);
                menu.preview = Some(RoomPreview {
                    design,
                    slot,
                    from: page,
                });
                menu.page = Some(FarmPage::FurniturePreview);
                menu.message.clear();
                player.0.stop();
            }
            None
        }
        Choice::PreviewSlot(step) if page == FarmPage::FurniturePreview => {
            if let Some(preview) = &mut menu.preview {
                preview.slot = (preview.slot as isize + step as isize)
                    .rem_euclid(garden.workshop.slots() as isize)
                    as usize;
            }
            None
        }
        Choice::Close if page == FarmPage::FurniturePreview => {
            if let Some(preview) = menu.preview.take() {
                menu.page = Some(preview.from);
            }
            None
        }
        Choice::Close => {
            menu.close();
            *gesture = Gesture::default();
            *controls = Controls::default();
            if page == FarmPage::Cinematic(Clip::Arrival)
                && garden.onboarding == Onboarding::Welcome
            {
                garden.onboarding = Onboarding::Till;
                Some(Ok("Prolog gestartet · Dein erster Morgen".into()))
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
                Some(Entry::Design(i)) if page.arranging() => {
                    if decor.place(adventure.area, i, player.0.position, &garden) {
                        player.0.stop();
                        menu.close();
                        *gesture = Gesture::default();
                        *controls = Controls::default();
                        None
                    } else {
                        Some(Err("Dieses Stück steht in deinem anderen Bereich. Lagere es zuerst ein.".into()))
                    }
                }
                Some(Entry::Building(building)) if page == FarmPage::Yard => {
                    decor.move_building(adventure.area, building, &garden);
                    player.0.stop();
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
                    let phase = game.progress().0;
                    if game.try_step(index, visible_time) {
                        let cue = match game.dish.method() {
                            Method::Cook if phase == 0 => crate::sound::Cue::KitchenChop,
                            Method::Cook => crate::sound::Cue::KitchenStir,
                            Method::Preserve => crate::sound::Cue::KitchenJar,
                            Method::Mill => crate::sound::Cue::KitchenMill,
                        };
                        sounds.write(cue);
                    }
                    let first_supper = game.dish == Dish::HarborBroth && garden.fish_supper.stage == crate::fish_supper::Stage::Cooking;
                    if let Some(servings) = game.finish(&mut garden) {
                        let dish = game.dish;
                        menu.preparation = None;
                        sounds.write(crate::sound::Cue::KitchenDone);
                        let message = if first_supper {
                            let when = if garden.fish_supper.offered_today(garden.day) { "heute" } else { "morgen" };
                            format!("+{servings} Hafenbrühe · Bo serviert sie {when} ab 18 Uhr am Brunnen.")
                        } else {
                            format!("+{servings} {}", dish.name())
                        };
                        Some(Ok(message))
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
        Choice::Arrange => {
            menu.open_child(FarmPage::Yard);
            None
        }
        Choice::PlaceFurniture if page == FarmPage::Workbench => {
            let Some(Entry::Design(i)) = menu.selected else { return };
            if garden.crafted[i] && decor.place(adventure.area, i, player.0.position, &garden) {
                player.0.stop();
                menu.close();
                *gesture = Gesture::default();
                *controls = Controls::default();
                None
            } else {
                Some(Err("Stelle das Stück zuerst her oder lagere es im Haus ein.".into()))
            }
        }
        Choice::Purchase if page == FarmPage::SeedShop => match menu.selected {
            Some(Entry::Seed(crop)) => Some(Village::Seeds.work(&mut garden, crop, None)),
            Some(Entry::Material(Material::Fertilizer)) => Some(if garden.buy_fertilizer() {
                Ok("+2 Dünger".into())
            } else {
                Err("Für das Düngerpaket brauchst du 4 Münzen.".into())
            }),
            _ => None,
        },
        Choice::MarketBuy if page == FarmPage::MarketSale => {
            menu.show(FarmPage::SeedShop);
            None
        }
        Choice::MarketSell if page == FarmPage::SeedShop => {
            menu.show(FarmPage::MarketSale);
            None
        }
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
        Choice::Guide(tab)
            if matches!(
                page,
                FarmPage::Journal(_) | FarmPage::Calendar | FarmPage::Commissions | FarmPage::Logbook
            ) =>
        {
            if page != FarmPage::Journal(tab) && !matches!(page, FarmPage::Journal(_))
                && !menu.history.is_empty()
            {
                menu.back();
            }
            menu.navigate(FarmPage::Journal(tab));
            None
        }
        Choice::Commissions if page == FarmPage::Journal(GuideTab::Orders) => {
            menu.open_child(FarmPage::Commissions);
            None
        }
        Choice::Batches if matches!(page, FarmPage::Materials(_)) => {
            menu.batches = match menu.batches {
                0 | 1 => 3,
                3 => 5,
                _ => 1,
            };
            None
        }
        Choice::Materials
            if matches!(
                page,
                FarmPage::Workbench | FarmPage::Tools { at_forge: true }
            ) =>
        {
            let station = if page == FarmPage::Workbench {
                Bench::Work
            } else {
                Bench::Forge
            };
            menu.open_child(FarmPage::Materials(station));
            None
        }
        Choice::Abandon if page == FarmPage::Commissions => {
            if menu.abandon_confirm && Order::abandon(&mut garden) {
                menu.abandon_confirm = false;
                menu.selected = None;
                Some(Ok("Auftrag aufgegeben.".into()))
            } else {
                menu.abandon_confirm = true;
                Some(Err(
                    "Drücke noch einmal auf Aufgeben, um einen neuen Auftrag zu wählen.".into(),
                ))
            }
        }
        Choice::Calendar if matches!(page, FarmPage::Journal(_) | FarmPage::SeasonMeeting) => {
            menu.open_child(FarmPage::Calendar);
            None
        }
        Choice::Logbook if matches!(page, FarmPage::Journal(_) | FarmPage::Calendar) => {
            menu.open_child(FarmPage::Logbook);
            None
        }
        Choice::Talents if page == FarmPage::Binder => {
            menu.open_child(FarmPage::Talents);
            None
        }
        Choice::Training if page == FarmPage::Binder && adventure.area == Area::Home => {
            menu.show_battle(Battle::training());
            None
        }
        Choice::Styles if page == FarmPage::Binder => {
            menu.open_child(FarmPage::Styles);
            None
        }
        Choice::Projects
            if page == FarmPage::Workbench || page == (FarmPage::Tools { at_forge: false }) =>
        {
            menu.open_child(FarmPage::Projects);
            None
        }
        Choice::Tools
            if matches!(page, FarmPage::Workbench | FarmPage::Projects)
                && garden.workshop.restored =>
        {
            menu.open_child(FarmPage::Tools { at_forge: false });
            None
        }
        Choice::Recipes
            if matches!(
                page,
                FarmPage::Projects | FarmPage::Tools { at_forge: false }
            ) && garden.workshop.restored =>
        {
            menu.open_child(FarmPage::Workbench);
            None
        }
        Choice::Gear if page == FarmPage::Workbench || page == FarmPage::CardShop => {
            let source = if page == FarmPage::Workbench {
                GearSource::Workbench
            } else {
                GearSource::Smith
            };
            menu.open_child(FarmPage::Gear(source));
            None
        }
        Choice::Return => {
            menu.back();
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
        Choice::Work if page == FarmPage::Styles && adventure.area == Area::Home => {
            if let Some(Entry::Style(style)) = menu.selected
                && styles_unlocked(&garden)
            {
                garden.rpg.style = style;
                Some(Ok(format!("Kampfstil gewählt · {}", style.name())))
            } else {
                None
            }
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
        Choice::Work if page == FarmPage::EveningMarket => {
            match menu.selected {
                Some(Entry::Ration(ration)) => Some(evening_market::buy(&mut garden, ration)),
                _ => None,
            }
        }

        Choice::Work if matches!(page, FarmPage::Gear(_)) => {
            let (FarmPage::Gear(source), Some(Entry::Gear(gear))) = (page, menu.selected) else {
                return;
            };
            Some(source.equip(&mut garden, gear))
        }
        Choice::Work if matches!(page, FarmPage::Materials(_)) => {
            let (FarmPage::Materials(bench), Some(Entry::Part(recipe))) = (page, menu.selected)
            else {
                return;
            };
            Some(recipe.make(&mut garden, bench, menu.batches.max(1)))
        }
        Choice::Work if page == FarmPage::Commissions => {
            let Some(Entry::Commission(order)) = menu.selected else {
                return;
            };
            let result = order.work(&mut garden);
            if result.is_ok() {
                menu.selected = None;
                menu.abandon_confirm = false;
            }
            Some(result)
        }
        Choice::Work if page == FarmPage::Logbook => {
            let Some(Entry::Practice(path)) = menu.selected else { return };
            Some(path.claim(&mut garden))
        }
        Choice::Work if matches!(page, FarmPage::Watch(_)) => {
            let FarmPage::Watch(site) = page else {
                unreachable!()
            };
            if !site.ready(&garden) {
                return;
            }
            match site.stage(&garden) {
                WatchStage::Unheard => {
                    site.begin(&mut garden);
                    adventure.tracked = Some(Guide::for_watch(site, &garden));
                    Some(Ok(
                        "Spur aufgenommen · Das Wegzeichen steht im Journal.".into()
                    ))
                }
                WatchStage::Surveying => {
                    adventure.tracked = Some(Guide::for_watch(site, &garden));
                    adventure.map = true;
                    player.0.stop();
                    menu.close();
                    None
                }
                WatchStage::Surveyed => Some(site.restore(&mut garden)),
                WatchStage::Guarding | WatchStage::Complete => {
                    if let Some(trial) = site.trial(&garden) {
                        menu.show_battle(Battle::new_trial(trial, &garden.rpg));
                    }
                    None
                }
            }
        }
        Choice::Work if page == FarmPage::WatchFinale => {
            if garden.watches.lights() == Watch::ALL.len() {
                let result = waywatch::conclude(&mut garden);
                if result.is_ok() {
                    menu.show(FarmPage::Cinematic(Clip::Watches));
                    adventure.tracked = None;
                    player.0.stop();
                }
                Some(result)
            } else {
                adventure.tracked = Some(Guide::WatchFinale);
                adventure.map = true;
                player.0.stop();
                menu.close();
                None
            }
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
                    sounds.write(crate::sound::Cue::Craft);
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
        Choice::SellOne | Choice::SellAll if page.selling() => {
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
            use crate::sound::Cue;
            match page {
                FarmPage::Talents | FarmPage::Styles | FarmPage::CardShop => {
                    sounds.write(Cue::Skill);
                }
                FarmPage::Workbench
                | FarmPage::Kitchen
                | FarmPage::Materials(_)
                | FarmPage::Projects
                | FarmPage::Tools { .. } => {
                    sounds.write(Cue::Craft);
                }
                _ => {}
            }
            commit(&garden, &path, &mut notice, now, message, None, None);
            menu.message.clone_from(&notice.message);
        }
        Some(Err(message)) => {
            *notice = crate::feedback::make(crate::feedback::Tone::Problem, message.clone(), now);
            menu.message = message;
        }
        None => (),
    }
}

pub fn update(
    session: Res<Session>,
    controller: Res<Controller>,
    mut menu: ResMut<FarmMenu>,
    adventure: Res<Adventure>,
    garden: Res<Garden>,
    notice: Res<Notice>,
    time: Res<Time>,
    assets: Res<AssetServer>,
    mut veil: Single<(&mut Node, &mut BackgroundColor), With<Pane>>,
    mut fields: Query<(&Field, &mut Text)>,
    mut choices: Query<(&Choice, &Interaction, &mut Node, &mut BackgroundColor, &mut FocusEnabled), Without<Pane>>,
    mut meter: Query<(&HeatMeter, &mut Node), (Without<Pane>, Without<Choice>)>,
    mut pictures: Query<
        (&ItemPicture, &mut ImageNode, &mut Node),
        (Without<Pane>, Without<Choice>, Without<HeatMeter>),
    >,
    mut station: Single<(&mut ImageNode, &mut Node), (
        With<StationArt>, Without<Pane>, Without<Choice>, Without<ItemPicture>, Without<HeatMeter>,
    )>,
) {
    let (veil, shade) = &mut *veil;
    let Some(page) = menu.page.filter(|&page| {
        session.playing() && !matches!(page, FarmPage::Battle | FarmPage::Cinematic(_))
    }) else {
        veil.display = Display::None;
        return;
    };
    veil.display = Display::Flex;
    let preview = menu.room_preview();
    veil.align_items = if preview.is_some() {
        AlignItems::End
    } else {
        AlignItems::Center
    };
    veil.padding.bottom = Val::Px(if preview.is_some() { 13. } else { 0. });
    shade.0 = Color::srgba(
        0.08,
        0.19,
        0.18,
        if preview.is_some() { 0.04 } else { 0.47 },
    );
    menu.visible_time = time.elapsed_secs();

    let entries = menu.entries(page, &garden);
    let pages = entries.len().saturating_sub(1) / PAGE_SIZE;
    let entry_at = |i| entries.get(menu.list_page * PAGE_SIZE + i).copied();
    let balance = garden.farm.coins;
    let batches = menu.batches.max(1);
    let selected = menu.selected;
    let plan = selected.and_then(|entry| entry.plan(page, &garden, batches));
    let progress = menu.preparation.as_ref();
    let (illustration, frame) = &mut *station;
    if let Some(game) = progress {
        frame.display = Display::Flex;
        let pulse = game.pulse(menu.visible_time);
        frame.width = Val::Px(146. + 13.*pulse);
        illustration.color = Color::srgb(1., 1.-0.06*pulse, 1.-0.13*pulse);
        illustration.image = assets.load(game.illustration());
    } else {
        frame.display = Display::None;
    }
    for (picture, mut image, mut node) in &mut pictures {
        let entry = entry_at(picture.0);
        let slug = match entry {
            Some(Entry::Stock(item)) => Some(inventory::product_icon(item)),
            Some(Entry::Dish(dish)) => Some(inventory::product_icon(Product::Prepared(dish))),
            Some(Entry::Ration(ration)) => Some(inventory::product_icon(Product::Prepared(ration.dish()))),
            Some(Entry::Material(item)) => Some(item.icon()),
            Some(Entry::Seed(crop)) => Some(Material::Seed(crop).icon()),
            Some(Entry::Fittings) => Some("fittings"),
            Some(Entry::Part(recipe)) => Some(recipe.icon()),
            Some(Entry::BuildingPlan(BuildingPlan::Barn(_))) => Some("plan_barn"),
            Some(Entry::BuildingPlan(BuildingPlan::Room(_)) | Entry::Project(Project::Room(_))) => {
                Some("plan_room")
            }
            Some(Entry::Design(i)) => Some(CATALOG[i].model),
            Some(Entry::Building(building)) => Some(building.model()),
            Some(Entry::Project(_)) => Some("plan_furniture"),
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
        let art = match entry {
            Some(Entry::CombatCard(card)) => Some(card.portrait()),
            _ => slug.map(|slug| format!("ui/items/{slug}.png")),
        };
        node.display = if art.is_some() { Display::Flex } else { Display::None };
        let wide = matches!(entry, Some(Entry::CombatCard(_) | Entry::Dish(_)));
        node.width = Val::Px(if wide { 71. } else { 43. });
        if let Some(art) = art {
            let next = assets.load(art);
            if image.image != next {
                image.image = next;
            }
        }
    }
    let range = progress.and_then(Preparation::timing_range);
    let heat = progress.map(|game| game.heat(menu.visible_time));
    for (part, mut node) in &mut meter {
        match part {
            HeatMeter::Track => {
                node.display = if range.is_some() {
                    Display::Flex
                } else {
                    Display::None
                }
            }
            HeatMeter::Target => if let Some(range) = &range {
                node.left = Val::Percent(100.*range.start());
                node.width = Val::Percent(100.*(range.end()-range.start()));
            },
            HeatMeter::Needle => node.left = Val::Percent(100. * heat.unwrap_or(0.)),
        }
    }

    for (field, mut text) in &mut fields {
        let mut value = match *field {
            Field::StationProgress => progress.map_or_else(String::new, |game| {
                let (passed, total) = game.progress();
                format!("{} · {passed} von {total} erledigt", game.dish.method().title())
            }),
            Field::Title if progress.is_some() => progress.unwrap().dish.name().into(),
            Field::Title => match page {
                FarmPage::SeedShop => "Saatgut und Dünger",
                FarmPage::EveningMarket => "Abendmarkt am Brunnen",
                FarmPage::Sale => "Verkaufstruhe",
                FarmPage::MarketSale => "Markt · Waren verkaufen",
                FarmPage::Workbench => "Hofwerkbank · Möbel",
                FarmPage::Materials(bench) => bench.name(),
                FarmPage::Commissions => "Hafenaufträge",
                FarmPage::Watch(site) => site.name(),
                FarmPage::WatchFinale => "Junas Inselkarte",
                FarmPage::Projects if !garden.workshop.restored => "Die alte Hofwerkbank",
                FarmPage::Projects => "Hofwerkbank · Bauprojekte",
                FarmPage::Tools { at_forge: false } => "Hofwerkbank · Werkzeuge",
                FarmPage::Tools { at_forge: true } => "Schmiedetisch",
                FarmPage::BlueprintShop => "Bauplanstand",
                FarmPage::Furnish => "Deine Möbel",
                FarmPage::Yard => "Hof einrichten",
                FarmPage::FurniturePreview => preview.map_or("Möbelvorschau", |view| CATALOG[view.design].name),
                FarmPage::Kitchen => "Hofküche",
                FarmPage::Travel(_) => "Wegglocken",
                FarmPage::Inventory => "Deine Tasche",
                FarmPage::Binder => "Kartenbinder",
                FarmPage::Styles => "Kampfstile",
                FarmPage::Talents => "Kampftalente",
                FarmPage::CardShop => "Runas Karten",
                FarmPage::Gear(_) => "Ausrüstung",
                FarmPage::Expedition => "Dämmergänge",
                FarmPage::Journal(GuideTab::Story) => "Das Licht der Wegglocken",
                FarmPage::Journal(GuideTab::Orders) => "Aufträge",
                FarmPage::Journal(GuideTab::Town) => "Bewohner der Insel",
                FarmPage::Journal(GuideTab::Watches) => "Die fünf Wegwachten",
                FarmPage::Harbor { .. } => "Der alte Fischersteg",
                FarmPage::Cinematic(_) => "",
                FarmPage::Calendar => "Jahreszeitenkalender",
                FarmPage::Logbook => "Dein Hofbuch",
                FarmPage::SeasonMeeting => Season::at(garden.day).meeting(),
                FarmPage::Battle => "",
            }
            .into(),
            Field::Instruction if progress.is_some_and(|p| p.dish == Dish::HarborBroth) && garden.fish_supper.stage == crate::fish_supper::Stage::Cooking => {
                match progress.unwrap().progress().0 {
                    0 => "Bo reicht dir das Brett. „Die Möhre klein schneiden, dann den Fisch in den Topf.“",
                    1..=3 => "Bo hält die Kelle. „Jetzt behutsam rühren, einmal pro Runde im grünen Bereich.“",
                    _ => "Bo stellt zwei Schalen hin. „Schöpf die Brühe ab. So soll es auch am Hafen duften.“",
                }.into()
            }
            Field::Instruction if progress.is_some() => progress.unwrap().instructions(),
            Field::Instruction => match page {
                FarmPage::SeedShop => {
                    format!("Du hast {balance} Münzen und {} Dünger.", garden.fertilizer)
                }
                FarmPage::Sale | FarmPage::MarketSale if pages > 0 => {
                    format!("{} / {} · {balance} Münzen", menu.list_page + 1, pages + 1)
                }
                FarmPage::Sale | FarmPage::MarketSale => format!("{balance} Münzen · Erlös sofort verfügbar"),
                FarmPage::Materials(_) => "Wähle ein Material und die Anzahl der Arbeitsgänge.".into(),
                FarmPage::Commissions if garden.ledger.active().is_some() => format!(
                    "Aktiv · {} · {} erledigt", garden.ledger.active().unwrap().name(),garden.ledger.count()
                ),
                FarmPage::Commissions => format!(
                    "Neue Aushänge alle drei Tage · {} erledigt",garden.ledger.count()
                ),
                FarmPage::Watch(site) => site.note(&garden),
                FarmPage::WatchFinale if garden.watches.gathered => "Die Karte ist vollständig. Jede Wacht leuchtet wieder.".into(),
                FarmPage::WatchFinale => format!(
                    "{}/5 Lichter · Juna will die Geschichten der fünf Wachten auf ihrer Inselkarte festhalten.",
                    garden.watches.lights()
                ),
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
                FarmPage::Furnish | FarmPage::Yard => {
                    "Wähle ein Stück. In der Draufsicht kannst du es bewegen und drehen.".into()
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
                FarmPage::Styles if !styles_unlocked(&garden) => {
                    "Das erste Duell an der Farnwacht schaltet die Kampfstile frei.".into()
                }
                FarmPage::Styles => format!(
                    "Aktiv · {}. Wechsle deinen Stil hier am Kartenbinder.",
                    garden.rpg.style.name()
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
                FarmPage::Journal(GuideTab::Story) => crate::campaign::next(&garden).chapter().into(),
                FarmPage::Journal(GuideTab::Orders) => {
                    "Hof, Eichenhafen und die Außenregionen".into()
                }
                FarmPage::Journal(GuideTab::Watches) => format!(
                    "{}/5 Lichter · Wähle eine Wacht und markiere den nächsten Schritt.",garden.watches.lights()
                ),
                FarmPage::Journal(GuideTab::Town) => {
                    "Wähle eine Person oder den Fischersteg für die Karte.".into()
                }
                FarmPage::Harbor { .. } => garden.harbor.phase.label().into(),
                FarmPage::Cinematic(_) => String::new(),
                FarmPage::Logbook => "Halte fest, was du auf der Insel ausprobierst. Wähle einen Bereich, um sein nächstes Ziel zu sehen.".into(),
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
                FarmPage::FurniturePreview => preview.map_or_else(String::new, |view| {
                    let slot = view.slot;
                    let room = match slot {
                        0..6 => "Wohnstube",
                        6..10 => "Gartenstube",
                        _ => "Lesekammer",
                    };
                    let occupancy = garden.furniture[slot].checked_sub(1).and_then(|i| CATALOG.get(i as usize))
                        .map_or_else(|| "Freier Platz".into(), |item| format!("Aktuell steht hier {}", item.name));
                    format!("{room} · Platz {} · {occupancy}", slot + 1)
                }),
                FarmPage::EveningMarket => format!("{balance} Münzen · Abendküche am Brunnen · 18:00–21:30"),
                FarmPage::Battle => String::new(),
            },
            Field::Message if progress.is_some() => progress.unwrap().message.into(),
            Field::Message if notice.toast.is_some() && notice.until > menu.visible_time
                && notice.message == menu.message => String::new(),
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
            Field::Message if page == FarmPage::Logbook => match selected {
                Some(Entry::Practice(path)) if path.rank(&garden) < 3 =>
                    format!("Belohnung für Rang {} · {}", path.rank(&garden)+1, path.reward(usize::from(path.rank(&garden)))),
                _ => String::new(),
            },
            Field::Message if page == FarmPage::SeasonMeeting => {
                let season = Season::at(garden.day);
                let supplies = garden.stock(Product::Seasonal(season));
                format!(
                    "{}\nBeitrag {supplies}/{CONTRIBUTION} {} · Belohnung {PRIZE} Münzen und 3 Samen.\nDas Jahreszeitenregal gibt es nach allen vier Treffen.",
                    season.invitation(),
                    season.find_name()
                )
            }
            Field::Message if matches!(page, FarmPage::Watch(site) if site.trial(&garden).is_some()) => {
                let FarmPage::Watch(site) = page else { unreachable!() };
                site.trial(&garden).map_or_else(String::new, |trial| trial.enemy().provision_hint())
            }
            Field::Message if preview.is_some() => "Vorschau im Haus · Wähle einen Stellplatz.".into(),
            Field::Message if plan.is_some() => plan.unwrap().detail(&garden),
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
                    Entry::Design(i).detail(page, &garden, batches)
                ),
                Entry::CombatCard(card) if page == FarmPage::Binder => {
                    format!(
                        "{} · {} von 2 möglichen Kopien im Deck",
                        card.name(),
                        menu.copies(card)
                    )
                }
                Entry::Dish(dish) => Ration::ALL.into_iter().find(|ration| ration.dish() == dish)
                    .map_or(String::new(), |ration| ration.best_against().into()),
                Entry::Ration(ration) => ration.best_against().into(),
                Entry::Level(1) => Enemy::at(1).provision_hint(),
                Entry::Level(level) if level < crate::duel::LAST_LEVEL => {
                    format!("Nächster Kampf · {}", Enemy::at(level + 1).provision_hint())
                }
                Entry::Talent(talent) => match garden.rpg.train_cost(talent) {
                    Some(cost) => format!("Nächster Rang · {cost} Talentpunkte"),
                    None => "Höchster Rang erreicht".into(),
                },
                _ => String::new(),
            },
            Field::Message if page.selling() && entries.is_empty() => {
                "Ernte oder sammle etwas, um es hier zu verkaufen.".into()
            }
            Field::Message if page.selling() => {
                "Wähle eine Ware. Du kannst ein Stück oder den Stapel verkaufen.".into()
            }
            Field::Message if page == FarmPage::SeedShop =>
                "Wähle einen Artikel. Preis und Bestand stehen am Bild.".into(),
            Field::Message if page == FarmPage::Binder => {
                "Wähle eine Karte. Schließen verwirft ungespeicherte Änderungen.".into()
            }
            Field::Message => String::new(),
            Field::CardName(i) => {
                entry_at(i).map_or_else(String::new, |entry| entry.title(&garden))
            }
            Field::CardDetail(i) => entry_at(i).map_or_else(String::new, |entry| {
                let extra = entry.detail(page, &garden, batches);
                match entry {
                    Entry::CombatCard(card) if page == FarmPage::Binder => {
                        format!("{} im Deck · {extra}", menu.copies(card))
                    }
                    _ => extra,
                }
            }),
            Field::Batches => format!("Menge ×{batches}"),
            Field::PinPlan => if plan.is_some() && plan == garden.plan { "Lösen" } else { "Anheften" }.into(),
            Field::Work => match (page, selected) {
                (FarmPage::Commissions, Some(Entry::Commission(order))) => order.action(&garden),
                (FarmPage::Commissions, _) => "Auftrag wählen",
                (FarmPage::Logbook, _) => "Rang einlösen",
                (FarmPage::Watch(site), _) => site.action(&garden),
                (FarmPage::WatchFinale, _) if garden.watches.lights() == Watch::ALL.len() => "Karte übergeben",
                (FarmPage::WatchFinale, _) => "Auf Karte zeigen",
                (FarmPage::BlueprintShop, _) => "Bauplan kaufen",
                (FarmPage::EveningMarket, _) => "Proviant kaufen",
                (FarmPage::Projects, Some(Entry::Project(Project::Restore))) => "Werkbank reparieren",
                (FarmPage::Projects, _) => "Bauen",
                (FarmPage::Tools { .. }, _) => "Verbessern",
                (FarmPage::CardShop, _) => "Karte kaufen",
                (FarmPage::Talents, _) => "Rang steigern",
                (FarmPage::Styles, _) => "Stil wählen",
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
        if matches!(field, Field::Instruction) && controller.active {
            if !value.is_empty() {
                value.push('\n');
            }
            value.push_str("Steuerkreuz · Wählen   A · Bestätigen   B · Zurück");
            if pages > 0 {
                value.push_str("   LB/RB · Seite");
            }
        }
        if text.0 != value {
            text.0 = value;
        }
    }

    for (&choice, &interaction, mut node, mut bg, mut focus_enabled) in &mut choices {
        let (visible, enabled) = match choice {
            Choice::Card(i) => (
                entry_at(i).is_some(),
                entry_at(i).is_some_and(|entry| entry.enabled(page, &garden)),
            ),
            Choice::Close => (true, true),
            Choice::Purchase => (page == FarmPage::SeedShop, match selected {
                Some(Entry::Seed(crop)) => balance >= crop.price(),
                Some(Entry::Material(Material::Fertilizer)) => balance >= FERTILIZER_PRICE,
                _ => false,
            }),
            Choice::MarketBuy => (matches!(page, FarmPage::SeedShop | FarmPage::MarketSale), page == FarmPage::MarketSale),
            Choice::MarketSell => (matches!(page, FarmPage::SeedShop | FarmPage::MarketSale), page == FarmPage::SeedShop),
            Choice::Back => (pages > 0, menu.list_page > 0),
            Choice::Next => (pages > 0, menu.list_page < pages),
            Choice::SellOne | Choice::SellAll => (page.selling(), selected.is_some()),
            Choice::Talents => (page == FarmPage::Binder, true),
            Choice::Training => (page == FarmPage::Binder, true),
            Choice::Logbook => (matches!(page, FarmPage::Journal(_) | FarmPage::Calendar), true),
            Choice::Styles => (page == FarmPage::Binder, true),
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
            Choice::Return => (!menu.history.is_empty(), true),
            Choice::Materials => (
                matches!(
                    page,
                    FarmPage::Workbench | FarmPage::Tools { at_forge: true }
                ),
                true,
            ),
            Choice::Batches => (matches!(page, FarmPage::Materials(_)), true),
            Choice::Commissions => (page == FarmPage::Journal(GuideTab::Orders), true),
            Choice::Abandon => (
                page == FarmPage::Commissions && garden.ledger.active().is_some(),
                true,
            ),
            Choice::Guide(tab) => (
                matches!(page, FarmPage::Calendar | FarmPage::Logbook)
                    || (page == FarmPage::Commissions && tab == GuideTab::Orders)
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
            Choice::PinPlan => (
                plan.is_some(),
                plan.is_some_and(|plan| garden.plan == Some(plan) || !plan.finished(&garden)),
            ),
            Choice::PreviewFurniture => (
                matches!(
                    page,
                    FarmPage::Workbench | FarmPage::BlueprintShop | FarmPage::Furnish
                ) && matches!(selected, Some(Entry::Design(_))),
                true,
            ),
            Choice::PlaceFurniture => (page == FarmPage::Workbench && matches!(selected, Some(Entry::Design(i)) if garden.crafted[i]), true),
            Choice::Arrange => (matches!(page, FarmPage::Workbench | FarmPage::Projects | FarmPage::Tools { .. }) && adventure.area.outdoors(), true),
            Choice::PreviewSlot(_) => (page == FarmPage::FurniturePreview, true),
            Choice::Work => {
                let available = match (page, selected) {
                    (FarmPage::Materials(bench), Some(Entry::Part(recipe))) => {
                        recipe.can_make(&garden, bench, batches)
                    }
                    (FarmPage::Commissions, Some(Entry::Commission(order))) => {
                        order.can_work(&garden)
                    }
                    (FarmPage::Logbook, Some(Entry::Practice(path))) => path.ready(&garden),
                    (FarmPage::Watch(site), _) => site.ready(&garden),
                    (FarmPage::WatchFinale, _) => !garden.watches.gathered,
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
                    (FarmPage::Styles, Some(Entry::Style(style))) => {
                        styles_unlocked(&garden) && garden.rpg.style != style
                    }
                    (FarmPage::Gear(source), Some(Entry::Gear(gear))) => {
                        source.ready(&garden, gear)
                    }
                    (FarmPage::Journal(_), Some(Entry::Guide(Guide::Farm))) => garden.task().1,
                    (FarmPage::Journal(_), Some(Entry::Guide(guide))) => {
                        guide.target(&garden).is_some()
                    }
                    (FarmPage::Calendar, _) => true,
                    (FarmPage::SeasonMeeting, _) => seasons::can_contribute(&garden),
                    (FarmPage::EveningMarket, Some(Entry::Ration(ration))) => {
                        evening_market::can_buy(&garden, ration)
                    }
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
                            | FarmPage::Materials(_)
                            | FarmPage::Commissions
                            | FarmPage::Watch(_)
                            | FarmPage::WatchFinale
                            | FarmPage::Tools { .. }
                            | FarmPage::BlueprintShop
                            | FarmPage::CardShop
                            | FarmPage::Talents
                            | FarmPage::Styles
                            | FarmPage::Gear(_)
                            | FarmPage::Journal(_)
                            | FarmPage::Calendar
                            | FarmPage::Logbook
                            | FarmPage::SeasonMeeting
                            | FarmPage::EveningMarket
                            | FarmPage::Harbor { .. }
                    ) && !matches!(page,FarmPage::Watch(site) if site.stage(&garden)==WatchStage::Complete && site.trial(&garden).is_none()),
                    available,
                )
            }
        };
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        focus_enabled.0 = visible && (enabled || matches!(choice, Choice::Card(_)));
        bg.0 = if let Choice::Card(i) = choice {
            let entry = entry_at(i);
            let highlighted = entry == selected && selected.is_some();
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

fn styles_unlocked(garden: &Garden) -> bool {
    garden.rpg.cleared > 0 || Watch::Fern.stage(garden) == WatchStage::Complete
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn back_restores_a_submenu_origin_and_the_deck_draft() {
        let garden = Garden::default();
        let mut menu = FarmMenu::default();
        menu.show(FarmPage::Binder);
        menu.load_deck(&garden);
        menu.deck_draft.pop();
        menu.list_page = 1;
        menu.selected = Some(Entry::CombatCard(Card::Scout));
        menu.open_child(FarmPage::Talents);
        menu.back();
        assert!(menu.page == Some(FarmPage::Binder));
        assert!(menu.selected == Some(Entry::CombatCard(Card::Scout)));
        assert_eq!(menu.list_page, 1);
        menu.show_battle(Battle::training());
        menu.back();
        assert!(menu.page == Some(FarmPage::Binder));
        assert!(menu.battle().is_none());
        assert_eq!(menu.deck_draft.len(), DECK_SIZE - 1);
    }

    #[test]
    fn back_from_a_station_returns_to_the_recipes() {
        let mut menu = FarmMenu::default();
        menu.show(FarmPage::Kitchen);
        menu.preparation = Some(Preparation::new(Dish::CarrotSoup, 0.));
        menu.back();
        assert!(menu.page == Some(FarmPage::Kitchen));
        assert!(menu.preparation.is_none());
        menu.back();
        assert!(!menu.open());
    }
}
