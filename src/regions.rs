use bevy::prelude::Vec2;
use serde::{Deserialize, Serialize};

use crate::{
    garden::{Crop, Garden},
    homestead::Product,
    processing::Dish,
    town::Require,
    travel::District,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Material {
    GorgeQuartz,
    MeadowFlax,
    MarshHerb,
}

impl Material {
    pub const ALL: [Self; 3] = [Self::GorgeQuartz, Self::MeadowFlax, Self::MarshHerb];

    pub fn name(self) -> &'static str {
        match self {
            Self::GorgeQuartz => "Klammquarz",
            Self::MeadowFlax => "Wiesenflachs",
            Self::MarshHerb => "Nebelminze",
        }
    }

    pub fn value(self) -> u32 {
        [8, 5, 7][self as usize]
    }

    pub fn model(self) -> &'static str {
        match self {
            Self::GorgeQuartz => "find_gorge_quartz",
            Self::MeadowFlax => "find_meadow_flax",
            Self::MarshHerb => "find_marsh_herb",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frontier {
    Copper,
    Wind,
    Moor,
}

impl Frontier {
    pub const ALL: [Self; 3] = [Self::Copper, Self::Wind, Self::Moor];

    pub fn district(self) -> District {
        match self {
            Self::Copper => District::Copper,
            Self::Wind => District::Wind,
            Self::Moor => District::Moor,
        }
    }

    pub fn for_district(district: District) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|gate| gate.district() == district)
    }

    pub fn position(self) -> Vec2 {
        match self {
            Self::Copper => Vec2::new(77.77, 9.0),
            Self::Wind => Vec2::new(9.5, 75.78),
            Self::Moor => Vec2::new(-86.79, -14.0),
        }
    }

    pub fn angle(self) -> f32 {
        match self {
            Self::Wind => 0.,
            Self::Copper => -std::f32::consts::FRAC_PI_2,
            Self::Moor => std::f32::consts::FRAC_PI_2,
        }
    }

    pub fn entrance(self) -> Vec2 {
        self.position()
            + 1.55
                * match self {
                    Self::Copper => Vec2::X,
                    Self::Wind => Vec2::Y,
                    Self::Moor => Vec2::NEG_X,
                }
    }

    pub fn boundary(self, point: Vec2) -> bool {
        match self {
            Self::Copper => point.x >= 78. && point.y < 70.,
            Self::Wind => {
                let from_bridge = point.x - 9.5;
                point.y
                    >= 76.15
                        + 1.13 * (from_bridge * 0.079).sin()
                        + 0.41 * (from_bridge * 0.153).sin()
            }
            Self::Moor => point.x <= -87.,
        }
    }

    pub fn unlocked(self, garden: &Garden) -> bool {
        garden.regions.gates[self as usize] >= 2
    }

    fn quest_name(self) -> &'static str {
        match self {
            Self::Copper => "Ein Steg über die Klamm",
            Self::Wind => "Den Weideweg abstecken",
            Self::Moor => "Licht für die Nebelpfade",
        }
    }

    fn supplies(self) -> &'static [Require] {
        match self {
            Self::Copper => &[Require::Wood(4), Require::Stone(3)],
            Self::Wind => &[Require::Wood(4), Require::Product(Product::Wool, 1)],
            Self::Moor => &[
                Require::Product(Product::Regional(Material::GorgeQuartz), 2),
                Require::Wood(3),
            ],
        }
    }

    pub fn describe(self, g: &Garden) -> (String, &'static str, bool) {
        if self.unlocked(g) {
            return ("Der Weg ist offen.".into(), "Freigeschaltet", false);
        }
        let needed = self as usize + 1;
        if g.town.completed() < needed {
            return (
                format!("{} / {needed} Stadtaufträge erfüllt.", g.town.completed()),
                "Gesperrt",
                false,
            );
        }
        let ready = has_supplies(self.supplies(), g);
        if g.regions.gates[self as usize] == 0 && !ready {
            return (
                format!("{} · {}", self.quest_name(), progress(self.supplies(), g)),
                "Auftrag annehmen",
                true,
            );
        }
        (progress(self.supplies(), g), "Weg öffnen", ready)
    }

    pub fn work(self, g: &mut Garden) -> Result<String, String> {
        let (hint, _, enabled) = self.describe(g);
        if !enabled {
            return Err(hint);
        }
        let state = g.regions.gates[self as usize];
        if state == 0 && !has_supplies(self.supplies(), g) {
            g.regions.gates[self as usize] = 1;
            return Ok(format!("Auftrag angenommen · {}", self.quest_name()));
        }
        take_supplies(self.supplies(), g);
        g.regions.gates[self as usize] = 2;
        Ok(format!(
            "Der Weg nach {} ist offen.",
            self.district().name()
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bell {
    Farm,
    Forest,
    Beach,
    Town,
    Copper,
    Wind,
    Moor,
}

impl Bell {
    pub const ALL: [Self; 7] = [
        Self::Farm,
        Self::Forest,
        Self::Beach,
        Self::Town,
        Self::Copper,
        Self::Wind,
        Self::Moor,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Farm => "Hof",
            Self::Forest => "Farnwald",
            Self::Beach => "Muschelbucht",
            Self::Town => "Eichenhafen",
            Self::Copper => "Kupferklamm",
            Self::Wind => "Windweiden",
            Self::Moor => "Nebelmoor",
        }
    }

    pub fn position(self) -> Vec2 {
        match self {
            Self::Farm => Vec2::new(1.40, 5.90),
            Self::Forest => Vec2::new(-41.53, -4.43),
            Self::Beach => Vec2::new(3.22, 48.38),
            Self::Town => Vec2::new(49.92, 8.91),
            Self::Copper => District::Copper.waypoint() + Vec2::X * 1.36,
            Self::Wind => District::Wind.waypoint() + Vec2::X * 1.36,
            Self::Moor => District::Moor.waypoint() + Vec2::X * 1.36,
        }
    }

    pub fn arrival(self) -> Vec2 {
        self.position()
            + if self == Self::Farm {
                Vec2::NEG_X
            } else {
                Vec2::Y
            } * 1.18
    }

    fn supplies(self) -> &'static [Require] {
        use Require::{Product as Goods, Stone, Wood};
        match self {
            Self::Farm => &[Wood(3)],
            Self::Forest => &[Goods(Product::Hazel, 2), Wood(2)],
            Self::Beach => &[Goods(Product::Shell, 2), Wood(2)],
            Self::Town => &[Stone(3)],
            Self::Copper => &[Goods(Product::Regional(Material::GorgeQuartz), 2), Stone(2)],
            Self::Wind => &[
                Goods(Product::Regional(Material::MeadowFlax), 2),
                Goods(Product::Wool, 1),
            ],
            Self::Moor => &[Goods(Product::Regional(Material::MarshHerb), 2), Wood(2)],
        }
    }

    pub fn ready(self, garden: &Garden) -> bool {
        garden.regions.bells[self as usize] >= 2
    }

    pub(crate) fn light(self, garden: &mut Garden) {
        garden.regions.bells[self as usize] = 2;
    }

    pub fn describe(self, g: &Garden) -> (String, &'static str, bool) {
        match g.regions.bells[self as usize] {
            0 => (
                "Repariere die Wegglocke für die Schnellreise.".into(),
                "Auftrag annehmen",
                true,
            ),
            1 => (
                progress(self.supplies(), g),
                "Glocke reparieren",
                has_supplies(self.supplies(), g),
            ),
            _ => (
                "Alle reparierten Wegglocken sind erreichbar.".into(),
                "Reisen",
                true,
            ),
        }
    }

    pub fn work(self, g: &mut Garden) -> Result<String, String> {
        let (hint, _, enabled) = self.describe(g);
        if !enabled || self.ready(g) {
            return Err(hint);
        }
        if g.regions.bells[self as usize] == 0 {
            g.regions.bells[self as usize] = 1;
            return Ok(format!("Auftrag angenommen · Wegglocke {}", self.name()));
        }
        take_supplies(self.supplies(), g);
        self.light(g);
        Ok(format!("Schnellreise freigeschaltet · {}", self.name()))
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct RegionProgress {
    pub goods: [u32; 3],
    gates: [u8; 3],
    bells: [u8; 7],
    stories: [u8; 3],
}

impl RegionProgress {
    pub fn passage(&self, point: Vec2) -> bool {
        Frontier::ALL
            .into_iter()
            .all(|area| self.gates[area as usize] >= 2 || !area.boundary(point))
    }

    pub fn story_steps(&self, area: Frontier) -> u8 {
        self.stories[area as usize] / 2
    }

    pub fn next_order(&self, area: Frontier) -> Option<&'static str> {
        ORDERS[area as usize]
            .get(self.story_steps(area) as usize)
            .map(|order| order.title)
    }
}

fn has_supplies(needs: &[Require], g: &Garden) -> bool {
    needs
        .iter()
        .all(|item| item.available(g) >= item.quantity())
}

fn progress(needs: &[Require], g: &Garden) -> String {
    needs
        .iter()
        .map(|item| item.progress(g))
        .collect::<Vec<_>>()
        .join(" · ")
}

fn take_supplies(needs: &[Require], g: &mut Garden) {
    for supply in needs {
        supply.take(g);
    }
}

struct Order {
    title: &'static str,
    request: &'static str,
    materials: &'static [Require],
    coins: u32,
    plan: Option<usize>,
    seed: Crop,
}

const ORDERS: [[Order; 3]; 3] = [
    [
        Order {
            title: "Aus dem anderen Ufer",
            request: "Sammle Quarz an den hellen Wänden der Klamm. Die Brückenbauer brauchen ihn für Wegmarken.",
            materials: &[Require::Product(
                Product::Regional(Material::GorgeQuartz),
                3,
            )],
            coins: 21,
            plan: None,
            seed: Crop::Turnip,
        },
        Order {
            title: "Licht für die Werkstatt",
            request: "Runa will die Kupferlampen am Ufer wieder entzünden. Bring Kupfer und Stein für die Sockel.",
            materials: &[Require::Copper(2), Require::Stone(4)],
            coins: 32,
            plan: Some(15),
            seed: Crop::Tomato,
        },
        Order {
            title: "Ein warmer Feierabend",
            request: "Koche eine Möhren-Pilz-Suppe für die Bauleute und bring ein wenig Holz für das Feuer.",
            materials: &[
                Require::Product(Product::Prepared(Dish::CarrotSoup), 1),
                Require::Wood(4),
            ],
            coins: 38,
            plan: None,
            seed: Crop::Sunflower,
        },
    ],
    [
        Order {
            title: "Flachs für die Mühle",
            request: "Suche nach Flachs mit blauen Blüten. Nele braucht neues Garn für die Flügel.",
            materials: &[Require::Product(Product::Regional(Material::MeadowFlax), 3)],
            coins: 23,
            plan: None,
            seed: Crop::Strawberry,
        },
        Order {
            title: "Eine warme Stube",
            request: "Wolle und Holz helfen Nele beim Ausbau der Mühle vor den ersten Stürmen.",
            materials: &[Require::Product(Product::Wool, 2), Require::Wood(5)],
            coins: 32,
            plan: Some(16),
            seed: Crop::Pumpkin,
        },
        Order {
            title: "Proviant am Deich",
            request: "Mühle Haselnüsse zu Mehl und gib es mit zwei Eiern für die nächste Wanderrunde ab.",
            materials: &[
                Require::Product(Product::Prepared(Dish::HazelFlour), 1),
                Require::Product(Product::Egg, 2),
            ],
            coins: 39,
            plan: None,
            seed: Crop::Sunflower,
        },
    ],
    [
        Order {
            title: "Ein Duft im Moor",
            request: "Ernte Nebelminze am Rand der Stillwasser. Edda markiert damit die Pfade für Wandernde.",
            materials: &[Require::Product(Product::Regional(Material::MarshHerb), 3)],
            coins: 25,
            plan: None,
            seed: Crop::Carrot,
        },
        Order {
            title: "Das alte Glockengewölbe",
            request: "Die Fassung am Steinbogen braucht Klammquarz und etwas Stein. Sie weist den Weg zu den Dämmergängen.",
            materials: &[
                Require::Product(Product::Regional(Material::GorgeQuartz), 2),
                Require::Stone(3),
            ],
            coins: 35,
            plan: None,
            seed: Crop::Tomato,
        },
        Order {
            title: "Ein Abend im Pfahlhaus",
            request: "Bring Waldbeerkonfitüre und warmen Wollstoff für ein Dankeschön an die Wegwacht.",
            materials: &[
                Require::Product(Product::Prepared(Dish::BerryJam), 1),
                Require::Product(Product::Wool, 1),
            ],
            coins: 44,
            plan: Some(17),
            seed: Crop::Strawberry,
        },
    ],
];

pub fn story(area: Frontier, g: &Garden) -> (String, &'static str, bool) {
    let state = g.regions.stories[area as usize];
    let Some(quest) = ORDERS[area as usize].get(usize::from(state / 2)) else {
        return ("Alle Aufträge hier sind erfüllt.".into(), "Erfüllt", false);
    };
    let materials = progress(quest.materials, g);
    (
        format!(
            "{}\n{}\n{materials} · {} Münzen Belohnung",
            quest.title, quest.request, quest.coins
        ),
        if state % 2 == 0 {
            "Auftrag annehmen"
        } else {
            "Auftrag abgeben"
        },
        true,
    )
}

pub(crate) fn story_mark(area: Frontier, garden: &Garden) -> Option<crate::town::QuestMark> {
    use crate::town::QuestMark::{Available, TurnIn};
    let state = garden.regions.stories[area as usize];
    let order = ORDERS[area as usize].get(usize::from(state / 2))?;
    if state % 2 == 0 {
        Some(Available)
    } else {
        has_supplies(order.materials, garden).then_some(TurnIn)
    }
}

pub fn finish_story(area: Frontier, g: &mut Garden) -> Result<String, String> {
    let state = g.regions.stories[area as usize];
    let Some(quest) = ORDERS[area as usize].get(usize::from(state / 2)) else {
        return Err("Alle Aufträge sind erfüllt.".into());
    };
    if state % 2 == 0 {
        g.regions.stories[area as usize] += 1;
        return Ok(format!("Auftrag angenommen · {}", quest.title));
    }
    if !has_supplies(quest.materials, g) {
        return Err(format!(
            "Sammle noch die fehlenden Waren · {}",
            progress(quest.materials, g)
        ));
    }
    take_supplies(quest.materials, g);
    g.farm.coins = g.farm.coins.saturating_add(quest.coins);
    g.add_seeds(quest.seed, 2);
    if let Some(plan) = quest.plan {
        crate::furniture::learn(g, plan);
    }
    g.regions.stories[area as usize] += 1;
    let mut reward = format!("Auftrag erfüllt · +{} Münzen · +2 Samen", quest.coins);
    if let Some(card) = crate::progression::quest_card(area, g.regions.story_steps(area))
        && g.rpg.learn(card)
    {
        reward.push_str(&format!(" · Karte „{}“", card.name()));
    }
    Ok(reward)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adventure::Adventure;

    #[test]
    fn each_gate_requires_its_quest_and_spends_materials_once() {
        let mut garden: Garden = serde_json::from_str(r#"{"town":{"quests":["Done","Done","Done"]},"wood":20,"stone":10,"farm":{"goods":[0,0,0,2,0,0]},"regions":{"goods":[2,0,0]}}"#).unwrap();
        for gate in Frontier::ALL {
            let goal = gate.district().waypoint();
            assert!(!garden.regions.passage(goal));
            gate.work(&mut garden).unwrap();
            assert!(garden.regions.passage(goal));
            assert!(Adventure::default().walkable(gate.entrance(), &garden));
            assert!(gate.work(&mut garden).is_err());
        }
        assert_eq!(garden.wood, 9);
        assert_eq!(garden.stone, 7);
        assert_eq!(garden.stock(Product::Wool), 1);
    }

    #[test]
    fn an_accepted_gate_stays_closed_until_its_materials_are_available() {
        let mut garden: Garden =
            serde_json::from_str(r#"{"town":{"quests":["Done","Offered","Offered"]}}"#).unwrap();
        let gate = Frontier::Copper;
        gate.work(&mut garden).unwrap();
        assert!(!gate.unlocked(&garden));
        garden.wood = 4;
        garden.stone = 3;
        gate.work(&mut garden).unwrap();
        assert!(gate.unlocked(&garden));
    }

    #[test]
    fn repaired_bells_have_walkable_arrivals() {
        let mut garden: Garden =
            serde_json::from_str(r#"{"regions":{"gates":[2,2,2]},"wood":3}"#).unwrap();
        assert!(!Bell::Farm.ready(&garden));
        Bell::Farm.work(&mut garden).unwrap();
        Bell::Farm.work(&mut garden).unwrap();
        assert!(Bell::Farm.ready(&garden));
        assert_eq!(garden.wood, 0);
        for bell in Bell::ALL {
            assert!(
                Adventure::default().walkable(bell.arrival(), &garden),
                "{bell:?}"
            );
        }
    }
}
