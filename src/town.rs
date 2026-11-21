use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    Controls, Notice,
    adventure::commit,
    evening_market::{self, Stage as MarketStage},
    farm_ui::{FarmMenu, FarmPage},
    garden::{Crop, Garden, SavePath},
    harbor,
    homestead::{Product, Village},
    input::Gesture,
    regions::{self, Frontier},
    session::Session,
    travel::FINDS,
    ui::{self, UiFonts},
    weather::Season,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Npc {
    Lina,
    Miro,
    Mara,
    Fiete,
    Bo,
    Juna,
    Runa,
    Nele,
    Edda,
    Ivo,
    Sanna,
    Tilda,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum QuestMark {
    Available,
    TurnIn,
}

impl QuestMark {
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Available => "!",
            Self::TurnIn => "?",
        }
    }
}

impl Npc {
    pub const ALL: [Self; 12] = [
        Self::Lina,
        Self::Miro,
        Self::Mara,
        Self::Fiete,
        Self::Bo,
        Self::Juna,
        Self::Runa,
        Self::Nele,
        Self::Edda,
        Self::Ivo,
        Self::Sanna,
        Self::Tilda,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Lina => "Lina",
            Self::Miro => "Miro",
            Self::Mara => "Mara",
            Self::Fiete => "Fiete",
            Self::Bo => "Bo",
            Self::Juna => "Juna",
            Self::Runa => "Runa",
            Self::Nele => "Nele",
            Self::Edda => "Edda",
            Self::Ivo => "Ivo",
            Self::Sanna => "Sanna",
            Self::Tilda => "Tilda",
        }
    }
    pub fn role(self) -> &'static str {
        match self {
            Self::Lina => "Saatguthändlerin",
            Self::Miro => "Markthändler",
            Self::Mara => "Köchin",
            Self::Fiete => "Muschelsammler",
            Self::Bo => "Hafenmeister",
            Self::Juna => "Wegkundlerin",
            Self::Runa => "Schmiedin der Kupferklamm",
            Self::Nele => "Müllerin der Windweiden",
            Self::Edda => "Wegwacht im Nebelmoor",
            Self::Ivo => "Harzsammler",
            Self::Sanna => "Bergschmiedin",
            Self::Tilda => "Hüterin der Tümpel",
        }
    }
    pub fn model(self) -> &'static str {
        match self {
            Self::Lina => "npc_lina",
            Self::Miro => "npc_miro",
            Self::Mara => "npc_mara",
            Self::Fiete => "npc_fiete",
            Self::Bo => "npc_bo",
            Self::Juna => "npc_juna",
            Self::Runa => "npc_runa",
            Self::Nele => "npc_nele",
            Self::Edda => "npc_edda",
            Self::Ivo => "npc_ivo",
            Self::Sanna => "npc_sanna",
            Self::Tilda => "npc_tilda",
        }
    }
    pub fn frontier(self) -> Option<Frontier> {
        match self {
            Self::Runa => Some(Frontier::Copper),
            Self::Nele => Some(Frontier::Wind),
            Self::Edda => Some(Frontier::Moor),
            _ => None,
        }
    }

    pub fn expansion(self) -> Option<crate::expansion::Region> {
        use crate::expansion::Region;
        match self {
            Self::Ivo => Some(Region::Amber),
            Self::Sanna => Some(Region::Heights),
            Self::Tilda => Some(Region::Reef),
            _ => None,
        }
    }

    pub fn available(self, garden: &Garden) -> bool {
        self.frontier().is_none_or(|region| region.unlocked(garden))
            && self.expansion().is_none_or(|region| region.open(garden))
    }

    pub fn position(self, garden: &Garden) -> Vec2 {
        crate::resident::pose(self, garden).point
    }

    pub fn whereabouts(self, garden: &Garden) -> &'static str {
        crate::resident::pose(self, garden).activity
    }

    fn harbor_greeting(self, garden: &Garden) -> Option<String> {
        if !matches!(self, Self::Bo | Self::Fiete) {
            return None;
        }
        if self == Self::Bo
            && garden.onboarding == crate::onboarding::Onboarding::Done
            && crate::waywatch::Watch::Fern.stage(garden) == crate::waywatch::Stage::Unheard
            && !garden.harbor.started
        {
            return Some("Meine Familie zählte an der Farnwacht die Lichter vor jedem Sturm. Nimm Holz und Stein für deine Werkbank. Folge der Markierung in den Farnwald und schau unter das grüne Dach.".into());
        }
        let phase = garden.harbor.phase;
        let greeting = match (self, phase) {
            (Self::Bo, harbor::Phase::Damaged) => {
                "Der Sturm hat die Bretter vom alten Fischersteg gelöst. Schau doch mit Fiete am Ufer nach gutem Treibholz."
            }
            (Self::Fiete, harbor::Phase::Damaged) => {
                "Gleich südwestlich vom Bautisch habe ich Treibholz zusammengebunden. Die Bretter tragen uns noch viele Jahre."
            }
            (Self::Bo, harbor::Phase::Landing) => {
                "Der Anleger trägt wieder. Bring zwei Kupfer oder fertige Beschläge zusammen mit Holz und Stein zum Bautisch."
            }
            (Self::Fiete, harbor::Phase::Landing) => {
                "Ich habe meinen Eimer schon zum Anleger gebracht. Probier es mit der Angel am Ende der frischen Bretter."
            }
            (Self::Bo, harbor::Phase::Rails) => {
                "Die Geländer sitzen. Wenn du etwas aus deinem Garten mitbringst, können wir am Steg gemeinsam essen."
            }
            (Self::Fiete, harbor::Phase::Rails) => {
                "Ich habe am Ende des Stegs Platz für die Bänder gelassen. Bring etwas Frisches für unseren Tisch mit."
            }
            (_, harbor::Phase::Open) if garden.harbor.celebrating(garden.day) => {
                "Herzlich willkommen zur Wiedereröffnung! Heute essen wir draußen auf den frischen Brettern."
            }
            (Self::Bo, harbor::Phase::Open) => {
                "Die Hafenlaterne leuchtet wieder bis zur Seeseite. Miss ruhig mit deiner Angel nach, was dort schwimmt."
            }
            (_, harbor::Phase::Open) => {
                "An der Seeseite fangen wir Hafenhering. Die alte Köderkiste steht gleich neben dem Geländer."
            }
            _ => return None,
        };
        let weather = match Season::at(garden.day) {
            Season::Spring => "Am Ufer treiben die ersten Blütenzweige an.",
            Season::Summer => "Abends bleibt es am Wasser lange warm.",
            Season::Autumn => "In der Hafengasse raschelt das Laub.",
            Season::Winter => "Auf den Pfählen glitzert Raureif.",
        };
        Some(format!("{greeting} {weather}"))
    }
    fn greeting(self) -> &'static str {
        match self {
            Self::Lina => {
                "Moin! Ich ziehe hier alles aus Samen. Was soll in deinem Garten wachsen?"
            }
            Self::Miro => {
                "Willkommen in Eichenhafen. Ich nehme dir gern Obst, Fisch oder einen Fund von unterwegs ab."
            }
            Self::Mara => {
                "In meinem Topf fehlen noch Pilze und Haselnüsse. Im Farnwald findest du beides, auch drüben am Bach."
            }
            Self::Fiete => {
                "An der Muschelbucht spült die Flut die schönsten Schalen an. Hilfst du mir, drei für mein Windspiel zu sammeln?"
            }
            Self::Bo => {
                "Der Brunnen braucht neue Einfassungen. Mit ein paar Brettern und Steinen können wir ihn ausbessern."
            }
            Self::Juna => {
                "Die Nordweststeine führen in den Farnwald. Folge für die neuen Wege den Schildern und hilf zuerst den Leuten in Eichenhafen."
            }
            Self::Runa => {
                "Willkommen zwischen den Felsen! Wenn der Quarz in den Wänden leuchtet, finden auch unsere Bauleute den Heimweg."
            }
            Self::Nele => {
                "Der Wind steht gut. Mit etwas Hilfe kann meine Mühle wieder die ganze Weide versorgen."
            }
            Self::Edda => {
                "Geh auf den Holzstegen und lausche den Glocken. Im Nebel kennen sie jeden Weg."
            }
            Self::Ivo => "Psst. Wenn wir leise sind, trauen sich die Langohren bis zu den Harzschalen.",
            Self::Sanna => "Morgens ist die Scharte windstill. Oben hört man dann sogar, wie sich die Ziegen im Geröll rufen.",
            Self::Tilda => "Die Laufenten sind früher als ich wach. Folgen wir ihren Spuren, finden wir die ersten Salzblüten.",
        }
    }

    fn merchant(self) -> Option<Village> {
        match self {
            Self::Lina => Some(Village::Seeds),
            Self::Miro => Some(Village::Sell),
            _ => None,
        }
    }

    fn quest(self) -> Option<Quest> {
        match self {
            Self::Mara => Some(Quest::Soup),
            Self::Fiete => Some(Quest::Chime),
            Self::Bo => Some(Quest::Fountain),
            _ => None,
        }
    }

    fn current_quest(self, garden: &Garden) -> Option<Quest> {
        let first = self.quest()?;
        if garden.town.state(first) != QuestState::Done {
            return Some(first);
        }
        let follow = match self {
            Self::Mara if garden.harbor.phase == harbor::Phase::Open => Quest::LongTable,
            Self::Fiete if garden.workshop.forge => Quest::StormBell,
            Self::Bo if garden.farm.housing[0] && garden.farm.housing[1] => Quest::FirstFair,
            _ => return Some(first),
        };
        Some(follow)
    }

    pub fn quest_mark(self, garden: &Garden) -> Option<QuestMark> {
        use QuestMark::{Available, TurnIn};
        if let Some(region) = self.expansion() { return region.mark(garden); }
        if let Some(region) = self.frontier() {
            return region
                .unlocked(garden)
                .then(|| regions::story_mark(region, garden))
                .flatten();
        }
        let city = self
            .current_quest(garden)
            .and_then(|quest| match garden.town.state(quest) {
                QuestState::Offered => Some(Available),
                QuestState::Accepted if quest.ready(garden) => Some(TurnIn),
                _ => None,
            });
        let story = match self {
            Self::Bo if garden.onboarding == crate::onboarding::Onboarding::Harbor => Some(TurnIn),
            Self::Bo if !garden.harbor.started => Some(Available),
            Self::Juna
                if !garden.watches.gathered
                    && garden.watches.lights() == crate::waywatch::Watch::ALL.len() =>
            {
                Some(TurnIn)
            }
            Self::Juna if !garden.story.met_juna => Some(Available),
            _ => None,
        };
        let market = if self == Self::Mara {
            evening_market::mark(garden)
        } else {
            None
        };
        let supper = (self == Self::Bo && garden.fish_supper.stage == crate::fish_supper::Stage::Visiting).then_some(Available);
        city.max(story).max(market).max(supper)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Quest {
    Soup,
    Chime,
    Fountain,
    LongTable,
    StormBell,
    FirstFair,
}

#[derive(Clone, Copy)]
pub(crate) enum Require {
    Product(Product, u32),
    Wood(u32),
    Stone(u32),
    Copper(u32),
}

impl Require {
    pub(crate) fn available(self, garden: &Garden) -> u32 {
        match self {
            Self::Product(product, _) => garden.stock(product),
            Self::Wood(_) => garden.wood,
            Self::Stone(_) => garden.stone,
            Self::Copper(_) => garden.copper,
        }
    }
    pub(crate) fn quantity(self) -> u32 {
        match self {
            Self::Product(_, quantity)
            | Self::Wood(quantity)
            | Self::Stone(quantity)
            | Self::Copper(quantity) => quantity,
        }
    }
    pub(crate) fn progress(self, garden: &Garden) -> String {
        let name = match self {
            Self::Product(product, _) => product.name(),
            Self::Wood(_) => "Holz",
            Self::Stone(_) => "Stein",
            Self::Copper(_) => "Kupfer",
        };
        let quantity = self.quantity();
        format!(
            "{}/{} {name}",
            self.available(garden).min(quantity),
            quantity
        )
    }
    pub(crate) fn take(self, g: &mut Garden) {
        match self {
            Self::Product(product, quantity) => {
                g.take_stock(product, quantity);
            }
            Self::Wood(quantity) => g.wood -= quantity,
            Self::Stone(quantity) => g.stone -= quantity,
            Self::Copper(quantity) => g.copper -= quantity,
        }
    }
}

impl Quest {
    const ALL: [Self; 6] = [
        Self::Soup, Self::Chime, Self::Fountain, Self::LongTable, Self::StormBell, Self::FirstFair,
    ];

    fn blueprint(self) -> usize {
        match self {
            Self::Soup => crate::furniture::ARMCHAIR,
            Self::Chime => crate::furniture::VINE_SHELF,
            Self::Fountain => crate::furniture::WALL_HANGING,
            Self::LongTable => crate::furniture::HARBOR_CHEST,
            Self::StormBell => crate::furniture::VINE_SHELF,
            Self::FirstFair => crate::furniture::ARMCHAIR,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Soup => "Für Maras Küche",
            Self::Chime => "Klang der Bucht",
            Self::Fountain => "Ein Brunnen für alle",
            Self::LongTable => "Ein Platz am langen Tisch",
            Self::StormBell => "Eine Glocke für den Sturm",
            Self::FirstFair => "Der erste Hofmarkt",
        }
    }
    fn request(self) -> &'static str {
        match self {
            Self::Soup => "3 Pilze und 2 Haselnüsse aus dem Farnwald",
            Self::Chime => "3 Muscheln aus der Muschelbucht",
            Self::Fountain => "4 Holz und 3 Stein aus Wald, Bucht oder Mine",
            Self::LongTable => "Zwei Möhren-Pilz-Suppen kochen und Waldbeeren einmachen.",
            Self::StormBell => "Bretter an der Werkbank und Nieten am Schmiedetisch herstellen.",
            Self::FirstFair => "Eier und Wolle einsammeln und Haselnüsse zu Mehl mahlen.",
        }
    }
    fn brief(self) -> &'static str {
        match self {
            Self::Soup => "Mara rührt seit Sonnenaufgang in einem leeren Topf. Für die späte Schicht am Hafen will sie die Pilzsuppe ihrer Großmutter kochen. Die letzten Haselnüsse hat der Herbststurm vom Dach gefegt.",
            Self::Chime => "Fiete hat am Strand einen Holzrahmen aus seiner Kindheit gefunden. Am Fenster soll er wieder wie die Muscheln an der Bucht klingen. Drei Muscheln mit heilen Rändern fehlen ihm.",
            Self::Fountain => "Beim Wasserschöpfen hat Bo die losen Steine am Brunnen gezählt. Mit einem neuen Rand könnten sich die Nachbarn wieder dort treffen und ihre Eimer sicher abstellen.",
            Self::LongTable => "Seit der Fischersteg wieder offen ist, bleiben die Heimkehrenden abends am Brunnen stehen. Mara möchte einen langen Tisch decken. Sie bittet dich um etwas Warmes und um ein Glas vom letzten Beerenfund.",
            Self::StormBell => "Beim letzten Nebel konnte Fiete die Fischerboote vom Ufer aus kaum erkennen. Er möchte eine Schutzhaube für ihre Glocke bauen. An deinem Schmiedetisch entstehen Nieten, die dem Wind standhalten.",
            Self::FirstFair => "Bo will die Türen am Hafenmarkt für die Höfe der Insel öffnen. Er verspricht, das erste Brot aus deinem Mehl mit den Nachbarn zu teilen. Für seinen Tisch fehlen Eier und Wolle von versorgten Tieren.",
        }
    }

    fn requirements(self) -> &'static [Require] {
        match self {
            Self::Soup => &[
                Require::Product(Product::Mushroom, 3),
                Require::Product(Product::Hazel, 2),
            ],
            Self::Chime => &[Require::Product(Product::Shell, 3)],
            Self::Fountain => &[Require::Wood(4), Require::Stone(3)],
            Self::LongTable => &[
                Require::Product(Product::Prepared(crate::processing::Dish::CarrotSoup), 2),
                Require::Product(Product::Prepared(crate::processing::Dish::BerryJam), 1),
            ],
            Self::StormBell => &[
                Require::Product(Product::Crafted(crate::craft::Recipe::Boards), 5),
                Require::Product(Product::Crafted(crate::craft::Recipe::Rivets), 3),
            ],
            Self::FirstFair => &[
                Require::Product(Product::Egg, 4),
                Require::Product(Product::Wool, 2),
                Require::Product(Product::Prepared(crate::processing::Dish::HazelFlour), 1),
            ],
        }
    }
    fn reward(self) -> (u32, Crop) {
        match self {
            Self::Soup => (18, Crop::Tomato),
            Self::Chime => (16, Crop::Strawberry),
            Self::Fountain => (24, Crop::Sunflower),
            Self::LongTable => (48, Crop::Pumpkin),
            Self::StormBell => (56, Crop::Sunflower),
            Self::FirstFair => (72, Crop::Strawberry),
        }
    }

    fn ready(self, garden: &Garden) -> bool {
        self.requirements()
            .iter()
            .all(|req| req.available(garden) >= req.quantity())
    }

    fn advance(self, g: &mut Garden) -> Result<String, String> {
        match g.town.state(self) {
            QuestState::Offered => {
                *g.town.state_mut(self) = QuestState::Accepted;
                Ok("Auftrag angenommen.".into())
            }
            QuestState::Accepted if self.ready(g) => {
                for item in self.requirements() {
                    item.take(g);
                }
                let (coins, crop) = self.reward();
                g.farm.coins = g.farm.coins.saturating_add(coins);
                g.add_seeds(crop, 2);
                *g.town.state_mut(self) = QuestState::Done;
                crate::furniture::learn(g, self.blueprint());
                Ok(format!("{} erfüllt · +{coins} Münzen · +2 Samen", self.name()))
            }
            QuestState::Accepted => Err("Dir fehlen noch Zutaten oder Materialien.".into()),
            QuestState::Done => Err("Dieser Auftrag ist erfüllt.".into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
enum QuestState {
    #[default]
    Offered,
    Accepted,
    Done,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Journal {
    quests: [QuestState; 3],
    follow_up: [QuestState; 3],
    picked: Vec<Option<u32>>,
    pub goods: [u32; 3],
}

impl Journal {
    fn state(&self, quest: Quest) -> QuestState {
        let index = quest as usize;
        if index < self.quests.len() { self.quests[index] } else { self.follow_up[index - self.quests.len()] }
    }

    fn state_mut(&mut self, quest: Quest) -> &mut QuestState {
        let index = quest as usize;
        if index < self.quests.len() { &mut self.quests[index] } else { &mut self.follow_up[index - self.quests.len()] }
    }

    pub fn earned_blueprints(&self) -> impl Iterator<Item = usize> + '_ {
        Quest::ALL
            .into_iter()
            .filter(|&quest| self.state(quest) == QuestState::Done)
            .map(Quest::blueprint)
    }

    pub fn completed(&self) -> usize {
        self.quests.iter().filter(|&&state| state == QuestState::Done).count()
    }

    pub fn total_completed(&self) -> usize {
        Quest::ALL.into_iter().filter(|&quest| self.state(quest) == QuestState::Done).count()
    }

    pub fn find_ready(&self, index: usize, day: u32) -> bool {
        self.picked
            .get(index)
            .copied()
            .flatten()
            .is_none_or(|last| day.saturating_sub(last) >= 2)
    }

    pub fn record_pick(&mut self, index: usize, day: u32) {
        self.picked.resize(FINDS.len(), None);
        self.picked[index] = Some(day);
    }
}

#[derive(Resource, Default)]
pub struct Dialog {
    npc: Option<Npc>,
    trading: bool,
    index: usize,
    message: String,
}

impl Dialog {
    pub(crate) fn person(&self) -> Option<Npc> {
        self.npc
    }

    pub fn open(&self) -> bool {
        self.npc.is_some()
    }
    pub fn talk(&mut self, npc: Npc) {
        *self = Self {
            npc: Some(npc),
            ..default()
        };
    }
    pub fn close(&mut self) {
        *self = Self::default();
    }
    pub fn back(&mut self) {
        if self.trading {
            self.trading = false;
        } else {
            self.close();
        }
    }
    fn product(&self, garden: &Garden) -> Option<Product> {
        Product::at(self.index, garden).or_else(|| Product::at(0, garden))
    }
    fn crop(&self) -> Crop {
        Crop::ALL[self.index % Crop::ALL.len()]
    }
}

#[derive(Component)]
pub struct DialoguePane;
#[derive(Component)]
pub enum DialogueText {
    Name,
    Role,
    Greeting,
    Request,
    Status,
}
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum DialogueButton {
    Close,
    Trade,
    Pick,
    Deal,
    Back,
    Quest,
    Story,
    Harbor,
    EveningMarket,
    FishSupper,
    Plans,
    WatchMap,
}
#[derive(Component)]
pub struct DialogueLabel(DialogueButton);

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>) {
    commands
        .spawn((
            DialoguePane,
            crate::controller::Panel::Dialogue,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(24.)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.18, 0.16, 0.31)),
            GlobalZIndex(88),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        width: Val::Px(418.),
                        max_width: Val::Percent(94.),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(11.),
                        padding: UiRect::all(Val::Px(23.)),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(24.)),
                    BackgroundColor(ui::cream()),
                    ui::shadow(),
                ))
                .with_children(|panel| {
                    panel.spawn((DialogueText::Role, fonts.bold("", 14.)));
                    panel.spawn((DialogueText::Name, fonts.title("", 35.)));
                    panel.spawn((DialogueText::Greeting, fonts.body("", 17.)));
                    panel.spawn((DialogueText::Request, fonts.bold("", 15.)));
                    panel.spawn((DialogueText::Status, fonts.body("", 14.)));
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            justify_content: JustifyContent::Start,
                            column_gap: Val::Px(9.),
                            row_gap: Val::Px(8.),
                            ..default()
                        })
                        .with_children(|buttons| {
                            for (choice, label, width) in [
                                (DialogueButton::FishSupper, "Hafenbrühe", 174.),
                                (DialogueButton::Trade, "Handeln", 129.),
                                (DialogueButton::Pick, "Nächste Ware", 129.),
                                (DialogueButton::Deal, "Kaufen", 129.),
                                (DialogueButton::Quest, "Annehmen", 187.),
                                (DialogueButton::Story, "Nachfragen", 161.),
                                (DialogueButton::EveningMarket, "Abendmarkt", 193.),
                                (DialogueButton::Harbor, "Alter Fischersteg", 180.),
                                (DialogueButton::Plans, "Baupläne", 103.),
                                (DialogueButton::WatchMap, "Inselkarte", 118.),
                                (DialogueButton::Back, "Zurück", 105.),
                                (DialogueButton::Close, "Schließen", 103.),
                            ] {
                                buttons
                                    .spawn((choice, ui::dialog_button(width)))
                                    .with_child((DialogueLabel(choice), fonts.button(label, 14.)));
                            }
                        });
                });
        });
}

pub fn input(
    session: Res<Session>,
    buttons: Query<(&DialogueButton, &Interaction), Changed<Interaction>>,
    mut dialog: ResMut<Dialog>,
    mut garden: ResMut<Garden>,
    path: Res<SavePath>,
    mut notice: ResMut<Notice>,
    time: Res<Time>,
    mut gesture: ResMut<Gesture>,
    mut controls: ResMut<Controls>,
    mut farm_menu: ResMut<FarmMenu>,
) {
    let Some(npc) = dialog.npc.filter(|_| session.playing()) else {
        return;
    };
    let Some(choice) = buttons
        .iter()
        .find_map(|(choice, pressed)| (*pressed == Interaction::Pressed).then_some(*choice))
    else {
        return;
    };
    let result = match choice {
        DialogueButton::Close => {
            dialog.close();
            *gesture = Gesture::default();
            *controls = Controls::default();
            None
        }
        DialogueButton::Trade if matches!(npc, Npc::Runa | Npc::Miro | Npc::Lina) && garden.clock.market_open() => {
            dialog.close();
            *gesture = Gesture::default();
            *controls = Controls::default();
            farm_menu.show(match npc {
                Npc::Lina => FarmPage::SeedShop,
                Npc::Miro => FarmPage::MarketSale,
                _ => FarmPage::CardShop,
            });
            None
        }
        DialogueButton::Trade if !dialog.trading && npc.merchant().is_some() => {
            dialog.trading = true;
            dialog.index = 0;
            dialog.message.clear();
            None
        }
        DialogueButton::Back => {
            dialog.trading = false;
            dialog.message.clear();
            None
        }
        DialogueButton::Plans if npc == Npc::Bo && garden.clock.market_open() => {
            dialog.close();
            *gesture = Gesture::default();
            *controls = Controls::default();
            farm_menu.show(FarmPage::BlueprintShop);
            None
        }
        DialogueButton::WatchMap if matches!(npc, Npc::Juna | Npc::Bo) => {
            dialog.close();
            farm_menu.show(if npc == Npc::Juna {
                FarmPage::WatchFinale
            } else {
                FarmPage::Journal(crate::story::GuideTab::Watches)
            });
            *gesture = Gesture::default();
            *controls = Controls::default();
            None
        }
        DialogueButton::Pick if dialog.trading => {
            dialog.index += 1;
            if npc == Npc::Miro && Product::at(dialog.index, &garden).is_none() {
                dialog.index = 0;
            }
            dialog.message.clear();
            None
        }
        DialogueButton::Deal if dialog.trading => {
            let crop = dialog.crop();
            let product = dialog.product(&garden);
            npc.merchant()
                .map(|merchant| merchant.work(&mut garden, crop, product))
        }
        DialogueButton::Quest if !dialog.trading => {
            if let Some(area) = npc.expansion() {
                Some(area.finish(&mut garden))
            } else if let Some(area) = npc.frontier() {
                Some(regions::finish_story(area, &mut garden))
            } else {
                npc.current_quest(&garden).map(|quest| quest.advance(&mut garden))
            }
        }
        DialogueButton::FishSupper if npc == Npc::Bo && !dialog.trading => {
            use crate::fish_supper::Stage;
            match garden.fish_supper.stage {
                Stage::Visiting => Some(crate::fish_supper::learn(&mut garden)),
                Stage::Cooking => {
                    dialog.close();
                    *gesture = Gesture::default();
                    *controls = Controls::default();
                    farm_menu.cook_with_bo(&garden);
                    None
                }
                Stage::Served if garden.fish_supper.offered_today(garden.day) && evening_market::hours(&garden) => {
                    dialog.close();
                    *gesture = Gesture::default();
                    *controls = Controls::default();
                    farm_menu.show(FarmPage::EveningMarket);
                    None
                }
                _ => None,
            }
        }
        DialogueButton::EveningMarket if npc == Npc::Mara && !dialog.trading => {
            if evening_market::trading(&garden) {
                dialog.close();
                *gesture = Gesture::default();
                *controls = Controls::default();
                farm_menu.show(FarmPage::EveningMarket);
                None
            } else {
                Some(evening_market::advance(&mut garden))
            }
        }
        DialogueButton::Story if npc == Npc::Juna && !garden.story.met_juna => {
            garden.story.met_juna = true;
            Some(Ok(
                "Ins Journal geschrieben · Das Licht der Wegglocken".into()
            ))
        }
        DialogueButton::Harbor if !dialog.trading && matches!(npc, Npc::Bo | Npc::Fiete) => {
            dialog.close();
            farm_menu.show(FarmPage::Harbor { at_site: false });
            *gesture = Gesture::default();
            *controls = Controls::default();
            None
        }
        _ => None,
    };
    if let Some(result) = result {
        match result {
            Ok(message) => {
                commit(
                    &garden,
                    &path,
                    &mut notice,
                    time.elapsed_secs(),
                    message,
                    None,
                    None,
                );
                dialog.message.clone_from(&notice.message);
            }
            Err(message) => dialog.message = message,
        }
    }
}

pub fn update(
    session: Res<Session>,
    dialog: Res<Dialog>,
    garden: Res<Garden>,
    mut pane: Single<&mut Node, With<DialoguePane>>,
    mut fields: Query<(&DialogueText, &mut Text), Without<DialogueLabel>>,
    mut labels: Query<(&DialogueLabel, &mut Text), Without<DialogueText>>,
    mut buttons: Query<
        (
            &DialogueButton,
            &mut Node,
            &Interaction,
            &mut BackgroundColor,
            &mut crate::controller::FocusEnabled,
        ),
        Without<DialoguePane>,
    >,
) {
    let display = if dialog.open() && session.playing() {
        Display::Flex
    } else {
        Display::None
    };
    if pane.display != display {
        pane.display = display;
    }
    let Some(npc) = dialog.npc else { return };
    let merchant = npc.merchant();
    let trade =
        merchant.map(|merchant| merchant.describe(&garden, dialog.crop(), dialog.product(&garden)));
    let quest = npc.current_quest(&garden);
    let region_quest = npc.expansion().map(|area| area.dialogue(&garden))
        .or_else(|| npc.frontier().map(|area| regions::story(area, &garden)));
    let quest_state = quest.map(|q| garden.town.state(q));
    for (field, mut text) in &mut fields {
        let value = match field {
            DialogueText::Name => npc.name().into(),
            DialogueText::Role if matches!(npc, Npc::Bo | Npc::Fiete) => {
                format!("{} · {}", npc.role(), npc.whereabouts(&garden))
            }
            DialogueText::Role => npc.role().into(),
            DialogueText::Greeting if dialog.trading => {
                "Wähle eine Ware und bestätige den Handel.".into()
            }
            DialogueText::Greeting if npc == Npc::Bo && garden.fish_supper.at_farm() =>
                if garden.fish_supper.learned() { "Ich halte den Topf warm. Hast du den Fisch und die Möhre? Dann stell dich zu mir." }
                else { "Dein erster Fang! Mein Vater hat aus seinem eine Brühe für den ganzen Kai gekocht. Hier, ich hab eine Möhre mitgebracht. Kochen wir zusammen?" }.into(),
            DialogueText::Greeting if npc == Npc::Bo && garden.fish_supper.waiting_at_farm(garden.day, garden.clock.minute()) =>
                "Lass die Brühe noch ein bisschen ziehen. Am nächsten Marktabend stelle ich eine Tafel für unser Rezept auf.".into(),
            DialogueText::Greeting if npc == Npc::Bo && garden.fish_supper.offered_today(garden.day) && evening_market::hours(&garden) =>
                "Riechst du den Dill? Seit wir an deinem Hof gekocht haben, gibt es unsere Brühe hier am Brunnen.".into(),
            DialogueText::Greeting if npc == crate::weather::Season::at(garden.day).host()
                && crate::weather::Season::day(garden.day) >= crate::seasons::MEETING_START => {
                format!("{} Komm zum Dorftisch am Marktplatz.",
                        crate::weather::Season::at(garden.day).invitation())
            }
            DialogueText::Greeting if matches!(npc, Npc::Bo | Npc::Fiete) => {
                npc.harbor_greeting(&garden).unwrap_or_else(|| npc.greeting().into())
            }
            DialogueText::Greeting if npc == Npc::Mara && evening_market::invited(&garden) => evening_market::greeting(&garden).into(),
            DialogueText::Greeting if quest_state == Some(QuestState::Done) => {
                "Danke für deine Hilfe. Schau gern wieder vorbei!".into()
            }
            DialogueText::Greeting if npc == Npc::Juna && garden.story.harbor_lit =>
                "Hörst du das? Sogar die Fischer am anderen Ufer drehen sich nach unserem Lichtfest um.".into(),
            DialogueText::Greeting if npc == Npc::Juna =>
                "Vor Jahren stimmten unsere Leute ihre Glocken nach dem Klang der Küste. Als die letzte Dämmerglocke verstummte, erlosch auch das Licht im Hafen.".into(),
            DialogueText::Greeting if npc == Npc::Runa && garden.story.tuned(Frontier::Copper) =>
                "Das Gleis kenne ich noch von meiner Großmutter. Die Schmiede gab dort den Takt vor, damit alle Wagen sicher über die Brücke kamen.".into(),
            DialogueText::Greeting if npc == Npc::Nele && garden.story.tuned(Frontier::Wind) =>
                "Der Klang hinter der Mühle ist zurück. Meine Mutter sagte immer, wir könnten daran hören, wann der Flachs trocken wird.".into(),
            DialogueText::Greeting if npc == Npc::Edda && garden.story.tuned(Frontier::Moor) =>
                "Das Moor antwortet wieder. Folg dem alten Steinbogen, wenn du herausfinden willst, warum die Dämmerglocken so lange stumm waren.".into(),
            DialogueText::Greeting => npc.greeting().into(),
            DialogueText::Request if dialog.trading => {
                trade.as_ref().map_or_else(String::new, |t| t.0.clone())
            }
            DialogueText::Request if npc == Npc::Bo && garden.fish_supper.at_farm() => {
                if garden.fish_supper.learned() {
                    crate::processing::Dish::HarborBroth.supplies(&garden)
                } else {
                    "Bos Familienrezept · 1 Fisch oder Hafenhering und 1 Möhre".into()
                }
            }
            DialogueText::Request if npc == Npc::Mara && evening_market::invited(&garden) => evening_market::detail(&garden),
            DialogueText::Request if region_quest.is_some() => region_quest
                .as_ref()
                .map_or_else(String::new, |value| value.0.clone()),
            DialogueText::Request if npc == Npc::Juna && !garden.story.met_juna =>
                "In der Klamm, auf der Weide und im Moor stehen noch die alten Klangsteine. Hilfst du mir, ihre Stimmen für das Lichtfest zu finden?".into(),
            DialogueText::Request if npc == Npc::Juna && !garden.story.harbor_lit => format!(
                "{}/3 Klangsteine gestimmt · Das Journal zeigt, wo du weitersuchen kannst.",
                garden.story.voices()
            ),
            DialogueText::Request => {
                quest
                    .zip(quest_state)
                    .map_or_else(String::new, |(quest, state)| {
                        let (coins, crop) = quest.reward();
                        match state {
                            QuestState::Done => format!("{} · Erfüllt", quest.name()),
                            _ => format!(
                                "{}\n{}\n{}\n{coins} Münzen und 2 Samen für {} als Belohnung",
                                quest.name(),
                                quest.brief(),
                                quest.request(),
                                crop.name(),
                            ),
                        }
                    })
            }
            DialogueText::Status
                if dialog.message.is_empty() && quest_state == Some(QuestState::Accepted) =>
            {
                quest.map_or_else(String::new, |quest| {
                    quest
                        .requirements()
                        .iter()
                        .map(|item| item.progress(&garden))
                        .collect::<Vec<_>>()
                        .join(" · ")
                })
            }
            DialogueText::Status => dialog.message.clone(),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
    for (label, mut text) in &mut labels {
        let caption = match label.0 {
            DialogueButton::Trade if npc == Npc::Runa => "Karten & Waffen",
            DialogueButton::WatchMap if npc == Npc::Bo => "Wegwachten",
            DialogueButton::WatchMap => "Inselkarte",
            DialogueButton::Trade => "Handeln",
            DialogueButton::FishSupper => match garden.fish_supper.stage {
                crate::fish_supper::Stage::Visiting => "Rezept lernen",
                crate::fish_supper::Stage::Cooking => "Mit Bo kochen",
                _ => "Hafenbrühe ansehen",
            },
            DialogueButton::EveningMarket => evening_market::action(&garden),
            DialogueButton::Deal if npc == Npc::Miro => "Verkaufen",
            DialogueButton::Deal => "Saat kaufen",
            DialogueButton::Quest if region_quest.is_some() => region_quest.as_ref().unwrap().1,
            DialogueButton::Quest if quest_state == Some(QuestState::Accepted) => "Auftrag abgeben",
            DialogueButton::Quest if npc == Npc::Bo && garden.fish_supper.at_farm() =>
                quest.map_or("Auftrag annehmen", Quest::name),
            DialogueButton::Quest => "Auftrag annehmen",
            _ => continue,
        };
        if text.0 != caption {
            text.0 = caption.into();
        }
    }
    for (button, mut node, interaction, mut bg, mut focus) in &mut buttons {
        let shown = match button {
            DialogueButton::FishSupper => npc == Npc::Bo && !dialog.trading && (garden.fish_supper.at_farm()
                || garden.fish_supper.offered_today(garden.day) && evening_market::hours(&garden)),
            DialogueButton::Trade => (merchant.is_some() || npc == Npc::Runa) && !dialog.trading,
            DialogueButton::Pick | DialogueButton::Deal | DialogueButton::Back => dialog.trading,
            DialogueButton::Quest => {
                !dialog.trading
                    && (quest_state.is_some_and(|s| s != QuestState::Done)
                        || region_quest.as_ref().is_some_and(|entry| {
                            if npc.expansion().is_some() { entry.1 != "Abgeschlossen" } else { entry.2 }
                        }))
            }
            DialogueButton::Story => npc == Npc::Juna && !garden.story.met_juna,
            DialogueButton::Harbor => !dialog.trading && matches!(npc, Npc::Bo | Npc::Fiete),
            DialogueButton::EveningMarket => {
                npc == Npc::Mara
                    && !dialog.trading
                    && evening_market::invited(&garden)
                    && (garden.evening_market.stage != MarketStage::Open
                        || evening_market::trading(&garden))
            }
            DialogueButton::Plans => {
                npc == Npc::Bo && garden.clock.market_open() && !dialog.trading
            }
            DialogueButton::WatchMap => matches!(npc, Npc::Juna | Npc::Bo) && !dialog.trading,
            DialogueButton::Close => true,
        };
        let next = if shown { Display::Flex } else { Display::None };
        if node.display != next {
            node.display = next;
        }
        let enabled = match button {
            DialogueButton::Deal => trade.as_ref().is_some_and(|t| t.2),
            DialogueButton::Quest if npc.expansion().is_some() => region_quest.as_ref().is_some_and(|entry| entry.2),
            DialogueButton::EveningMarket => {
                evening_market::can_advance(&garden) || evening_market::trading(&garden)
            }
            DialogueButton::Trade if npc == Npc::Runa => garden.clock.market_open(),
            _ => true,
        };
        focus.0 = shown && enabled;
        let color = ui::button_color(*interaction, enabled);
        if bg.0 != color {
            bg.0 = color;
        }
    }
}

pub fn order_name(npc: Npc, garden: &Garden) -> &'static str {
    npc.current_quest(garden).map_or(npc.name(), Quest::name)
}

pub fn order_target(npc: Npc, garden: &Garden) -> bool {
    npc.current_quest(garden).is_some_and(|quest| garden.town.state(quest) != QuestState::Done)
}

pub fn order_done(npc: Npc, garden: &Garden) -> bool {
    npc.quest()
        .is_some_and(|q| garden.town.state(q) == QuestState::Done)
}

pub fn next_city_order(garden: &Garden) -> Option<Npc> {
    [Npc::Mara, Npc::Fiete, Npc::Bo]
        .into_iter()
        .find(|&npc| !order_done(npc, garden))
}

pub fn order_preview(npc: Npc, garden: &Garden) -> String {
    let Some(quest) = npc.current_quest(garden) else { return npc.name().into() };
    let state = match garden.town.state(quest) {
        QuestState::Offered => "Verfügbar",
        QuestState::Accepted if quest.ready(garden) => "Abgabebereit",
        QuestState::Accepted => "Angenommen",
        QuestState::Done => "Erfüllt",
    };
    format!("{state} · {}", npc.name())
}

pub fn order_detail(npc: Npc, garden: &Garden) -> String {
    let Some(quest) = npc.current_quest(garden) else {
        return String::new();
    };
    if garden.town.state(quest) == QuestState::Done {
        return format!("{}\nAbgeschlossen · {}", quest.brief(), npc.name());
    }
    let state = match garden.town.state(quest) {
        QuestState::Offered => "Offen",
        QuestState::Accepted if quest.ready(garden) => "Abgabebereit",
        _ => "Angenommen",
    };
    let supplies = quest
        .requirements()
        .iter()
        .map(|item| item.progress(garden))
        .collect::<Vec<_>>()
        .join(" · ");
    format!("{}\n{state} · {} · {supplies}", quest.brief(), npc.name())
}

pub(crate) fn tracking(npc: Npc, garden: &Garden) -> Option<(&'static str, String, bool)> {
    let quest = npc.current_quest(garden)?;
    if garden.town.state(quest) != QuestState::Accepted {
        return None;
    }
    let materials = quest.requirements().iter()
        .map(|item| item.progress(garden))
        .collect::<Vec<_>>()
        .join(" · ");
    Some((quest.name(), materials, quest.ready(garden)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quest_requires_acceptance_and_consumes_materials_once() {
        let mut garden = Garden::default();
        let coins = garden.farm.coins;
        let quest = Quest::Chime;
        assert_eq!(Npc::Fiete.quest_mark(&garden), Some(QuestMark::Available));
        assert!(quest.advance(&mut garden).is_ok());
        assert_eq!(Npc::Fiete.quest_mark(&garden), None);
        assert!(quest.advance(&mut garden).is_err());
        garden.add_stock(Product::Shell, 4);
        assert_eq!(Npc::Fiete.quest_mark(&garden), Some(QuestMark::TurnIn));
        assert!(quest.advance(&mut garden).is_ok());
        assert_eq!(Npc::Fiete.quest_mark(&garden), None);
        assert_eq!(garden.stock(Product::Shell), 1);
        assert_eq!(garden.farm.coins, coins + 16);
        assert_eq!(garden.town.completed(), 1);
        assert!(quest.advance(&mut garden).is_err());
    }

    #[test]
    fn maras_follow_up_opens_after_the_pier_and_is_tracked_separately() {
        let mut garden = Garden::default();
        *garden.town.state_mut(Quest::Soup) = QuestState::Done;
        assert_eq!(Npc::Mara.current_quest(&garden), Some(Quest::Soup));
        assert!(!order_target(Npc::Mara, &garden));
        garden.harbor.phase = harbor::Phase::Open;
        assert_eq!(Npc::Mara.quest_mark(&garden), Some(QuestMark::Available));
        let quest = Npc::Mara.current_quest(&garden).unwrap();
        assert_eq!(quest, Quest::LongTable);
        quest.advance(&mut garden).unwrap();
        assert!(!tracking(Npc::Mara, &garden).unwrap().2);
        for need in quest.requirements() {
            if let Require::Product(product, quantity) = *need {
                garden.add_stock(product, quantity);
            }
        }
        assert!(tracking(Npc::Mara, &garden).unwrap().2);
        let mut garden = Garden::decode(&serde_json::to_vec(&garden).unwrap()).unwrap();
        quest.advance(&mut garden).unwrap();
        assert_eq!((garden.town.completed(), garden.town.total_completed()), (1, 2));
    }
}
