use bevy::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, de::Error};

use crate::{
    adventure::Target,
    craft::{self, Need, Recipe},
    furniture,
    garden::Garden,
    homestead::{Animal, Product},
    regions::{Bell, Frontier, Material},
    town::{Npc, QuestMark},
    travel::District,
};

/// New entries are appended to persisted catalogs. Short arrays are from older saves.
pub(crate) fn read_array<'de, D, T, const N: usize>(deserializer: D) -> Result<[T; N], D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    let values = Vec::<T>::deserialize(deserializer)?;
    if values.len() > N {
        return Err(D::Error::invalid_length(
            values.len(),
            &"a supported catalog length",
        ));
    }
    let mut values = values.into_iter();
    Ok(std::array::from_fn(|_| values.next().unwrap_or_default()))
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Progress {
    gates: [u8; 3],
    orders: [u8; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    Amber,
    Heights,
    Reef,
}

impl Region {
    pub const ALL: [Self; 3] = [Self::Amber, Self::Heights, Self::Reef];

    pub const fn district(self) -> District {
        [District::Amber, District::Heights, District::Reef][self as usize]
    }

    pub const fn npc(self) -> Npc {
        [Npc::Ivo, Npc::Sanna, Npc::Tilda][self as usize]
    }

    pub const fn animal(self) -> Animal {
        [Animal::Rabbit, Animal::Goat, Animal::Duck][self as usize]
    }

    fn bell(self) -> Bell {
        [Bell::Amber, Bell::Heights, Bell::Reef][self as usize]
    }

    fn previous(self) -> Frontier {
        [Frontier::Moor, Frontier::Copper, Frontier::Wind][self as usize]
    }

    pub fn gate(self) -> Vec2 {
        [
            Vec2::new(-203., -14.),
            Vec2::new(206., 4.),
            Vec2::new(31., 192.),
        ][self as usize]
    }

    pub fn direction(self) -> Vec2 {
        [Vec2::NEG_X, Vec2::X, Vec2::Y][self as usize]
    }

    pub fn boundary(self, point: Vec2) -> bool {
        match self {
            Self::Amber => point.x < self.gate().x,
            Self::Heights => point.x > self.gate().x && point.y < 69.,
            Self::Reef => point.y > self.gate().y && (-65. ..100.).contains(&point.x),
        }
    }

    pub fn on_causeway(self, point: Vec2) -> bool {
        let offset = point - self.gate();
        let along = offset.dot(self.direction());
        let across = offset.perp_dot(self.direction());
        (-9. ..13.).contains(&along) && across.abs() < 1.32
    }

    pub fn passage(point: Vec2, g: &Garden) -> bool {
        Self::ALL
            .into_iter()
            .all(|r| r.open(g) || !r.boundary(point))
    }

    pub fn open(self, g: &Garden) -> bool {
        g.expansion.gates[self as usize] >= 2
    }

    pub fn gate_title(self) -> &'static str {
        [
            "Die Planken am Harzpfad",
            "Das Seil über der Scharte",
            "Der Damm zur Morgenflut",
        ][self as usize]
    }

    fn gate_text(self) -> &'static str {
        [
            "Am Westrand des Moors endet der Holzweg. Hier trugen Eddas Leute früher die Harztöpfe über den Bach. Ersetze die morschen Planken und reibe die neuen mit Nebelminze ein. So finden die Träger den Abzweig selbst im Dunst.",
            "Hinter der Kupferklamm beginnt der Pfad zu Sannas Werkhof. Bei Wind schwingt das alte Geländer bis über die Kante. Mit Klammquarz als Gegengewicht und frischen Nieten lässt sich das Seil wieder spannen.",
            "Am Südende der Windweiden stand früher ein trockener Übergang für die Entenhirtinnen. Nele hat die Pfosten freigelegt. Bring Bretter und Wiesenflachs mit. Die Fasern halten auch dann, wenn die Flut unter dem Damm hindurchläuft.",
        ][self as usize]
    }

    fn gate_needs(self) -> &'static [Need] {
        use Material::*;
        use Need::Stock;
        match self {
            Self::Amber => &[
                Stock(Product::Regional(MarshHerb), 4),
                Stock(Product::Crafted(Recipe::Boards), 5),
            ],
            Self::Heights => &[
                Stock(Product::Regional(GorgeQuartz), 4),
                Need::Copper(5),
                Stock(Product::Crafted(Recipe::Boards), 4),
            ],
            Self::Reef => &[
                Stock(Product::Regional(MeadowFlax), 5),
                Stock(Product::Crafted(Recipe::Boards), 5),
            ],
        }
    }

    pub fn gate_status(self, g: &Garden) -> (String, &'static str, bool) {
        if self.open(g) {
            return ("Der Übergang ist begehbar.".into(), "Weg geöffnet", false);
        }
        let previous = self.previous();
        let required = 2;
        let done = g.regions.story_steps(previous);
        if done < required {
            let clue = previous.district().name();
            let name = match previous {
                Frontier::Moor => "Edda",
                Frontier::Copper => "Runa",
                Frontier::Wind => "Nele",
            };
            return (
                format!("Erledige zwei Aufträge von {name} in {clue} ({done}/{required})."),
                "Gesperrt",
                false,
            );
        }
        if !self.gate_accepted(g) {
            return (
                format!("{} · Details im Journal", self.gate_title()),
                "Auftrag annehmen",
                true,
            );
        }
        (
            craft::detail(self.gate_needs(), g, 1),
            "Weg herrichten",
            craft::ready(self.gate_needs(), g, 1),
        )
    }

    fn gate_accepted(self, g: &Garden) -> bool {
        g.expansion.gates[self as usize] == 1
    }

    pub fn work_gate(self, g: &mut Garden) -> Result<String, String> {
        let (hint, _, ready) = self.gate_status(g);
        if !ready {
            return Err(hint);
        }
        if !self.gate_accepted(g) {
            g.expansion.gates[self as usize] = 1;
            return Ok(format!("Auftrag angenommen · {}", self.gate_title()));
        }
        craft::take(self.gate_needs(), g, 1);
        g.expansion.gates[self as usize] = 2;
        Ok(format!(
            "Gebiet freigeschaltet · {}",
            self.district().name()
        ))
    }

    fn state(self, g: &Garden) -> u8 {
        g.expansion.orders[self as usize]
    }

    fn order(self, g: &Garden) -> Option<&'static Order> {
        ORDERS[self as usize].get((self.state(g) / 2) as usize)
    }

    pub fn steps(self, g: &Garden) -> usize {
        usize::from(self.state(g) / 2).min(ORDERS[0].len())
    }

    pub fn tracking(self, g: &Garden) -> Option<(&'static str, String, bool, &'static str)> {
        let at_gate = self.gate_accepted(g);
        let order = self.order(g).filter(|_| self.state(g) % 2 == 1);
        let (title, needs, receiver) = if at_gate {
            (self.gate_title(), self.gate_needs(), self.district().name())
        } else {
            let order = order?;
            (order.title, order.needs, self.npc().name())
        };
        Some((
            title,
            craft::detail(needs, g, 1),
            craft::ready(needs, g, 1),
            receiver,
        ))
    }

    pub fn mark(self, g: &Garden) -> Option<QuestMark> {
        let order = self.order(g).filter(|_| self.open(g))?;
        if self.state(g) % 2 == 0 {
            Some(QuestMark::Available)
        } else if craft::ready(order.needs, g, 1) {
            Some(QuestMark::TurnIn)
        } else {
            None
        }
    }

    pub fn current_title(self, g: &Garden) -> &'static str {
        if !self.open(g) {
            self.gate_title()
        } else {
            self.order(g)
                .map_or(self.district().name(), |order| order.title)
        }
    }

    pub fn journal(self, g: &Garden) -> String {
        if !self.open(g) {
            let (status, _, _) = self.gate_status(g);
            let progress = if self.gate_accepted(g) {
                status
            } else {
                craft::detail(self.gate_needs(), g, 1)
            };
            let prior = self.previous().district().name();
            return format!(
                "{}\n\n{}\n\nZugang über {prior} · Zwei Ortsaufträge\n{progress}",
                self.gate_title(),
                self.gate_text()
            );
        }
        let (text, _, _) = self.dialogue(g);
        let (bell, _, _) = self.bell().describe(g);
        format!("{text}\n\nWegglocke · {bell}")
    }

    pub fn target(self, g: &Garden) -> Option<Target> {
        if !self.open(g) {
            if !self.previous().unlocked(g) {
                return Some(Target::Gate(self.previous()));
            }
            if g.regions.story_steps(self.previous()) < 2 {
                let npc = [Npc::Runa, Npc::Nele, Npc::Edda][self.previous() as usize];
                return Some(Target::Person(npc));
            }
            return Some(Target::ExpansionGate(self));
        }
        if self.order(g).is_some() {
            Some(Target::Person(self.npc()))
        } else {
            (!self.bell().ready(g)).then_some(Target::Bell(self.bell()))
        }
    }

    pub fn dialogue(self, g: &Garden) -> (String, &'static str, bool) {
        let Some(order) = self.order(g).filter(|_| self.open(g)) else {
            return ([
                "Die Harzschalen sind wieder voll. Ich habe das Blätterbuch in der Hütte ausgelegt. Schau ruhig hinein, wenn du am Bach vorbeikommst.",
                "Wenn die Luft klar ist, kann ich von hier bis zu deinem Hof sehen. Dank unserer Sternkarte kenne ich jetzt auch die Gipfel dahinter.",
                "Bei der nächsten Ebbe klingt unser Lichtsteg wie ein Glockenspiel. Komm vorbei, wenn die Schatten aufs Wasser fallen.",
            ][self as usize].into(), "Abgeschlossen", false);
        };
        let accepted = self.state(g) % 2 == 1;
        let action = if accepted {
            "Auftrag abgeben"
        } else {
            "Auftrag annehmen"
        };
        let ready = !accepted || craft::ready(order.needs, g, 1);
        (
            format!(
                "{}\n{}\n{}\nBelohnung · {}",
                order.title,
                order.text,
                craft::detail(order.needs, g, 1),
                order.reward
            ),
            action,
            ready,
        )
    }

    pub fn finish(self, g: &mut Garden) -> Result<String, String> {
        let Some(order) = self.order(g).filter(|_| self.open(g)) else {
            return Err("Alle Ortsaufträge sind erledigt.".into());
        };
        let index = self as usize;
        if self.state(g) % 2 == 0 {
            g.expansion.orders[index] += 1;
            return Ok(format!("Auftrag angenommen · {}", order.title));
        }
        if !craft::ready(order.needs, g, 1) {
            return Err(craft::detail(order.needs, g, 1));
        }
        craft::take(order.needs, g, 1);
        let coins = order.coins;
        g.farm.coins = g.farm.coins.saturating_add(coins);
        match order.unlock {
            Unlock::Barn => g.workshop.barns[self.animal() as usize] = true,
            Unlock::Furniture(index) => {
                furniture::learn(g, index);
            }
        }
        g.expansion.orders[index] += 1;
        Ok(format!(
            "Auftrag erfüllt · {} · +{coins} Münzen · {}",
            order.title, order.reward
        ))
    }
}

pub fn starsteel(g: &Garden) -> bool {
    Region::Heights.steps(g) >= 2
}

enum Unlock {
    Barn,
    Furniture(usize),
}

struct Order {
    title: &'static str,
    text: &'static str,
    needs: &'static [Need],
    reward: &'static str,
    coins: u32,
    unlock: Unlock,
}

const fn goods(material: Material, quantity: u32) -> Need {
    Need::Stock(Product::Regional(material), quantity)
}

const fn boards(quantity: u32) -> Need {
    Need::Stock(Product::Crafted(Recipe::Boards), quantity)
}

use Material::*;
use furniture::EXPANSION_FURNITURE as F;

const ORDERS: [[Order; 4]; 3] = [
    [
        Order {
            title: "Die Harzspur",
            text: "Ivos Harzschalen sind nach einem Sturm verschwunden. Auf den warmen Steinen am Bach ist inzwischen wieder Goldharz ausgetreten. Sammle es ein, damit Ivo seine Laternen für den Waldweg wieder abdichten kann.",
            needs: &[goods(GoldResin, 5)],
            reward: "Bauplan für die Harzlaterne",
            coins: 22,
            unlock: Unlock::Furniture(F),
        },
        Order {
            title: "Ein Stall voller Ohren",
            text: "Wenn Ivo zur Dämmerung die Schale füllt, kommen sogar die scheuen Langohren aus dem Unterholz. Eines frisst schon aus seiner Hand. Bring Eisenholz und Bretter für einen Stall, der auch im Winter trocken bleibt.",
            needs: &[goods(Ironwood, 5), goods(GoldResin, 3), boards(3)],
            reward: "Kaninchenstall und Adoption",
            coins: 35,
            unlock: Unlock::Barn,
        },
        Order {
            title: "Decken für den ersten Frost",
            text: "Ivo hat im letzten Winter beim Nachtwachen unter drei kratzigen Decken geschlafen. Für die nächste Nacht am Harzpfad wünscht er sich eine aus Kaninchenflaum. Füttere und versorge deine Tiere. Nach zwei gefütterten Tagen kannst du den Flaum am Stall einsammeln.",
            needs: &[goods(RabbitDown, 4), goods(GoldResin, 4)],
            reward: "Bauplan für die Flaumbank",
            coins: 46,
            unlock: Unlock::Furniture(F + 1),
        },
        Order {
            title: "Das goldene Blätterbuch",
            text: "Im alten Blätterbuch fehlen die Farben der Bäume, die nach dem Sturm nachwachsen. Ivo will echte Blätter zwischen dünne Bernsteinscheiben legen. Such an der Westseite des Bachs nach Einschlüssen. Ein Sternstahlkopf bricht die Ader sauber auf.",
            needs: &[goods(Amber, 3), goods(Ironwood, 6), goods(RabbitDown, 2)],
            reward: "Bauplan für die Bernsteinvitrine",
            coins: 68,
            unlock: Unlock::Furniture(F + 2),
        },
    ],
    [
        Order {
            title: "Pfad am Seil",
            text: "Sanna faltet jeden Morgen ein Blatt Salbei in ihr Kopftuch. Seine Farbe zeigt ihr im Nebel, welche Steinplatten den ganzen Tag Sonne bekommen. Sammle Salbei an den unteren Terrassen. Mit Brettern setzt ihr solche Wegmarken auch für andere Wanderer.",
            needs: &[goods(AlpineSage, 5), boards(2)],
            reward: "Bauplan für den Salbeitrog",
            coins: 25,
            unlock: Unlock::Furniture(F + 3),
        },
        Order {
            title: "Funken im kalten Stein",
            text: "Die Ziegen haben oben am Geröllweg eine dunkle Ader freigescharrt. Sanna erkennt darin Sternenerz. Mit genug davon kann sie eine Schmiedevorlage zeichnen und den alten Felsstall ausbessern. Für die ersten Erzbrocken genügt eine Silberspitzhacke.",
            needs: &[goods(StarOre, 6), Need::Copper(5), boards(3)],
            reward: "Sternstahlwerkzeuge, Ziegenstall und Salbei-Ziegenkäse",
            coins: 42,
            unlock: Unlock::Barn,
        },
        Order {
            title: "Frühstück am Wolkenrand",
            text: "Vor der ersten Wanderung setzte Sannas Familie immer eine Kanne Ziegenmilch auf den Tisch. Seit der Sturm den Unterstand mitgenommen hat, ist der Tisch leer. Bring Milch von deinem Hof und etwas Salbei für frisches Brot.",
            needs: &[goods(GoatMilk, 4), goods(AlpineSage, 4)],
            reward: "Bauplan für die Almhüttenbank",
            coins: 55,
            unlock: Unlock::Furniture(F + 4),
        },
        Order {
            title: "Sternkarte im Giebel",
            text: "Sanna stellt im Herbst Kerzen in die Dachfenster. In klaren Nächten will sie den Lichtern Namen geben, damit Kinder die Sternbilder lernen. Für die neue Linse braucht sie Himmelsquarz von den oberen Graten. Nimm eine Sternstahlspitzhacke mit.",
            needs: &[goods(SkyQuartz, 5), goods(StarOre, 5), goods(GoatMilk, 2)],
            reward: "Bauplan für die Sternwarte-Lampe",
            coins: 78,
            unlock: Unlock::Furniture(F + 5),
        },
    ],
    [
        Order {
            title: "Wenn die Flut leise wird",
            text: "Tilda hört an der Brandung, wann die Tümpel wieder frei liegen. Für die Kinder am Anleger will sie überall dort, wo der Sand trocken bleibt, kleine Muschelzeichen setzen. Salzblüten halten die Farbe auch, wenn die Schilder nass werden.",
            needs: &[goods(SaltBloom, 5), Need::Stock(Product::Shell, 3)],
            reward: "Bauplan für das Gezeitenwindspiel",
            coins: 24,
            unlock: Unlock::Furniture(F + 6),
        },
        Order {
            title: "Gummistiefel für Enten",
            text: "Sobald Tilda die Werkstatt öffnet, laufen die Enten durch ihren Laden. Eine von ihnen hat sogar im Lackkübel gebadet. Auf deinem Hof hätten sie mehr Platz. Bring Perlmutt von den Felsbänken und ein paar Bretter. Daraus baut ihr trockene Nistboxen.",
            needs: &[goods(MotherPearl, 6), goods(SaltBloom, 3), boards(4)],
            reward: "Entenhaus, Adoption und Gezeiten-Omelett",
            coins: 39,
            unlock: Unlock::Barn,
        },
        Order {
            title: "Frühe Eier am Anleger",
            text: "Tilda hat den alten Ofen ausgefegt. Am nächsten Morgen möchte sie für die Sammler am Kai kleine Küchlein backen. Bring ihr Eier deiner Laufenten und Salzblüten für den Guss. Die Enten legen nach einem gefütterten Tag.",
            needs: &[goods(DuckEgg, 5), goods(SaltBloom, 5)],
            reward: "Bauplan für den Perlmuttkasten",
            coins: 52,
            unlock: Unlock::Furniture(F + 7),
        },
        Order {
            title: "Lichter am Seil",
            text: "In windstillen Nächten blieben die Boote früher draußen, um den Aufgang des Mondes zu sehen. Jetzt will Tilda die ersten Laternen am Anleger wieder anzünden. Riffholz trägt die Lampen. Mit Entenfedern prüft sie vor dem Fest, an welchen Stellen die Böen am stärksten sind.",
            needs: &[
                goods(Reefwood, 5),
                goods(DuckFeather, 4),
                goods(MotherPearl, 4),
            ],
            reward: "Bauplan für den Strandtafeltisch",
            coins: 72,
            unlock: Unlock::Furniture(F + 8),
        },
    ],
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        adventure::Adventure,
        homestead::Village,
        workshop::ToolKind,
        world::{Gardener, Reach},
    };

    fn stocked_garden() -> Garden {
        let mut garden = Garden::decode(br#"{"version":19,"wood":180,"stone":160,"copper":90,"regions":{"gates":[2,2,2],"stories":[4,4,4],"goods":[25,25,25],"bells":[2,0,0,0,2,2,2]},"workshop":{"restored":true,"forge":true,"barns":[true,true],"tools":[2,2,2,2]},"farm":{"coins":900,"housing":[true,true],"animals":[{},{}],"barn_work":[null,null],"barn_extended":[false,false]},"building_layout":[null,null,null,null,null]}"#).unwrap();
        garden.add_stock(Product::Crafted(Recipe::Boards), 80);
        garden.add_stock(Product::Shell, 20);
        garden
    }

    #[test]
    fn older_slots_and_new_access_quests_preserve_progress_independently() {
        let mut garden = stocked_garden();
        let adventure = Adventure::default();
        assert!(Bell::Farm.ready(&garden));
        assert!(garden.farm.housing[0]);
        assert!(!garden.farm.housing[Animal::Goat as usize]);
        assert_eq!(garden.regions.goods[GorgeQuartz as usize], 25);
        for region in Region::ALL {
            let gate = region.gate();
            let before = gate - region.direction() * 1.25;
            let after = gate + region.direction() * 1.25;
            assert!(adventure.walkable(before, &garden), "{region:?} approach");
            assert!(!adventure.walkable(after, &garden), "{region:?} lock");
            assert!(!region.bell().accessible(&garden));
            assert!(region.npc().quest_mark(&garden).is_none());
            assert!(!region.dialogue(&garden).2);
            region.work_gate(&mut garden).unwrap();
            assert!(region.tracking(&garden).unwrap().2);
            region.work_gate(&mut garden).unwrap();
            assert!(region.bell().accessible(&garden));
            assert!(
                adventure.walkable(after, &garden),
                "{region:?} opened approach"
            );
            let goal = region.district().waypoint();
            let mut hero = Gardener::new(before);
            assert!(
                hero.navigate(Reach::ground(goal), &adventure, &garden),
                "{region:?} way from gate to district heart"
            );
            assert_eq!(region.npc().quest_mark(&garden), Some(QuestMark::Available));
        }
        let restored = Garden::decode(&serde_json::to_vec(&garden).unwrap()).unwrap();
        assert!(Region::ALL.into_iter().all(|region| region.open(&restored)));
        assert!(Bell::Farm.ready(&restored));
    }

    #[test]
    fn each_district_unlocks_a_home_production_and_a_way_bell_through_its_quests() {
        let mut garden = stocked_garden();
        assert!(!ToolKind::Pickaxe.can_upgrade(&garden, true));
        for region in Region::ALL {
            region.work_gate(&mut garden).unwrap();
            region.work_gate(&mut garden).unwrap();
            let animal = region.animal();
            let bell = region.bell();
            let animal_index = animal as usize;
            for need in region.order(&garden).unwrap().needs {
                if let Need::Stock(Product::Regional(material), _) = *need {
                    garden.add_stock(Product::Regional(material), 50);
                }
            }
            bell.work(&mut garden).unwrap();
            bell.work(&mut garden).unwrap();
            assert!(bell.ready(&garden));
            region.finish(&mut garden).unwrap();
            assert_eq!(region.mark(&garden), Some(QuestMark::TurnIn));
            region.finish(&mut garden).unwrap();

            region.finish(&mut garden).unwrap();
            assert!(
                region.finish(&mut garden).is_err(),
                "{region:?} rejects short orders"
            );
            assert!(!garden.workshop.barns[animal_index]);
            for &tier_two in region.order(&garden).unwrap().needs {
                if let Need::Stock(Product::Regional(material), _) = tier_two {
                    garden.add_stock(Product::Regional(material), 80);
                }
            }
            region.finish(&mut garden).unwrap();
            assert!(garden.workshop.barns[animal_index]);
            garden.wood += 80;
            garden.stone += 60;
            animal.build_housing(&mut garden).unwrap();
            assert!(animal.adopt(&mut garden).is_err());
            while !garden.farm.housing[animal_index] {
                garden.sleep();
            }
            animal.adopt(&mut garden).unwrap();
            assert!(animal.adopt(&mut garden).is_err(), "one initial space");
            garden.farm.grass = 12;
            let pen = Village::Animal(animal);
            for _ in 0..2 {
                pen.work(&mut garden, crate::garden::Crop::Carrot, None)
                    .unwrap();
                garden.sleep();
                if garden.farm.animals[animal_index].has_products() {
                    break;
                }
            }
            pen.work(&mut garden, crate::garden::Crop::Carrot, None)
                .unwrap();
            assert!(
                garden.stock(animal.product()) > 0,
                "{region:?} daily production"
            );
        }
        garden.add_stock(Product::Silver, 20);
        ToolKind::Pickaxe.upgrade(&mut garden, true).unwrap();
        assert_eq!(ToolKind::Pickaxe.level(&garden), 3);
        assert!(garden.stock(Product::Regional(DuckFeather)) > 0);
    }

    #[test]
    fn advanced_ore_requires_sternstahl_and_regrows_after_harvest() {
        let mut garden = stocked_garden();
        let node = crate::forest::near(Vec2::new(274., -56.), 24.)
            .find(|node| node.material() == Some(SkyQuartz))
            .expect("Himmelsquarz baked into the terraces");
        assert!(garden.quarry(node.id).is_none());
        assert!(garden.stock(Product::Regional(SkyQuartz)) == 0);
        let mut saved = serde_json::to_value(&garden).unwrap();
        saved["workshop"]["tools"][ToolKind::Pickaxe as usize] = 3.into();
        garden = Garden::decode(&serde_json::to_vec(&saved).unwrap()).unwrap();
        assert!(garden.quarry(node.id).is_some());
        assert_eq!(garden.stock(Product::Regional(SkyQuartz)), 3);
        assert!(garden.quarry(node.id).is_none());
        for _ in 0..crate::garden::RESOURCE_RENEWAL_DAYS {
            garden.sleep();
        }
        assert!(garden.quarry(node.id).is_some());
    }
}
