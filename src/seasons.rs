use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    furniture,
    garden::{Crop, Garden},
    homestead::Product,
    town::Npc,
    weather::Season,
};

pub const MEETING_POINT: Vec2 = Vec2::new(47.2, 6.2);
pub const MEETING_START: u32 = 5;
pub const CONTRIBUTION: u32 = 2;
pub const PRIZE: u32 = 18;

#[derive(Clone, Copy)]
pub struct SeasonalFind {
    pub season: Season,
    pub point: Vec2,
}

impl SeasonalFind {
    const fn new(season: Season, x: f32, z: f32) -> Self {
        Self {
            season,
            point: Vec2::new(x, z),
        }
    }
}

// Keep the order stable. The saved cooldowns are indexed by gathering spot.
pub const SITES: [SeasonalFind; 12] = [
    SeasonalFind::new(Season::Spring, -9.9, 4.9),
    SeasonalFind::new(Season::Spring, -40.0, -4.6),
    SeasonalFind::new(Season::Spring, -55.4, -5.7),
    SeasonalFind::new(Season::Summer, 1.6, 44.6),
    SeasonalFind::new(Season::Summer, -5.8, 50.3),
    SeasonalFind::new(Season::Summer, -14.5, 54.8),
    SeasonalFind::new(Season::Autumn, -13.5, 1.9),
    SeasonalFind::new(Season::Autumn, -44.9, -5.1),
    SeasonalFind::new(Season::Autumn, -59.8, -4.6),
    SeasonalFind::new(Season::Winter, 10.0, 6.3),
    SeasonalFind::new(Season::Winter, 26.4, 5.7),
    SeasonalFind::new(Season::Winter, 38.8, 7.7),
];

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct SeasonProgress {
    pub stock: [u32; 4],
    pub discovered: [bool; 4],
    pub celebrated: [bool; 4],
    picked: [Option<u32>; SITES.len()],
    attended: [Option<u32>; 4],
}

impl SeasonProgress {
    pub fn available(&self, site: usize, day: u32) -> bool {
        SITES[site].season == Season::at(day)
            && self.picked[site].is_none_or(|last| day.saturating_sub(last) >= 2)
    }

    pub fn pick(garden: &mut Garden, site: usize) -> Option<String> {
        if !garden.seasons.available(site, garden.day) {
            return None;
        }
        let season = SITES[site].season;
        garden.seasons.picked[site] = Some(garden.day);
        let first = !std::mem::replace(&mut garden.seasons.discovered[season as usize], true);
        garden.add_stock(Product::Seasonal(season), 1);
        Some(if first {
            format!("Entdeckt · {} im Kalender", season.find_name())
        } else {
            format!("+1 {}", season.find_name())
        })
    }

    pub fn collection(&self) -> usize {
        self.discovered.iter().filter(|&&item| item).count()
    }

    pub fn meetings(&self) -> usize {
        self.celebrated.iter().filter(|&&event| event).count()
    }

    pub fn attended(&self, day: u32) -> bool {
        self.attended[Season::at(day) as usize] == Some(Season::period(day))
    }
}

impl Season {
    pub fn find_name(self) -> &'static str {
        match self {
            Self::Spring => "Blütenzweig",
            Self::Summer => "Sonnenmuschel",
            Self::Autumn => "Bernsteinnuss",
            Self::Winter => "Frostbeere",
        }
    }

    pub fn find_hint(self) -> &'static str {
        match self {
            Self::Spring => "Am Weg vom Hof zum Farnwald und rund um die Bachbrücke.",
            Self::Summer => "Im Sand entlang des Küstenwegs in der Muschelbucht.",
            Self::Autumn => "Am westlichen Hofweg und auf beiden Bachseiten im Farnwald.",
            Self::Winter => "An der Straße vom Hof nach Eichenhafen.",
        }
    }

    pub fn meeting(self) -> &'static str {
        match self {
            Self::Spring => "Blütentausch",
            Self::Summer => "Sommertafel",
            Self::Autumn => "Laublichter",
            Self::Winter => "Frostfrühstück",
        }
    }

    pub fn host(self) -> Npc {
        match self {
            Self::Spring => Npc::Lina,
            Self::Summer => Npc::Fiete,
            Self::Autumn => Npc::Bo,
            Self::Winter => Npc::Mara,
        }
    }

    pub fn invitation(self) -> &'static str {
        match self {
            Self::Spring => {
                "Beim Blütentausch binden wir Zweige für die ersten Sträuße des Jahres."
            }
            Self::Summer => "An der Sommertafel legen wir unsere schönsten Muscheln aus.",
            Self::Autumn => "Bei den Laublichtern hängen wir Nüsse und Laternen an den Dorftisch.",
            Self::Winter => "Beim Frostfrühstück gibt es warmen Tee und frostblaue Beeren.",
        }
    }

    pub fn gift_seeds(self) -> Crop {
        match self {
            Self::Spring => Crop::Strawberry,
            Self::Summer => Crop::Tomato,
            Self::Autumn => Crop::Pumpkin,
            Self::Winter => Crop::Turnip,
        }
    }

    pub fn find_model(self) -> &'static str {
        match self {
            Self::Spring => "season_find_spring",
            Self::Summer => "season_find_summer",
            Self::Autumn => "season_find_autumn",
            Self::Winter => "season_find_winter",
        }
    }

    pub fn table_model(self) -> &'static str {
        match self {
            Self::Spring => "season_table_spring",
            Self::Summer => "season_table_summer",
            Self::Autumn => "season_table_autumn",
            Self::Winter => "season_table_winter",
        }
    }
}

pub fn meeting_status(garden: &Garden) -> String {
    let day = Season::day(garden.day);
    if garden.seasons.attended(garden.day) {
        "Du hast an diesem Treffen teilgenommen.".into()
    } else if day < MEETING_START {
        format!(
            "Beginn in {} Tagen · Tag {MEETING_START}–7, 06–20 Uhr",
            MEETING_START - day
        )
    } else if !garden.clock.market_open() {
        "Für heute ist der Dorftisch geschlossen.".into()
    } else {
        "Der Dorftisch ist bis 20 Uhr geöffnet.".into()
    }
}

pub fn can_contribute(garden: &Garden) -> bool {
    Season::day(garden.day) >= MEETING_START
        && garden.clock.market_open()
        && !garden.seasons.attended(garden.day)
        && garden.stock(Product::Seasonal(Season::at(garden.day))) >= CONTRIBUTION
}

pub fn contribute(garden: &mut Garden) -> Result<String, String> {
    if !can_contribute(garden) {
        return Err(
            "Für den Beitrag brauchst du zwei passende Saisonfunde und einen geöffneten Dorftisch."
                .into(),
        );
    }
    let season = Season::at(garden.day);
    garden.take_stock(Product::Seasonal(season), CONTRIBUTION);
    garden.seasons.attended[season as usize] = Some(Season::period(garden.day));
    garden.seasons.celebrated[season as usize] = true;
    garden.farm.coins = garden.farm.coins.saturating_add(PRIZE);
    garden.add_seeds(season.gift_seeds(), 3);
    let mut message = format!(
        "Danke fürs Mitfeiern! +{PRIZE} Münzen · +3 {}-Samen",
        season.gift_seeds().name()
    );
    if garden.seasons.meetings() == Season::ALL.len()
        && furniture::learn(garden, furniture::SEASONS_SHELF)
    {
        message.push_str(" · Bauplan fürs Jahreszeitenregal");
    }
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_return_after_two_days_and_discovery_survives_sale() {
        let mut garden = Garden::default();
        let spring = Product::Seasonal(Season::Spring);
        assert!(SeasonProgress::pick(&mut garden, 0).is_some());
        assert!(SeasonProgress::pick(&mut garden, 0).is_none());
        assert_eq!(garden.sell_many(spring, 1), Some(spring.value()));
        assert_eq!(garden.seasons.collection(), 1);
        garden.sleep();
        assert!(!garden.seasons.available(0, garden.day));
        garden.sleep();
        assert!(garden.seasons.available(0, garden.day));
        assert!(!garden.seasons.available(3, garden.day));
        garden.day = 7;
        garden.sleep();
        assert_eq!(garden.day, 8);
        assert_eq!(Season::day(garden.day), 1);
        assert!(!garden.seasons.available(0, garden.day));
        assert!(garden.seasons.available(3, garden.day));
    }

    #[test]
    fn gifts_are_awarded_once_per_meeting_and_the_cycle_repeats() {
        let mut garden = Garden::default();
        let starting_balance = garden.farm.coins;
        for (index, season) in Season::ALL.into_iter().enumerate() {
            garden.day = 1 + Season::LENGTH * index as u32;
            let gift = Product::Seasonal(season);
            garden.add_stock(gift, CONTRIBUTION * 2);
            assert!(contribute(&mut garden).is_err());
            garden.day += MEETING_START - 1;
            let seeds = garden.seed_count(season.gift_seeds());
            assert!(contribute(&mut garden).is_ok());
            assert_eq!(garden.seed_count(season.gift_seeds()), seeds + 3);
            assert!(contribute(&mut garden).is_err());
            assert_eq!(garden.stock(gift), CONTRIBUTION);
            assert_eq!(garden.seasons.meetings(), index + 1);
            assert_eq!(garden.blueprints[furniture::SEASONS_SHELF], index == 3);
        }
        assert_eq!(garden.farm.coins, starting_balance + 4 * PRIZE);
        garden.day = 4 * Season::LENGTH + MEETING_START;
        assert!(contribute(&mut garden).is_ok());
        assert_eq!(garden.seasons.meetings(), 4);
    }

    #[test]
    fn winter_finds_replace_the_dormant_orchard_until_spring() {
        let mut garden = Garden::default();
        garden.day = 22;
        assert!(SeasonProgress::pick(&mut garden, 9).is_some());
        assert!(!garden.farm.fruit_ready(0, garden.day));
        garden.day = 28;
        garden.sleep();
        assert_eq!(Season::at(garden.day), Season::Spring);
        assert!(garden.farm.fruit_ready(0, garden.day));
        assert!(!garden.seasons.available(9, garden.day));
    }
}
