use serde::{Deserialize, Serialize};

pub const DECK_SIZE: usize = 12;
pub const LAST_LEVEL: u32 = 25;
pub const CAMP_INTERVAL: u32 = 5;
const COPY_LIMIT: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Emblem {
    Strike,
    Guard,
    Heal,
    Focus,
    Spore,
}

impl Emblem {
    pub fn texture(self) -> &'static str {
        match self {
            Self::Strike => "ui/card_strike.png",
            Self::Guard => "ui/card_guard.png",
            Self::Heal => "ui/card_heal.png",
            Self::Focus => "ui/card_focus.png",
            Self::Spore => "ui/card_spore.png",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum Card {
    TwigStrike,
    BarkShield,
    TrailRush,
    LeafPoultice,
    Scout,
    SeedShot,
    RainStep,
    ForageTea,
    ReedSnare,
    QuartzThrust,
    CopperArc,
    FlaxWrap,
    LanternWard,
    MossBloom,
    MoonSlice,
    HarborSong,
    StoneEcho,
    HearthMeal,
    BloomWard,
}

struct CardSpec {
    name: &'static str,
    cost: u8,
    damage: u8,
    guard: u8,
    healing: u8,
    draw: u8,
    spores: u8,
    piercing: bool,
}

impl CardSpec {
    const fn basic(name: &'static str, cost: u8, damage: u8, guard: u8, healing: u8) -> Self {
        Self {
            name,
            cost,
            damage,
            guard,
            healing,
            draw: 0,
            spores: 0,
            piercing: false,
        }
    }
}

impl Card {
    pub const ALL: [Self; 19] = [
        Self::TwigStrike,
        Self::BarkShield,
        Self::TrailRush,
        Self::LeafPoultice,
        Self::Scout,
        Self::SeedShot,
        Self::RainStep,
        Self::ForageTea,
        Self::ReedSnare,
        Self::QuartzThrust,
        Self::CopperArc,
        Self::FlaxWrap,
        Self::LanternWard,
        Self::MossBloom,
        Self::MoonSlice,
        Self::HarborSong,
        Self::StoneEcho,
        Self::HearthMeal,
        Self::BloomWard,
    ];

    fn spec(self) -> CardSpec {
        let base = CardSpec::basic;
        match self {
            Self::TwigStrike => base("Asthieb", 1, 7, 0, 0),
            Self::BarkShield => base("Rindenschild", 1, 0, 8, 0),
            Self::TrailRush => base("Pfadsturm", 2, 14, 2, 0),
            Self::LeafPoultice => base("Blattverband", 1, 0, 2, 6),
            Self::Scout => CardSpec {
                draw: 2,
                ..base("Spuren lesen", 1, 0, 3, 0)
            },
            Self::SeedShot => base("Samenwurf", 0, 3, 0, 0),
            Self::RainStep => base("Regenschritt", 1, 4, 7, 0),
            Self::ForageTea => CardSpec {
                draw: 1,
                ..base("Waldtee", 1, 0, 1, 8)
            },
            Self::ReedSnare => CardSpec {
                spores: 4,
                ..base("Schilfschlinge", 1, 3, 2, 0)
            },
            Self::QuartzThrust => CardSpec {
                piercing: true,
                ..base("Quarzstoß", 1, 9, 0, 0)
            },
            Self::CopperArc => base("Kupferbogen", 2, 18, 4, 0),
            Self::FlaxWrap => CardSpec {
                draw: 1,
                ..base("Flachswickel", 1, 0, 6, 4)
            },
            Self::LanternWard => base("Laternenschutz", 2, 0, 16, 3),
            Self::MossBloom => CardSpec {
                spores: 7,
                ..base("Moosblüte", 2, 8, 1, 0)
            },
            Self::MoonSlice => CardSpec {
                piercing: true,
                ..base("Mondsichel", 2, 22, 0, 0)
            },
            Self::HarborSong => CardSpec {
                draw: 3,
                ..base("Hafenlied", 0, 0, 2, 0)
            },
            Self::StoneEcho => base("Steinecho", 1, 9, 7, 0),
            Self::BloomWard => base("Blütenwacht", 1, 0, 7, 3),
            Self::HearthMeal => base("Abendmahl", 2, 0, 10, 14),
        }
    }

    pub fn name(self) -> &'static str {
        self.spec().name
    }

    pub fn cost(self) -> u8 {
        self.spec().cost
    }

    pub fn portrait(self) -> String {
        format!("ui/portrait_card_{:02}.png", self as usize)
    }

    pub fn emblem(self) -> Emblem {
        let spec = self.spec();
        if spec.spores > 0 {
            Emblem::Spore
        } else if spec.damage > 0 {
            Emblem::Strike
        } else if spec.healing > 0 {
            Emblem::Heal
        } else if spec.draw > 0 {
            Emblem::Focus
        } else {
            Emblem::Guard
        }
    }

    pub fn description(self) -> String {
        self.description_with(0)
    }

    fn description_with(self, strength: u8) -> String {
        let spec = self.spec();
        let mut pieces = Vec::new();
        if spec.damage > 0 {
            pieces.push(format!(
                "{} Schaden{}",
                spec.damage + strength,
                if spec.piercing { " durch Schild" } else { "" }
            ));
        }
        if spec.guard > 0 {
            pieces.push(format!("{} Schild", spec.guard));
        }
        if spec.healing > 0 {
            pieces.push(format!("{} Heilung", spec.healing));
        }
        if spec.spores > 0 {
            pieces.push(format!(
                "{} Sporenschaden über zwei Runden",
                spec.spores * 2
            ));
        }
        if spec.draw > 0 {
            pieces.push(format!("{} ziehen", spec.draw));
        }
        match self {
            Self::BloomWard => pieces.push("Mit vorhandenem Schild 1 ziehen".into()),
            Self::Scout => pieces.push("Nächster Treffer +4".into()),
            Self::SeedShot => pieces.push("Verstärkt aktive Sporen".into()),
            _ => {}
        }
        pieces.join(" · ")
    }
}

const START_DECK: [Card; DECK_SIZE] = [
    Card::TwigStrike,
    Card::TwigStrike,
    Card::BarkShield,
    Card::BarkShield,
    Card::TrailRush,
    Card::TrailRush,
    Card::LeafPoultice,
    Card::LeafPoultice,
    Card::Scout,
    Card::Scout,
    Card::SeedShot,
    Card::SeedShot,
];

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum Gear {
    TrailStick,
    CopperSickle,
    SilverGuard,
    CrystalBlade,
}

impl Gear {
    pub const ALL: [Self; 4] = [
        Self::TrailStick,
        Self::CopperSickle,
        Self::SilverGuard,
        Self::CrystalBlade,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::TrailStick => "Wanderstab",
            Self::CopperSickle => "Kupfersichel",
            Self::SilverGuard => "Silbergriff",
            Self::CrystalBlade => "Kristallklinge",
        }
    }

    pub fn damage(self) -> u8 {
        [0, 1, 2, 4][self as usize]
    }

    pub fn resistance(self) -> u8 {
        [0, 0, 1, 1][self as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Talent {
    Heart,
    Shell,
    Focus,
}

impl Talent {
    pub const ALL: [Self; 3] = [Self::Heart, Self::Shell, Self::Focus];
    pub fn name(self) -> &'static str {
        match self {
            Self::Heart => "Wanderherz",
            Self::Shell => "Schutzfell",
            Self::Focus => "Wacher Blick",
        }
    }
    pub fn text(self) -> &'static str {
        match self {
            Self::Heart => "+5 maximale Lebenspunkte je Rang",
            Self::Shell => "+1 Widerstand gegen gegnerische Angriffe je Rang",
            Self::Focus => "Mehr Karten zu Rundenbeginn auf Rang 1, mehr Kraft auf Rang 2",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loot {
    MossDown,
    ClaySeal,
    MoonDust,
}

impl Loot {
    pub const ALL: [Self; 3] = [Self::MossDown, Self::ClaySeal, Self::MoonDust];

    pub fn name(self) -> &'static str {
        match self {
            Self::MossDown => "Moosflaum",
            Self::ClaySeal => "Tonsiegel",
            Self::MoonDust => "Mondstaub",
        }
    }

    pub fn value(self) -> u32 {
        [7, 10, 15][self as usize]
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Progress {
    #[serde(deserialize_with = "read_collection")]
    cards: [bool; Card::ALL.len()],
    deck: [Card; DECK_SIZE],
    pub gear: Gear,
    pub gear_owned: [bool; Gear::ALL.len()],
    pub ranks: [u8; Talent::ALL.len()],
    pub points: u8,
    pub cleared: u32,
    pub hp: u16,
    pub trophies: [u32; Loot::ALL.len()],
    pub style: Style,
}

fn read_collection<'de, D: serde::Deserializer<'de>>(reader: D) -> Result<[bool; Card::ALL.len()], D::Error> {
    let cards = Vec::<bool>::deserialize(reader)?;
    if !(18..=Card::ALL.len()).contains(&cards.len()) {
        return Err(serde::de::Error::custom("Ungültige Kartensammlung"));
    }
    let mut known = [false; Card::ALL.len()];
    known[..cards.len()].copy_from_slice(&cards);
    Ok(known)
}

impl Default for Progress {
    fn default() -> Self {
        let cards = std::array::from_fn(|index| index < 6);
        Self {
            cards,
            deck: START_DECK,
            gear: Gear::TrailStick,
            gear_owned: [true, false, false, false],
            ranks: [0; 3],
            points: 0,
            cleared: 0,
            hp: 34,
            trophies: [0; 3],
            style: Style::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub enum Style {
    #[default]
    Wanderer,
    Warden,
    Trail,
    Hearth,
}

impl Style {
    pub const ALL: [Self; 3] = [Self::Warden, Self::Trail, Self::Hearth];

    pub fn name(self) -> &'static str {
        match self {
            Self::Wanderer => "Wanderer",
            Self::Warden => "Rindenwacht",
            Self::Trail => "Windspur",
            Self::Hearth => "Herdfeuer",
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            Self::Wanderer => "Dein vertrauter Wanderstab.",
            Self::Warden => {
                "Der erste Schild im Zug gibt +3 Schutz. Hält er einen Angriff vollständig auf, konterst du mit 4 Schaden. Passt zu Rindenschild und Regenschritt."
            }
            Self::Trail => {
                "Deine dritte Karte im Zug zieht eine weitere Karte. Samenwurf und Spuren lesen halten deine Kartenfolge in Gang."
            }
            Self::Hearth => {
                "Dein Proviant gibt zusätzlich +1 Kraft und +4 Schild. Bereite daheim ein Gericht vor."
            }
        }
    }
}

impl Progress {
    pub fn knows(&self, card: Card) -> bool {
        self.cards[card as usize]
    }

    pub fn learn(&mut self, card: Card) -> bool {
        !std::mem::replace(&mut self.cards[card as usize], true)
    }

    pub fn deck(&self) -> [Card; DECK_SIZE] {
        self.deck
    }

    pub fn save_deck(&mut self, cards: &[Card]) -> Result<(), &'static str> {
        let Ok(next) = <&[Card; DECK_SIZE]>::try_from(cards) else {
            return Err("Für dein Kampfdeck brauchst du zwölf Karten.");
        };
        let mut count = [0; Card::ALL.len()];
        for &card in next {
            if !self.knows(card) {
                return Err("Diese Karte fehlt noch in deiner Sammlung.");
            }
            count[card as usize] += 1;
            if count[card as usize] > COPY_LIMIT {
                return Err("Jede Karte passt höchstens zweimal ins Deck.");
            }
        }
        self.deck = *next;
        Ok(())
    }

    pub fn max_hp(&self) -> u16 {
        34 + u16::from(self.ranks[Talent::Heart as usize]) * 5
    }

    pub fn rest(&mut self) {
        self.hp = self.max_hp();
    }

    pub fn camp(&self) -> u32 {
        self.cleared / CAMP_INTERVAL * CAMP_INTERVAL
    }

    pub fn train(&mut self, talent: Talent) -> bool {
        let index = talent as usize;
        let limit = if talent == Talent::Focus { 2 } else { 4 };
        let cost = self.ranks[index] + 1;
        if self.ranks[index] >= limit || self.points < cost {
            return false;
        }
        self.points -= cost;
        self.ranks[index] += 1;
        if talent == Talent::Heart {
            self.hp += 5;
        }
        true
    }

    pub fn train_cost(&self, talent: Talent) -> Option<u8> {
        let rank = self.ranks[talent as usize];
        let limit = if talent == Talent::Focus { 2 } else { 4 };
        (rank < limit).then_some(rank.saturating_add(1))
    }

    pub fn win(&mut self, level: u32, hp: u16) -> Option<(Loot, Option<Card>)> {
        if level != self.cleared.saturating_add(1) || level > LAST_LEVEL {
            return None;
        }
        self.hp = hp.min(self.max_hp()).max(1);
        self.cleared = level;
        self.points = self
            .points
            .saturating_add(if level % CAMP_INTERVAL == 0 { 3 } else { 1 });
        let prize = Enemy::at(level).loot();
        self.trophies[prize as usize] += 1;
        let learned = match level {
            1 => Some(Card::RainStep),
            3 => Some(Card::ReedSnare),
            5 => Some(Card::LanternWard),
            9 => Some(Card::MossBloom),
            15 => Some(Card::StoneEcho),
            20 => Some(Card::HarborSong),
            25 => Some(Card::MoonSlice),
            _ => None,
        }
        .filter(|&card| self.learn(card));
        Some((prize, learned))
    }

    pub fn normalize(&mut self) {
        self.cleared = self.cleared.min(LAST_LEVEL);
        self.ranks[0] = self.ranks[0].min(4);
        self.ranks[1] = self.ranks[1].min(4);
        self.ranks[2] = self.ranks[2].min(2);
        self.hp = self.hp.min(self.max_hp()).max(1);
        for card in START_DECK {
            self.cards[card as usize] = true;
        }
        let deck = self.deck;
        if self.save_deck(&deck).is_err() {
            self.deck = START_DECK;
        }
        self.gear_owned[0] = true;
        if !self.gear_owned[self.gear as usize] {
            self.gear = Gear::TrailStick;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Enemy {
    Mosskin,
    KilnGuard,
    MistOwl,
    MossKeeper,
    KilnKeeper,
    MistKeeper,
    RootKeeper,
    BellKeeper,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chapter {
    Moss,
    Clay,
    Mist,
    Root,
    Bell,
}

impl Chapter {
    pub fn at(level: u32) -> Self {
        match level {
            0..=5 => Self::Moss,
            6..=10 => Self::Clay,
            11..=15 => Self::Mist,
            16..=20 => Self::Root,
            _ => Self::Bell,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Moss => "Moosgänge",
            Self::Clay => "Tonwerk",
            Self::Mist => "Nebelkammern",
            Self::Root => "Wurzelsaal",
            Self::Bell => "Glockenhalle",
        }
    }

    pub fn floor(self, variation: usize) -> &'static str {
        match self {
            Self::Moss => ["dusk_floor_0", "dusk_floor_1", "dusk_floor_2"][variation % 3],
            Self::Clay => "dusk_floor_clay",
            Self::Mist => "dusk_floor_mist",
            Self::Root => "dusk_floor_root",
            Self::Bell => "dusk_floor_bell",
        }
    }

    pub fn wall_feature(self) -> &'static str {
        match self {
            Self::Moss => "dusk_wall",
            Self::Clay => "dusk_wall_clay",
            Self::Mist => "dusk_wall_mist",
            Self::Root => "dusk_wall_root",
            Self::Bell => "dusk_wall_bell",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intent {
    Attack(u16),
    Flurry(u16),
    Brace(u16),
    Mend(u16),
    Charge { guard: u16, damage: u16 },
    Stagger,
}

impl Enemy {
    pub fn at(level: u32) -> Self {
        if level % CAMP_INTERVAL == 0 {
            match Chapter::at(level) {
                Chapter::Moss => Self::MossKeeper,
                Chapter::Clay => Self::KilnKeeper,
                Chapter::Mist => Self::MistKeeper,
                Chapter::Root => Self::RootKeeper,
                Chapter::Bell => Self::BellKeeper,
            }
        } else {
            [Self::Mosskin, Self::KilnGuard, Self::MistOwl][((level - 1) * 7 % 3) as usize]
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Mosskin => "Mooshüter",
            Self::KilnGuard => "Tonwächter",
            Self::MistOwl => "Nebeleule",
            Self::MossKeeper => "Moosglocke",
            Self::KilnKeeper => "Ofenglocke",
            Self::MistKeeper => "Nebelglocke",
            Self::RootKeeper => "Wurzelglocke",
            Self::BellKeeper => "Hüter der ersten Glocke",
        }
    }

    pub fn provision(self) -> Ration {
        match self {
            Self::Mosskin | Self::MossKeeper => Ration::Compote,
            Self::KilnGuard | Self::KilnKeeper | Self::BellKeeper => Ration::Stew,
            Self::MistOwl | Self::MistKeeper => Ration::Jam,
            Self::RootKeeper => Ration::Soup,
        }
    }

    pub fn provision_hint(self) -> String {
        format!(
            "{} · Vorteil mit {}",
            self.name(),
            self.provision().dish().name()
        )
    }

    pub fn model(self) -> &'static str {
        match self {
            Self::Mosskin => "duel_mosskin",
            Self::KilnGuard => "duel_kiln",
            Self::MistOwl => "duel_owl",
            Self::MossKeeper => "duel_keeper",
            Self::KilnKeeper => "duel_keeper_clay",
            Self::MistKeeper => "duel_keeper_mist",
            Self::RootKeeper => "duel_keeper_root",
            Self::BellKeeper => "duel_keeper_bell",
        }
    }

    pub fn loot(self) -> Loot {
        match self {
            Self::Mosskin => Loot::MossDown,
            Self::KilnGuard => Loot::ClaySeal,
            _ => Loot::MoonDust,
        }
    }

    fn hp(self, level: u32) -> u16 {
        let base = 14 + level as u16 * 2;
        base + match self {
            Self::KilnGuard => 4,
            Self::Mosskin | Self::MistOwl => 0,
            _ => 12,
        }
    }

    fn intent(self, level: u32, round: u16) -> Intent {
        let level = level as u16;
        let hit = 4 + level / 2;
        let guard = 4 + level / 3;
        let heal = 5 + level / 4;
        let charge = |armor, power| Intent::Charge {
            guard: armor,
            damage: power,
        };
        match self {
            Self::Mosskin => match round % 4 {
                2 => Intent::Mend(heal),
                0 => Intent::Brace(guard),
                _ => Intent::Attack(hit),
            },
            Self::KilnGuard => match round % 4 {
                1 => Intent::Brace(guard + 3),
                3 => charge(guard, hit + 5),
                _ => Intent::Attack(hit + 2),
            },
            Self::MistOwl => match round % 4 {
                0 => Intent::Brace(guard),
                2 => Intent::Attack(hit + 3),
                _ => Intent::Flurry((hit + 3) / 2),
            },
            Self::MossKeeper => match round % 4 {
                1 => Intent::Attack(hit + 3),
                2 => Intent::Mend(heal + 3),
                3 => Intent::Brace(guard + 1),
                _ => Intent::Flurry((hit + 5) / 2),
            },
            Self::KilnKeeper => match round % 4 {
                1 => Intent::Brace(guard + 4),
                2 => charge(guard + 2, hit + 8),
                _ => Intent::Attack(hit + 3),
            },
            Self::MistKeeper => match round % 5 {
                1 | 4 => Intent::Flurry((hit + 6) / 2),
                2 => Intent::Brace(guard),
                0 => Intent::Mend(heal),
                _ => charge(guard, hit + 7),
            },
            Self::RootKeeper => match round % 5 {
                1 => Intent::Attack(hit + 5),
                2 | 0 => Intent::Mend(heal + 5),
                3 => Intent::Flurry((hit + 6) / 2),
                _ => charge(guard + 1, hit + 7),
            },
            Self::BellKeeper => match round % 4 {
                1 => charge(guard + 4, hit + 11),
                3 => Intent::Flurry((hit + 8) / 2),
                0 => Intent::Mend(heal + 3),
                _ => Intent::Attack(hit + 5),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Running,
    Won,
    Lost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ration {
    Soup,
    Stew,
    Jam,
    Compote,
    Broth,
}

impl Ration {
    pub const MARKET: [Self; 4] = [Self::Soup, Self::Stew, Self::Jam, Self::Compote];
    pub const ALL: [Self; 5] = [Self::Soup, Self::Stew, Self::Jam, Self::Compote, Self::Broth];

    pub fn dish(self) -> crate::processing::Dish {
        use crate::processing::Dish;
        match self {
            Self::Soup => Dish::CarrotSoup,
            Self::Stew => Dish::TomatoStew,
            Self::Jam => Dish::BerryJam,
            Self::Compote => Dish::StrawberryCompote,
            Self::Broth => Dish::HarborBroth,
        }
    }

    pub fn effect(self) -> &'static str {
        match self {
            Self::Soup => "+10 Leben",
            Self::Stew => "+1 Kraft und +6 Schild",
            Self::Jam => "+6 Schild und ziehe 1 Karte",
            Self::Compote => "+4 Leben und ziehe 2 Karten",
            Self::Broth => "+6 Leben und +4 Schild",
        }
    }

    pub fn matchup(self, enemy: Enemy) -> Option<&'static str> {
        (self == enemy.provision()).then_some(match self {
            Self::Soup => "+5 Leben, +3 Schild",
            Self::Stew => "Treffer bricht bis zu 8 Rüstung",
            Self::Jam => "+6 Schild extra",
            Self::Compote => "+1 Kraft extra",
            Self::Broth => return None,
        })
    }

    pub fn best_against(self) -> &'static str {
        match self {
            Self::Soup => "Wurzelglocke · mehr Leben und Schild",
            Self::Stew => "Tonwächter, Ofen- und erste Glocke · Rüstung brechen",
            Self::Jam => "Nebeleule und Nebelglocke · mehr Schild",
            Self::Compote => "Mooshüter und Moosglocke · mehr Kraft",
            Self::Broth => "Stärkt und schützt dich in jeder Wacht.",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CardImpact {
    pub damage: u16,
    pub armor: u16,
    pub healed: u16,
    pub shield: u16,
}

pub struct Battle {
    pub level: u32,
    pub enemy: Enemy,
    pub enemy_hp: u16,
    pub enemy_max: u16,
    pub enemy_guard: u16,
    pub hp: u16,
    pub max_hp: u16,
    pub guard: u16,
    pub energy: u8,
    pub round: u16,
    pub hand: Vec<Card>,
    pub outcome: Outcome,
    pub message: String,
    pub retreat_pending: bool,
    pub trial: Option<crate::waywatch::Trial>,
    pub training: bool,
    pub pack_open: bool,
    pub ration_used: bool,
    pub style: Style,
    played_this_turn: u8,
    warden_ready: bool,
    resistance: u8,
    strength: u8,
    hand_size: u8,
    energy_limit: u8,
    drawing: Vec<Card>,
    used: Vec<Card>,
    spores: [u16; 2],
    charged: Option<u16>,
    armor_break: bool,
    marked: bool,
    rng: u64,
}

impl Battle {
    pub(crate) fn hand_controls(&self) -> bool {
        self.outcome == Outcome::Running && !self.pack_open && !self.retreat_pending
    }

    pub fn new(level: u32, progress: &Progress) -> Self {
        Self::encounter(level, Enemy::at(level), progress)
    }

    pub fn new_trial(trial: crate::waywatch::Trial, progress: &Progress) -> Self {
        let mut battle = Self::encounter(trial.level(), trial.enemy(), progress);
        battle.hp = battle.max_hp;
        battle.trial = Some(trial);
        if trial.watch == crate::waywatch::Watch::Kiln {
            battle.enemy_guard = 3;
        }
        battle
    }

    pub fn training() -> Self {
        let mut battle = Self::encounter(1, Enemy::Mosskin, &Progress::default());
        battle.training = true;
        battle.enemy_max = 31;
        battle.enemy_hp = battle.enemy_max;
        battle.hand = vec![Card::BarkShield, Card::TwigStrike, Card::TrailRush, Card::SeedShot];
        battle.drawing = vec![Card::LeafPoultice, Card::TrailRush, Card::BarkShield,
                              Card::TwigStrike, Card::SeedShot];
        battle.message = "Der Strohwächter greift für 8 an. Spiele Rindenschild und Asthieb, dann beende den Zug.".into();
        battle
    }

    fn encounter(level: u32, foe: Enemy, progress: &Progress) -> Self {
        let focus = progress.ranks[Talent::Focus as usize];
        let mut battle = Self {
            level,
            enemy: foe,
            enemy_hp: foe.hp(level),
            enemy_max: foe.hp(level),
            enemy_guard: 0,
            hp: progress.hp,
            max_hp: progress.max_hp(),
            guard: 0,
            energy: 3 + u8::from(focus >= 2),
            round: 1,
            hand: Vec::new(),
            outcome: Outcome::Running,
            message: String::new(),
            retreat_pending: false,
            trial: None,
            training: false,
            pack_open: false,
            ration_used: false,
            style: progress.style,
            played_this_turn: 0,
            warden_ready: true,
            resistance: progress.ranks[Talent::Shell as usize] + progress.gear.resistance(),
            strength: progress.gear.damage(),
            hand_size: 5 + u8::from(focus >= 1),
            energy_limit: 3 + u8::from(focus >= 2),
            drawing: progress.deck.to_vec(),
            used: Vec::new(),
            spores: [0; 2],
            charged: None,
            armor_break: false,
            marked: false,
            rng: u64::from(level) * 0x85ebca77 + 0xc2b2ae35,
        };
        shuffle(&mut battle.drawing, &mut battle.rng);
        battle.draw(battle.hand_size);
        battle
    }

    pub fn eat(&mut self, ration: Ration) -> bool {
        if self.ration_used || self.outcome != Outcome::Running || self.retreat_pending {
            return false;
        }
        self.ration_used = true;
        self.pack_open = false;
        let matchup = ration.matchup(self.enemy);
        let (heal, guard, power, draw) = match (ration, matchup.is_some()) {
            (Ration::Soup, matched) => (10 + 5 * u16::from(matched), 3 * u16::from(matched), 0, 0),
            (Ration::Stew, matched) => {
                self.armor_break = matched;
                (0, 6, 1, 0)
            }
            (Ration::Jam, matched) => (0, 6 + 6 * u16::from(matched), 0, 1),
            (Ration::Compote, matched) => (4, 0, u8::from(matched), 2),
            (Ration::Broth, _) => (6, 4, 0, 0),
        };
        self.hp = self.hp.saturating_add(heal).min(self.max_hp);
        self.guard = self.guard.saturating_add(guard);
        self.energy = self.energy.saturating_add(power);
        self.draw(draw);
        self.message = format!("{} · {}", ration.dish().name(), ration.effect());
        if let Some(bonus) = matchup {
            self.message.push_str(&format!(" · Vorteil · {bonus}"));
        }
        if self.style == Style::Hearth {
            self.energy = self.energy.saturating_add(1);
            self.guard = self.guard.saturating_add(4);
            self.message.push_str(" · Herdfeuer +1 Kraft, +4 Schild");
        }
        true
    }

    pub fn intent(&self) -> Intent {
        if self.training {
            return Intent::Attack(8);
        }
        if let Some(damage) = self.charged {
            return if self.enemy_guard == 0 {
                Intent::Stagger
            } else {
                Intent::Attack(damage)
            };
        }
        if let Some(trial) = self.trial.filter(|trial| trial.rank == 0) {
            return trial.story_intent(self.round);
        }
        self.enemy.intent(self.level, self.round)
    }

    pub fn incoming(&self, intent: Intent) -> u16 {
        let (raw, hits) = match intent {
            Intent::Attack(raw) => (raw, 1),
            Intent::Flurry(raw) => (raw, 2),
            _ => return 0,
        };
        (raw.saturating_sub(u16::from(self.resistance)) * hits).saturating_sub(self.guard)
    }

    pub fn energy_limit(&self) -> u8 {
        self.energy_limit
    }

    pub fn piles(&self) -> (usize, usize) {
        (self.drawing.len(), self.used.len())
    }

    pub(crate) fn impact(&self, card: Card) -> CardImpact {
        let spec = card.spec();
        let warden = self.style == Style::Warden && self.warden_ready && spec.guard > 0;
        let mut result = CardImpact {
            shield: u16::from(spec.guard) + 3 * u16::from(warden),
            healed: u16::from(spec.healing).min(self.max_hp.saturating_sub(self.hp)),
            ..Default::default()
        };
        if spec.damage > 0 {
            let broken = if self.armor_break { self.enemy_guard.min(8) } else { 0 };
            let power = u16::from(spec.damage) + u16::from(self.strength) + 4 * u16::from(self.marked);
            let absorbed = if spec.piercing { 0 } else { (self.enemy_guard - broken).min(power) };
            result.armor = broken + absorbed;
            result.damage = (power - absorbed).min(self.enemy_hp);
        }
        result
    }

    pub fn forecast(&self, card: Card) -> String {
        if card.cost() > self.energy {
            return format!("{} · Noch {} Kraft nötig.", card.name(), card.cost() - self.energy);
        }
        let hit = self.impact(card);
        let mut lines = Vec::new();
        if card.spec().damage > 0 {
            lines.push(format!("Hüter −{} Leben", hit.damage));
            if hit.armor > 0 { lines.push(format!("−{} Rüstung", hit.armor)); }
        }
        if hit.healed > 0 { lines.push(format!("Tanuki +{} Leben", hit.healed)); }
        if hit.shield > 0 {
            lines.push(format!("+{} Schild", hit.shield));
        }
        let incoming = self.incoming(self.intent());
        let spores = self.spores[0].saturating_add(u16::from(card.spec().spores))
            .saturating_add(if card == Card::SeedShot && self.spores[0] > 0 { 2 } else { 0 });
        let interrupts = self.charged.is_some() && hit.armor == self.enemy_guard;
        if incoming > 0 {
            let counter = if self.enemy_hp.saturating_sub(hit.damage) <= spores || interrupts {
                0
            } else {
                incoming.saturating_sub(hit.shield)
            };
            if counter != incoming {
                lines.push(format!("Gegentreffer {incoming} → {counter}"));
            }
        }
        if card == Card::BloomWard && self.guard > 0 { lines.push("1 Karte ziehen".into()); }
        format!("{} · {}", card.name(), lines.join(" · "))
    }

    pub fn card_description(&self, card: Card) -> String {
        card.description_with(self.strength)
    }

    pub fn conditions(&self) -> String {
        match (self.marked, self.spores[0]) {
            (true, 0) => "Spur erkannt · Nächster Treffer +4".into(),
            (true, spores) => format!("Nächster Treffer +4 · {spores} Sporen vor dem Gegenzug"),
            (false, spores) if spores > 0 => format!("{spores} Sporen vor dem Gegenzug"),
            _ => String::new(),
        }
    }

    fn draw(&mut self, count: u8) {
        for _ in 0..count {
            if self.hand.len() >= 8 {
                break;
            }
            if self.drawing.is_empty() {
                if self.used.is_empty() {
                    break;
                }
                std::mem::swap(&mut self.drawing, &mut self.used);
                shuffle(&mut self.drawing, &mut self.rng);
            }
            self.hand.push(self.drawing.pop().unwrap());
        }
    }

    pub fn play(&mut self, index: usize) -> bool {
        let Some(&card) = self.hand.get(index) else {
            return false;
        };
        if self.outcome != Outcome::Running {
            return false;
        }
        let spec = card.spec();
        if spec.cost > self.energy {
            self.message = "Für diese Karte fehlt dir Kraft.".into();
            return false;
        }
        self.energy -= spec.cost;
        self.hand.remove(index);
        let (old_hp, old_enemy_hp, old_enemy_guard) = (self.hp, self.enemy_hp, self.enemy_guard);
        let sheltered = card == Card::BloomWard && self.guard > 0;
        let impact = self.impact(card);
        let shield = impact.shield;
        if spec.guard > 0 && self.style == Style::Warden {
            self.warden_ready = false;
        }
        self.guard = self.guard.saturating_add(shield);
        self.hp += impact.healed;
        let marked_hit = spec.damage > 0 && std::mem::take(&mut self.marked);
        if spec.damage > 0 && self.enemy_guard > 0 {
            self.armor_break = false;
        }
        self.enemy_guard -= impact.armor;
        self.enemy_hp -= impact.damage;
        for spores in &mut self.spores {
            *spores = spores.saturating_add(u16::from(spec.spores));
        }
        self.draw(spec.draw + u8::from(sheltered));
        self.used.push(card);
        self.played_this_turn = self.played_this_turn.saturating_add(1);
        let mut effects = Vec::new();
        if sheltered { effects.push("Blütenwacht · eine Karte ziehen".into()); }
        if card == Card::Scout {
            self.marked = true;
            effects.push("Spur erkannt · nächster Treffer +4".into());
        }
        if marked_hit {
            effects.push("Spur genutzt".into());
        }
        if card == Card::SeedShot && self.spores[0] > 0 {
            self.spores[0] = self.spores[0].saturating_add(2);
            effects.push("Sporen keimen · +2".into());
        }
        if self.style == Style::Trail && self.played_this_turn == 3 {
            self.draw(1);
            effects.push("Windspur · eine Karte ziehen".into());
        }
        if self.enemy_hp < old_enemy_hp {
            effects.push(format!(
                "−{} Leben beim Hüter",
                old_enemy_hp - self.enemy_hp
            ));
        }
        if self.enemy_guard < old_enemy_guard {
            effects.push(format!("−{} Rüstung", old_enemy_guard - self.enemy_guard));
        }
        if self.hp > old_hp {
            effects.push(format!("+{} Leben", self.hp - old_hp));
        }
        if shield > 0 {
            effects.push(format!("+{shield} Schild"));
        }
        if spec.spores > 0 {
            effects.push(format!("{} Sporen pro Zug", spec.spores));
        }
        self.message = if effects.is_empty() {
            format!("{} gespielt", spec.name)
        } else {
            format!("{} · {}", spec.name, effects.join(" · "))
        };
        if self.enemy_hp == 0 {
            self.outcome = Outcome::Won;
        }
        true
    }

    pub fn end_turn(&mut self) {
        if self.outcome != Outcome::Running {
            return;
        }
        self.enemy_hp = self.enemy_hp.saturating_sub(self.spores[0]);
        self.spores = [self.spores[1], 0];
        if self.enemy_hp == 0 {
            self.outcome = Outcome::Won;
            return;
        }
        let intent = self.intent();
        self.charged = None;
        self.message = match intent {
            Intent::Attack(_) | Intent::Flurry(_) => {
                let lost = self.incoming(intent);
                self.hp = self.hp.saturating_sub(lost);
                let name = if self.training { "Der Strohwächter" } else { self.enemy.name() };
                if lost == 0 {
                    format!("{name} greift an · Du wehrst den Angriff ab.")
                } else {
                    format!("{name} greift an · −{lost} Leben")
                }
            }
            Intent::Brace(shield) => {
                self.enemy_guard = shield;
                format!("{} baut {} Schild auf.", self.enemy.name(), shield)
            }
            Intent::Mend(amount) => {
                let restored = amount.min(self.enemy_max - self.enemy_hp);
                self.enemy_hp += restored;
                format!("{} sammelt Kräfte · +{restored} Leben", self.enemy.name())
            }
            Intent::Charge { guard, damage } => {
                self.enemy_guard = self.enemy_guard.max(guard);
                self.charged = Some(damage);
                format!(
                    "Aufgeladen · Brich {} Rüstung, um den Schlag zu stoppen.",
                    self.enemy_guard
                )
            }
            Intent::Stagger => "Die gebrochene Rüstung unterbricht den Schlag.".into(),
        };
        if self.style == Style::Warden
            && self.guard > 0
            && matches!(intent, Intent::Attack(_) | Intent::Flurry(_))
            && self.incoming(intent) == 0
        {
            self.enemy_hp = self.enemy_hp.saturating_sub(4);
            self.message.push_str(" · Rindenwacht kontert für 4");
            if self.enemy_hp == 0 {
                self.outcome = Outcome::Won;
                return;
            }
        }
        use crate::waywatch::Watch;
        let watch = self.trial.map(|trial| trial.watch);
        if watch == Some(Watch::Fern) && self.guard == 0 && self.hp > 0 {
            self.hp = self.hp.saturating_sub(2);
            self.message.push_str(" · Dornen −2 Leben");
        }
        if self.hp == 0 {
            self.outcome = Outcome::Lost;
            return;
        }
        self.used.append(&mut self.hand);
        self.round = self.round.saturating_add(1);
        self.played_this_turn = 0;
        self.warden_ready = true;
        self.guard = 0;
        self.energy = self.energy_limit;
        if watch == Some(Watch::Kiln) && self.round % 2 == 1 && self.charged.is_none() {
            self.enemy_guard = self.enemy_guard.saturating_add(3);
            self.message.push_str(" · Kupferhaut +3 Rüstung");
        }
        if watch == Some(Watch::Fen) && self.round % 3 == 0 {
            self.energy -= 1;
            self.message.push_str(" · Nebel −1 Kraft");
        }
        self.draw(self.hand_size);
    }
}

#[cfg(test)]
#[test]
fn each_style_rewards_its_own_tactic_once_per_trigger() {
    let mut progress = Progress::default();
    progress.style = Style::Warden;
    let mut warden = Battle::new(1, &progress);
    warden.hand = vec![Card::BarkShield, Card::BarkShield];
    warden.enemy_guard = 3;
    warden.charged = Some(10);
    assert!(warden.play(0) && warden.play(0));
    assert_eq!(warden.guard, 19);
    let enemy = warden.enemy_hp;
    warden.end_turn();
    assert_eq!(warden.enemy_hp, enemy - 4);

    progress.style = Style::Trail;
    let mut trail = Battle::new(1, &progress);
    trail.hand = vec![Card::SeedShot, Card::SeedShot, Card::TwigStrike];
    assert!(trail.play(0) && trail.play(0) && trail.play(0));
    assert_eq!(trail.hand.len(), 1);
    assert!(trail.message.contains("Windspur"));

    progress.style = Style::Hearth;
    let mut hearth = Battle::new(1, &progress);
    assert!(hearth.eat(Ration::Soup));
    assert_eq!((hearth.energy, hearth.guard), (4, 4));
    assert!(!hearth.eat(Ration::Jam));
}

#[cfg(test)]
#[test]
fn matching_provisions_help_against_their_intended_opponents() {
    let progress = Progress::default();
    let mut kiln = Battle::encounter(7, Enemy::KilnGuard, &progress);
    kiln.hand = vec![Card::TwigStrike; 3];
    assert!(kiln.eat(Ration::Stew));
    assert!(kiln.play(0));
    assert!(kiln.armor_break);
    kiln.enemy_guard = 8;
    assert!(kiln.play(0));
    assert_eq!(kiln.enemy_guard, 0);
    assert!(!kiln.armor_break);
    assert!(!kiln.eat(Ration::Stew));

    let mut mist = Battle::encounter(7, Enemy::MistOwl, &progress);
    let mut moss = Battle::encounter(7, Enemy::Mosskin, &progress);
    assert!(mist.eat(Ration::Jam));
    assert!(moss.eat(Ration::Jam));
    assert_eq!((mist.guard, moss.guard), (12, 6));
}

fn shuffle(items: &mut [Card], rng: &mut u64) {
    for index in (1..items.len()).rev() {
        *rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let next = ((*rng >> 32) % (index + 1) as u64) as usize;
        items.swap(index, next);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tracked_enemy_and_sown_spores_reward_play_order() {
        let mut battle = Battle::new(1, &Progress::default());
        battle.hand = vec![Card::Scout, Card::TwigStrike, Card::ReedSnare, Card::SeedShot];
        assert!(battle.play(0));
        let before = battle.enemy_hp;
        assert!(battle.conditions().contains("+4"));
        assert!(battle.play(0));
        assert_eq!(before-battle.enemy_hp,11);
        assert!(battle.play(0));
        assert_eq!(battle.spores,[4,4]);
        assert!(battle.play(0));
        assert_eq!(battle.spores,[6,4]);
        assert!(!battle.conditions().contains("+4"));
    }

    #[test]
    fn a_deck_needs_twelve_owned_cards_and_at_most_two_copies() {
        let mut p = Progress::default();
        assert!(p.save_deck(&START_DECK[..11]).is_err());
        assert!(p.save_deck(&[Card::TwigStrike; DECK_SIZE]).is_err());
        let mut choices = START_DECK;
        choices[0] = Card::QuartzThrust;
        assert!(p.save_deck(&choices).is_err());
        p.learn(Card::QuartzThrust);
        assert!(p.save_deck(&choices).is_ok());
        assert_eq!(p.deck()[0], Card::QuartzThrust);
    }

    #[test]
    fn a_won_level_cannot_be_rewarded_twice_and_camps_unlock_every_five() {
        let mut p = Progress::default();
        assert!(p.win(2, 25).is_none());
        for level in 1..=5 {
            assert!(p.win(level, 25).is_some());
            assert!(p.win(level, 25).is_none());
        }
        assert_eq!((p.cleared, p.camp(), p.points), (5, 5, 7));
        assert!(p.knows(Card::LanternWard));
        assert!(p.train(Talent::Heart));
        assert_eq!(p.max_hp(), 39);
    }

    #[test]
    fn the_farnwacht_can_be_won_with_the_first_deck() {
        use crate::waywatch::{Trial, Watch};
        let mut battle = Battle::new_trial(
            Trial {
                watch: Watch::Fern,
                rank: 0,
            },
            &Progress::default(),
        );
        for _ in 0..12 {
            if battle.outcome != Outcome::Running {
                break;
            }
            for _ in 0..8 {
                let Some(index) = battle
                    .hand
                    .iter()
                    .enumerate()
                    .filter(|(_, card)| card.cost() <= battle.energy)
                    .max_by_key(|(_, card)| {
                        let spec = card.spec();
                        let block =
                            if battle.guard == 0 && (battle.round % 4 != 2 || battle.hp <= 20) {
                                spec.guard
                            } else {
                                0
                            };
                        u16::from(spec.damage) * 4
                            + u16::from(block) * 2
                            + u16::from(spec.healing) * (u16::from(battle.hp < 21) * 2)
                    })
                    .map(|(i, _)| i)
                else {
                    break;
                };
                if !battle.play(index) || battle.outcome != Outcome::Running {
                    break;
                }
            }
            battle.end_turn();
        }
        assert_eq!(battle.outcome, Outcome::Won);
    }

    #[test]
    fn armour_terrain_and_provisions_change_turn_decisions() {
        use crate::waywatch::{Trial, Watch};
        let p = Progress::default();
        let mut fern = Battle::new_trial(
            Trial {
                watch: Watch::Fern,
                rank: 0,
            },
            &p,
        );
        let incoming = fern.incoming(fern.intent());
        fern.end_turn();
        assert_eq!(fern.hp, fern.max_hp - incoming - 2);
        let mut defended = Battle::new_trial(
            Trial {
                watch: Watch::Fern,
                rank: 0,
            },
            &p,
        );
        assert!(defended.eat(Ration::Jam));
        assert!(!defended.eat(Ration::Soup));
        defended.end_turn();
        assert_eq!(defended.hp, defended.max_hp);
        let kiln = Battle::new_trial(
            Trial {
                watch: Watch::Kiln,
                rank: 0,
            },
            &p,
        );
        assert_eq!(kiln.enemy_guard, 3);
        let mut fen = Battle::new_trial(
            Trial {
                watch: Watch::Fen,
                rank: 0,
            },
            &p,
        );
        fen.end_turn();
        fen.end_turn();
        assert_eq!(fen.round, 3);
        assert_eq!(fen.energy, fen.energy_limit() - 1);
        assert!(fen.eat(Ration::Stew));
        assert_eq!(fen.energy, fen.energy_limit());
    }

    #[test]
    fn a_trained_deck_can_win_all_twelve_watch_trials() {
        use crate::waywatch::{Trial, Watch};
        let mut p = Progress::default();
        p.gear = Gear::CrystalBlade;
        p.ranks = [4, 4, 2];
        let cards = [
            Card::QuartzThrust,
            Card::CopperArc,
            Card::LanternWard,
            Card::FlaxWrap,
            Card::MossBloom,
            Card::StoneEcho,
        ];
        for card in cards {
            p.learn(card);
        }
        let deck = cards
            .into_iter()
            .flat_map(|card| [card, card])
            .collect::<Vec<_>>();
        p.save_deck(&deck).unwrap();
        for watch in [Watch::Fern, Watch::Kiln, Watch::Fen] {
            for rank in 0..Trial::RANKS {
                let mut battle = Battle::new_trial(Trial { watch, rank }, &p);
                for _ in 0..26 {
                    for _ in 0..8 {
                        let candidate = battle
                            .hand
                            .iter()
                            .enumerate()
                            .filter(|(_, card)| card.cost() <= battle.energy)
                            .max_by_key(|(_, card)| {
                                let spec = card.spec();
                                let threat = battle.incoming(battle.intent());
                                let protect = u16::from(spec.guard).min(threat);
                                let heal = u16::from(spec.healing).min(battle.max_hp - battle.hp);
                                u16::from(spec.damage) * 5
                                    + protect * 3
                                    + heal * 2
                                    + u16::from(spec.spores) * 4
                            })
                            .map(|(index, _)| index);
                        if !candidate.is_some_and(|index| battle.play(index)) {
                            break;
                        }
                    }
                    battle.end_turn();
                    if battle.outcome != Outcome::Running {
                        break;
                    }
                }
                assert_eq!(battle.outcome, Outcome::Won, "{watch:?} rank {rank}");
            }
        }
    }

    #[test]
    fn the_opening_battle_is_winnable_with_the_starter_deck() {
        let mut fight = Battle::new(1, &Progress::default());
        for _ in 0..8 {
            if fight.outcome != Outcome::Running {
                break;
            }
            for _ in 0..8 {
                let candidate = fight
                    .hand
                    .iter()
                    .enumerate()
                    .filter(|&(_, card)| card.cost() <= fight.energy && card.spec().damage > 0)
                    .max_by_key(|&(_, card)| card.spec().damage)
                    .map(|(index, _)| index);
                let Some(index) = candidate else {
                    break;
                };
                fight.play(index);
            }
            fight.end_turn();
        }
        assert_eq!(fight.outcome, Outcome::Won);
        assert!(fight.hp > 0);
    }

    #[test]
    fn a_growing_collection_and_equipment_can_finish_the_full_expedition() {
        let mut progress = Progress::default();
        progress.gear = Gear::CopperSickle;
        progress.learn(Card::QuartzThrust);
        progress.learn(Card::FlaxWrap);
        progress.learn(Card::CopperArc);
        for level in 1..=LAST_LEVEL {
            // Rest after each group of five, spend earned points and replace starter cards.
            if level == 6 || level == 11 {
                progress.gear = if level == 6 {
                    Gear::SilverGuard
                } else {
                    Gear::CrystalBlade
                };
            }
            if progress.ranks[Talent::Focus as usize] < 2 {
                progress.train(Talent::Focus);
            } else {
                progress.train(Talent::Heart);
                progress.train(Talent::Shell);
            }
            let mut deck = [
                Card::QuartzThrust,
                Card::QuartzThrust,
                Card::CopperArc,
                Card::CopperArc,
                Card::FlaxWrap,
                Card::FlaxWrap,
                Card::SeedShot,
                Card::SeedShot,
                Card::BarkShield,
                Card::LeafPoultice,
                Card::LeafPoultice,
                Card::Scout,
            ];
            if progress.knows(Card::RainStep) {
                deck[8] = Card::RainStep;
            }
            if progress.knows(Card::MossBloom) {
                deck[7] = Card::MossBloom;
            }
            if progress.knows(Card::StoneEcho) {
                deck[9] = Card::StoneEcho;
            }
            if progress.knows(Card::HarborSong) {
                deck[11] = Card::HarborSong;
            }
            progress.save_deck(&deck).unwrap();
            let mut battle = Battle::new(level, &progress);
            for _ in 0..12 {
                if battle.outcome != Outcome::Running {
                    break;
                }
                // Prefer a finishing move, otherwise weigh health, incoming damage and attack.
                while let Some(index) = choose_card(&battle) {
                    battle.play(index);
                }
                battle.end_turn();
            }
            assert_eq!(
                battle.outcome,
                Outcome::Won,
                "level {level}, health {}",
                progress.hp
            );
            assert!(progress.win(level, battle.hp).is_some());
            if level % CAMP_INTERVAL == 0 {
                progress.rest();
            }
        }
        assert_eq!(progress.camp(), LAST_LEVEL);
        assert!(progress.knows(Card::MoonSlice));
    }

    fn choose_card(battle: &Battle) -> Option<usize> {
        if battle.outcome != Outcome::Running {
            return None;
        }
        let incoming = battle.incoming(battle.intent());
        battle
            .hand
            .iter()
            .enumerate()
            .filter_map(|(index, card)| {
                let spec = card.spec();
                if spec.cost > battle.energy {
                    return None;
                }
                let damage = if spec.damage > 0 {
                    (u16::from(spec.damage) + u16::from(battle.strength))
                        .saturating_sub(if spec.piercing { 0 } else { battle.enemy_guard })
                } else {
                    0
                };
                let healing = u16::from(spec.healing).min(battle.max_hp - battle.hp);
                let guard = u16::from(spec.guard).min(incoming);
                let wins = damage >= battle.enemy_hp;
                let safety = guard > 0 && incoming >= battle.hp;
                let breaks_charge = battle.charged.is_some()
                    && !spec.piercing
                    && u16::from(spec.damage) + u16::from(battle.strength) >= battle.enemy_guard;
                let score = i32::from(damage + u16::from(spec.spores) * 2 + guard + healing * 2)
                    + i32::from(spec.draw) * 2
                    - i32::from(spec.cost) * 2
                    + if wins {
                        1000
                    } else if safety || breaks_charge && battle.enemy_guard != 0 {
                        100
                    } else {
                        0
                    };
                (score > 0).then_some((index, score))
            })
            .max_by_key(|&(_, score)| score)
            .map(|(index, _)| index)
    }

    #[test]
    fn a_charge_can_be_stopped_and_flurries_apply_resistance_per_hit() {
        let mut battle = Battle::new(25, &Progress::default());
        let Intent::Charge { guard, damage } = battle.intent() else {
            panic!("final boss must telegraph")
        };
        battle.end_turn();
        assert_eq!(battle.intent(), Intent::Attack(damage));
        assert_eq!(battle.enemy_guard, guard);
        battle.hand = vec![Card::TrailRush, Card::TwigStrike];
        assert!(battle.play(0));
        assert!(battle.play(0));
        assert_eq!(battle.intent(), Intent::Stagger);
        battle.end_turn();
        assert_eq!(battle.hp, battle.max_hp);
        battle.resistance = 2;
        battle.guard = 3;
        assert_eq!(battle.incoming(Intent::Flurry(8)), 9);
    }
}

#[cfg(test)]
#[test]
fn choosing_any_card_forecasts_the_actual_life_and_armor_change() {
    for card in Card::ALL {
        for armor in [0, 5, 60] {
            for combo in [false, true] {
                let mut battle = Battle::new(1, &Progress::default());
                battle.enemy_guard=armor; battle.enemy_hp=12; battle.hp=battle.max_hp-3;
                battle.marked=combo; battle.armor_break=combo; battle.style=Style::Warden;
                battle.warden_ready=true; battle.guard=u16::from(combo)*4;
                battle.hand=vec![card]; battle.energy=3;
                let hit=battle.impact(card); let before=(battle.enemy_hp,battle.hp,battle.guard);
                assert!(battle.play(0));
                assert_eq!((before.0-battle.enemy_hp, armor-battle.enemy_guard, battle.hp-before.1,battle.guard-before.2),
                    (hit.damage,hit.armor,hit.healed,hit.shield), "{card:?}, armor {armor}, combo {combo}");
                if card==Card::BloomWard {assert_eq!(battle.hand.len(),usize::from(combo));}
            }
        }
    }
}

#[cfg(test)]
#[test]
fn preview_recognizes_an_interrupted_charge_and_a_lethal_spore_tick() {
    let mut battle = Battle::new(1, &Progress::default());
    battle.charged = Some(16);
    battle.enemy_guard = 3;
    battle.hand = vec![Card::TwigStrike];
    assert!(battle.forecast(Card::TwigStrike).contains("16 → 0"));
    let hp = battle.hp;
    assert!(battle.play(0));
    battle.end_turn();
    assert_eq!(battle.hp, hp);

    let mut dummy = Battle::training();
    dummy.enemy_hp = 7;
    dummy.spores = [2, 0];
    dummy.hand = vec![Card::SeedShot];
    assert!(dummy.forecast(Card::SeedShot).contains("8 → 0"));
    assert!(dummy.play(0));
    dummy.end_turn();
    assert_eq!(dummy.outcome, Outcome::Won);
    assert_eq!(dummy.hp, dummy.max_hp);
}
