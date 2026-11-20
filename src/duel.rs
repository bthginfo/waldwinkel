use serde::{Deserialize, Serialize};

pub const DECK_SIZE: usize = 12;
pub const LAST_LEVEL: u32 = 25;
pub const CAMP_INTERVAL: u32 = 5;
const COPY_LIMIT: usize = 2;

#[derive(Clone, Copy)]
pub enum Emblem {
    Strike,
    Guard,
    Heal,
    Focus,
    Spore,
}

impl Emblem {
    pub const ALL: [Self; 5] = [
        Self::Strike,
        Self::Guard,
        Self::Heal,
        Self::Focus,
        Self::Spore,
    ];

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
    pub const ALL: [Self; 18] = [
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
            Self::HearthMeal => base("Abendmahl", 2, 0, 10, 14),
        }
    }

    pub fn name(self) -> &'static str {
        self.spec().name
    }

    pub fn cost(self) -> u8 {
        self.spec().cost
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
        let spec = self.spec();
        let mut pieces = Vec::new();
        if spec.damage > 0 {
            pieces.push(format!(
                "{} Schaden{}",
                spec.damage,
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
    cards: [bool; Card::ALL.len()],
    deck: [Card; DECK_SIZE],
    pub gear: Gear,
    pub gear_owned: [bool; Gear::ALL.len()],
    pub ranks: [u8; Talent::ALL.len()],
    pub points: u8,
    pub cleared: u32,
    pub hp: u16,
    pub trophies: [u32; Loot::ALL.len()],
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
    BellKeeper,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intent {
    Attack(u16),
    Brace(u16),
}

impl Enemy {
    pub fn at(level: u32) -> Self {
        if level % CAMP_INTERVAL == 0 {
            Self::BellKeeper
        } else {
            [Self::Mosskin, Self::KilnGuard, Self::MistOwl][((level - 1) * 7 % 3) as usize]
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Mosskin => "Mooshüter",
            Self::KilnGuard => "Tonwächter",
            Self::MistOwl => "Nebeleule",
            Self::BellKeeper => "Glockenhüter",
        }
    }

    pub fn model(self) -> &'static str {
        match self {
            Self::Mosskin => "duel_mosskin",
            Self::KilnGuard => "duel_kiln",
            Self::MistOwl => "duel_owl",
            Self::BellKeeper => "duel_keeper",
        }
    }

    pub fn loot(self) -> Loot {
        match self {
            Self::Mosskin => Loot::MossDown,
            Self::KilnGuard => Loot::ClaySeal,
            Self::MistOwl | Self::BellKeeper => Loot::MoonDust,
        }
    }

    fn hp(self, level: u32) -> u16 {
        let base = 14 + level as u16 * 2;
        base + match self {
            Self::KilnGuard => 4,
            Self::BellKeeper => 12,
            _ => 0,
        }
    }

    fn intent(self, level: u32, round: u16) -> Intent {
        let bonus = match self {
            Self::MistOwl => 2,
            Self::BellKeeper => 4,
            _ => 0,
        };
        let brace_round = match self {
            Self::Mosskin => 3,
            Self::KilnGuard => 2,
            Self::MistOwl => 4,
            Self::BellKeeper => 3,
        };
        if round % brace_round == 0 {
            Intent::Brace(4 + level as u16 / 3 + bonus)
        } else {
            Intent::Attack(4 + level as u16 / 2 + bonus)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Running,
    Won,
    Lost,
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
    resistance: u8,
    strength: u8,
    hand_size: u8,
    energy_limit: u8,
    drawing: Vec<Card>,
    used: Vec<Card>,
    spores: [u16; 2],
    rng: u64,
}

impl Battle {
    pub fn new(level: u32, progress: &Progress) -> Self {
        let foe = Enemy::at(level);
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
            resistance: progress.ranks[Talent::Shell as usize] + progress.gear.resistance(),
            strength: progress.gear.damage(),
            hand_size: 5 + u8::from(focus >= 1),
            energy_limit: 3 + u8::from(focus >= 2),
            drawing: progress.deck.to_vec(),
            used: Vec::new(),
            spores: [0; 2],
            rng: u64::from(level) * 0x85ebca77 + 0xc2b2ae35,
        };
        shuffle(&mut battle.drawing, &mut battle.rng);
        battle.draw(battle.hand_size);
        battle
    }

    pub fn intent(&self) -> Intent {
        self.enemy.intent(self.level, self.round)
    }

    pub fn incoming(&self, raw: u16) -> u16 {
        raw.saturating_sub(u16::from(self.resistance))
            .saturating_sub(self.guard)
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
        self.guard = self.guard.saturating_add(u16::from(spec.guard));
        self.hp = self
            .hp
            .saturating_add(u16::from(spec.healing))
            .min(self.max_hp);
        if spec.damage > 0 {
            let power = u16::from(spec.damage) + u16::from(self.strength);
            let absorbed = if spec.piercing {
                0
            } else {
                self.enemy_guard.min(power)
            };
            self.enemy_guard -= absorbed;
            self.enemy_hp = self.enemy_hp.saturating_sub(power - absorbed);
        }
        for spores in &mut self.spores {
            *spores = spores.saturating_add(u16::from(spec.spores));
        }
        self.draw(spec.draw);
        self.used.push(card);
        self.message = format!("{} gespielt", spec.name);
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
        self.message = match self.intent() {
            Intent::Attack(damage) => {
                let lost = self.incoming(damage);
                self.hp = self.hp.saturating_sub(lost);
                format!(
                    "{} greift an · {} Lebenspunkte verloren",
                    self.enemy.name(),
                    lost
                )
            }
            Intent::Brace(shield) => {
                self.enemy_guard = shield;
                format!("{} baut {} Schild auf.", self.enemy.name(), shield)
            }
        };
        if self.hp == 0 {
            self.outcome = Outcome::Lost;
            return;
        }
        self.used.append(&mut self.hand);
        self.round = self.round.saturating_add(1);
        self.guard = 0;
        self.energy = self.energy_limit;
        self.draw(self.hand_size);
    }
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
        let incoming = match battle.intent() {
            Intent::Attack(raw) => battle.incoming(raw),
            Intent::Brace(_) => 0,
        };
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
                let score = i32::from(damage + u16::from(spec.spores) * 2 + guard + healing * 2)
                    + i32::from(spec.draw) * 2
                    - i32::from(spec.cost) * 2
                    + if wins {
                        1000
                    } else if safety {
                        100
                    } else {
                        0
                    };
                (score > 0).then_some((index, score))
            })
            .max_by_key(|&(_, score)| score)
            .map(|(index, _)| index)
    }
}
