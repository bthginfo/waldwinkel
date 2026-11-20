use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    Controls, Notice,
    adventure::commit,
    garden::{Crop, Garden, SavePath},
    homestead::{Product, Village},
    input::Gesture,
    session::Session,
    travel::{District, FINDS},
    ui::{self, UiFonts},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Npc {
    Lina,
    Miro,
    Mara,
    Fiete,
    Bo,
    Juna,
}

impl Npc {
    pub const ALL: [Self; 6] = [
        Self::Lina,
        Self::Miro,
        Self::Mara,
        Self::Fiete,
        Self::Bo,
        Self::Juna,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Lina => "Lina",
            Self::Miro => "Miro",
            Self::Mara => "Mara",
            Self::Fiete => "Fiete",
            Self::Bo => "Bo",
            Self::Juna => "Juna",
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
        }
    }
    pub fn position(self) -> Vec2 {
        District::Town.origin()
            + match self {
                Self::Lina => Vec2::new(-5.50, -1.55),
                Self::Miro => Vec2::new(5.81, -1.50),
                Self::Mara => Vec2::new(-6.84, 1.92),
                Self::Fiete => Vec2::new(3.80, 3.36),
                Self::Bo => Vec2::new(1.41, -2.39),
                Self::Juna => Vec2::new(7.26, 2.56),
            }
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
                "Die alten Steine am Nordwestweg führen in den Farnwald. Zur Muschelbucht geht es über den südlichen Küstenpfad."
            }
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Quest {
    Soup,
    Chime,
    Fountain,
}

#[derive(Clone, Copy)]
enum Require {
    Product(Product, u32),
    Wood(u32),
    Stone(u32),
}

impl Require {
    fn available(self, garden: &Garden) -> u32 {
        match self {
            Self::Product(product, _) => garden.stock(product),
            Self::Wood(_) => garden.wood,
            Self::Stone(_) => garden.stone,
        }
    }
    fn quantity(self) -> u32 {
        match self {
            Self::Product(_, quantity) | Self::Wood(quantity) | Self::Stone(quantity) => quantity,
        }
    }
    fn progress(self, garden: &Garden) -> String {
        let name = match self {
            Self::Product(product, _) => product.name(),
            Self::Wood(_) => "Holz",
            Self::Stone(_) => "Stein",
        };
        let quantity = self.quantity();
        format!(
            "{}/{} {name}",
            self.available(garden).min(quantity),
            quantity
        )
    }
    fn take(self, g: &mut Garden) {
        match self {
            Self::Product(product, quantity) => {
                g.take_stock(product, quantity);
            }
            Self::Wood(quantity) => g.wood -= quantity,
            Self::Stone(quantity) => g.stone -= quantity,
        }
    }
}

impl Quest {
    fn name(self) -> &'static str {
        match self {
            Self::Soup => "Für Maras Küche",
            Self::Chime => "Klang der Bucht",
            Self::Fountain => "Ein Brunnen für alle",
        }
    }
    fn request(self) -> &'static str {
        match self {
            Self::Soup => "3 Pilze und 2 Haselnüsse aus dem Farnwald",
            Self::Chime => "3 Muscheln aus der Muschelbucht",
            Self::Fountain => "4 Holz und 3 Stein aus Wald, Bucht oder Mine",
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
        }
    }
    fn reward(self) -> (u32, Crop) {
        match self {
            Self::Soup => (18, Crop::Tomato),
            Self::Chime => (16, Crop::Strawberry),
            Self::Fountain => (24, Crop::Sunflower),
        }
    }

    fn ready(self, garden: &Garden) -> bool {
        self.requirements()
            .iter()
            .all(|req| req.available(garden) >= req.quantity())
    }

    fn advance(self, g: &mut Garden) -> Result<String, String> {
        match g.town.quests[self as usize] {
            QuestState::Offered => {
                g.town.quests[self as usize] = QuestState::Accepted;
                Ok("Auftrag angenommen.".into())
            }
            QuestState::Accepted if self.ready(g) => {
                for item in self.requirements() {
                    item.take(g);
                }
                let (coins, crop) = self.reward();
                g.farm.coins = g.farm.coins.saturating_add(coins);
                g.add_seeds(crop, 2);
                g.town.quests[self as usize] = QuestState::Done;
                Ok(format!("Auftrag erfüllt · +{coins} Münzen · +2 Samen"))
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
    picked: Vec<Option<u32>>,
    pub goods: [u32; 3],
}

impl Journal {
    pub fn completed(&self) -> usize {
        self.quests
            .iter()
            .filter(|&&state| state == QuestState::Done)
            .count()
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
}
#[derive(Component)]
pub struct DialogueLabel(DialogueButton);

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>) {
    commands
        .spawn((
            DialoguePane,
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
                                (DialogueButton::Trade, "Handeln", 129.),
                                (DialogueButton::Pick, "Nächste Ware", 129.),
                                (DialogueButton::Deal, "Kaufen", 129.),
                                (DialogueButton::Quest, "Annehmen", 187.),
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
            npc.quest().map(|quest| quest.advance(&mut garden))
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
    let quest = npc.quest();
    let quest_state = quest.map(|q| garden.town.quests[q as usize]);
    for (field, mut text) in &mut fields {
        let value = match field {
            DialogueText::Name => npc.name().into(),
            DialogueText::Role => npc.role().into(),
            DialogueText::Greeting if dialog.trading => {
                "Wähle eine Ware und bestätige den Handel.".into()
            }
            DialogueText::Greeting if quest_state == Some(QuestState::Done) => {
                "Danke für deine Hilfe. Schau gern wieder vorbei!".into()
            }
            DialogueText::Greeting => npc.greeting().into(),
            DialogueText::Request if dialog.trading => {
                trade.as_ref().map_or_else(String::new, |t| t.0.clone())
            }
            DialogueText::Request => {
                quest
                    .zip(quest_state)
                    .map_or_else(String::new, |(quest, state)| {
                        let (coins, crop) = quest.reward();
                        match state {
                            QuestState::Done => format!("{} · Erfüllt", quest.name()),
                            _ => format!(
                                "{}\n{}\n{coins} Münzen und 2 Samen für {} als Belohnung",
                                quest.name(),
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
            DialogueButton::Deal if npc == Npc::Miro => "Verkaufen",
            DialogueButton::Deal => "Saat kaufen",
            DialogueButton::Quest if quest_state == Some(QuestState::Accepted) => "Auftrag abgeben",
            DialogueButton::Quest => "Auftrag annehmen",
            _ => continue,
        };
        if text.0 != caption {
            text.0 = caption.into();
        }
    }
    for (button, mut node, interaction, mut bg) in &mut buttons {
        let shown = match button {
            DialogueButton::Trade => merchant.is_some() && !dialog.trading,
            DialogueButton::Pick | DialogueButton::Deal | DialogueButton::Back => dialog.trading,
            DialogueButton::Quest => {
                !dialog.trading && quest_state.is_some_and(|s| s != QuestState::Done)
            }
            DialogueButton::Close => true,
        };
        let next = if shown { Display::Flex } else { Display::None };
        if node.display != next {
            node.display = next;
        }
        let enabled = *button != DialogueButton::Deal || trade.as_ref().is_some_and(|t| t.2);
        let color = ui::button_color(*interaction, enabled);
        if bg.0 != color {
            bg.0 = color;
        }
    }
}

pub fn journal_summary(garden: &Garden) -> String {
    let statuses = [Quest::Soup, Quest::Chime, Quest::Fountain].map(|q| {
        let state = match garden.town.quests[q as usize] {
            QuestState::Offered => "offen",
            QuestState::Accepted if q.ready(garden) => "abgabebereit",
            QuestState::Accepted => "angenommen",
            QuestState::Done => "erfüllt",
        };
        format!("{} · {state}", q.name())
    });
    statuses.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quest_requires_acceptance_and_consumes_materials_once() {
        let mut garden = Garden::default();
        let coins = garden.farm.coins;
        let quest = Quest::Chime;
        assert!(quest.advance(&mut garden).is_ok());
        assert!(quest.advance(&mut garden).is_err());
        garden.add_stock(Product::Shell, 4);
        assert!(quest.advance(&mut garden).is_ok());
        assert_eq!(garden.stock(Product::Shell), 1);
        assert_eq!(garden.farm.coins, coins + 16);
        assert_eq!(garden.town.completed(), 1);
        assert!(quest.advance(&mut garden).is_err());
    }
}
