use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    craft::Recipe,
    duel::Card,
    garden::{Crop, Garden},
    homestead::Product,
    inventory::{Material, crop_icon},
    seasons::MEETING_POINT,
};

pub const TITLE: &str = "Ein Garten für Eichenhafen";
pub const SITE: Vec2 = Vec2::new(MEETING_POINT.x - 0.45, MEETING_POINT.y - 1.95);
pub const HALF: Vec2 = Vec2::new(1.12, 0.48);

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum Stage {
    #[default]
    Ground,
    Beds,
    Pump,
    Bloom,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct TownGarden {
    pub stage: Stage,
    delivered: [u32; 2],
    basket_day: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Supply {
    Wood,
    Carrot,
    Stone,
    Boards,
    Sunflower,
    Rivets,
}

impl Supply {
    pub fn name(self) -> &'static str {
        match self {
            Self::Wood => "Holz",
            Self::Carrot => "Möhren",
            Self::Stone => "Stein",
            Self::Boards => "Bretter",
            Self::Sunflower => "Sonnenblumen",
            Self::Rivets => "Kupfernieten",
        }
    }

    pub fn source(self) -> &'static str {
        match self {
            Self::Wood => "Fälle einen Baum mit der Axt.",
            Self::Carrot => "Ernte Möhren auf deinem Hof.",
            Self::Stone => "Baue einen Findling mit der Spitzhacke ab.",
            Self::Boards => "Verarbeite Holz an der Hofwerkbank.",
            Self::Sunflower => "Besorge Sonnenblumensamen und ziehe die Blumen auf deinem Hof.",
            Self::Rivets => {
                "Fertige Kupfernieten an der Schmiede. Den Plan findest du an der Kupferwacht."
            }
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Wood => Material::Wood.icon(),
            Self::Stone => Material::Stone.icon(),
            Self::Carrot => crop_icon(Crop::Carrot),
            Self::Sunflower => crop_icon(Crop::Sunflower),
            Self::Boards => Recipe::Boards.icon(),
            Self::Rivets => Recipe::Rivets.icon(),
        }
    }

    fn stock(self, g: &Garden) -> u32 {
        match self {
            Self::Wood => g.wood,
            Self::Stone => g.stone,
            Self::Carrot => g.crop_total(Crop::Carrot),
            Self::Sunflower => g.crop_total(Crop::Sunflower),
            Self::Boards => g.stock(Product::Crafted(Recipe::Boards)),
            Self::Rivets => g.stock(Product::Crafted(Recipe::Rivets)),
        }
    }

    fn take(self, g: &mut Garden, amount: u32) {
        match self {
            Self::Wood => g.wood -= amount,
            Self::Stone => g.stone -= amount,
            Self::Carrot => {
                g.take_crop(Crop::Carrot, amount);
            }
            Self::Sunflower => {
                g.take_crop(Crop::Sunflower, amount);
            }
            Self::Boards => {
                g.take_stock(Product::Crafted(Recipe::Boards), amount);
            }
            Self::Rivets => {
                g.take_stock(Product::Crafted(Recipe::Rivets), amount);
            }
        }
    }
}

impl Stage {
    pub fn requests(self) -> &'static [(Supply, u32)] {
        use Supply::*;
        match self {
            Self::Ground => &[(Wood, 5), (Carrot, 3)],
            Self::Beds => &[(Stone, 8), (Boards, 4)],
            Self::Pump => &[(Sunflower, 4), (Rivets, 3)],
            Self::Bloom => &[],
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Ground => "1 · Linas Pflanzkisten",
            Self::Beds => "2 · Wasser für die Beete",
            Self::Pump => "3 · Licht für die Abendblüher",
            Self::Bloom => "Der blühende Dorfplatz",
        }
    }

    pub fn story(self) -> &'static str {
        match self {
            Self::Ground => {
                "Lina hat einen sonnigen Platz am Dorftisch gefunden. Bring Holz für ihre Kisten und Möhren für das erste Beet."
            }
            Self::Beds => {
                "Im Sommer wird das Gießen mühsam. Bo möchte eine Schwengelpumpe auf ein festes Fundament setzen."
            }
            Self::Pump => {
                "Mara wünscht sich einen Platz zum Zusammensitzen. Bring Blumen und Nieten für das Laternenspalier."
            }
            Self::Bloom => {
                "Jetzt treffen sich die Nachbarn im Garten. Alle drei Tage legen sie Saatgut und Dünger für deinen Hof zurück."
            }
        }
    }

    pub fn reward(self) -> &'static str {
        match self {
            Self::Ground => "4 Dünger und 3 Rübensamen",
            Self::Beds => "1 Talentpunkt und 4 Erdbeersamen",
            Self::Pump => "Karte Blütenwacht und der Erntekorb",
            Self::Bloom => "3 Sonnenblumensamen und 2 Dünger",
        }
    }

    pub fn art(self) -> &'static str {
        match self {
            Self::Ground => "ui/town_seedlings.png",
            Self::Beds => "ui/town_pump.png",
            Self::Pump | Self::Bloom => "ui/town_lanterns.png",
        }
    }

    pub fn model(self) -> Option<&'static str> {
        match self {
            Self::Ground => None,
            Self::Beds => Some("town_planter"),
            Self::Pump => Some("town_pump"),
            Self::Bloom => Some("town_pergola"),
        }
    }
}

impl TownGarden {
    fn slot(&self, supply: Supply) -> Option<(usize, u32)> {
        self.stage
            .requests()
            .iter()
            .enumerate()
            .find_map(|(i, &(item, need))| (item == supply).then_some((i, need)))
    }

    pub fn fraction(&self, supply: Supply) -> f32 {
        self.slot(supply).map_or(0., |(i, need)| {
            self.delivered[i].min(need) as f32 / need as f32
        })
    }

    pub fn amount(&self, supply: Supply, g: &Garden) -> u32 {
        self.slot(supply).map_or(0, |(i, need)| {
            need.saturating_sub(self.delivered[i]).min(supply.stock(g))
        })
    }

    pub fn detail(&self, supply: Supply, g: &Garden) -> String {
        self.slot(supply).map_or_else(String::new, |(i, need)| {
            format!(
                "{} / {need} abgegeben · {} im Gepäck",
                self.delivered[i].min(need),
                supply.stock(g)
            )
        })
    }

    pub fn basket_ready(&self, day: u32) -> bool {
        self.stage == Stage::Bloom && day >= self.basket_day
    }

    pub fn summary(&self, day: u32) -> String {
        if self.stage == Stage::Bloom {
            return if self.basket_ready(day) {
                "Am Dorftisch wartet dein Erntekorb.".into()
            } else {
                format!("Der nächste Erntekorb wartet ab Tag {}.", self.basket_day)
            };
        }
        format!(
            "{} · Am Dorftisch abgeben: {}",
            self.stage.title(),
            self.stage
                .requests()
                .iter()
                .enumerate()
                .map(|(i, (s, n))| format!("{}/{} {}", self.delivered[i].min(*n), n, s.name()))
                .collect::<Vec<_>>()
                .join(" · ")
        )
    }
}

pub fn deliver(g: &mut Garden, supply: Supply) -> Result<String, String> {
    let project = g.town_garden.stage;
    let (i, _) = g
        .town_garden
        .slot(supply)
        .ok_or("Für dieses Bauvorhaben wird das nicht gebraucht.")?;
    let amount = g.town_garden.amount(supply, g);
    if amount == 0 {
        return Err(supply.source().into());
    }
    supply.take(g, amount);
    g.town_garden.delivered[i] += amount;
    if !project
        .requests()
        .iter()
        .enumerate()
        .all(|(j, (_, n))| g.town_garden.delivered[j] >= *n)
    {
        return Ok(format!(
            "{amount} {} am Dorftisch abgegeben.",
            supply.name()
        ));
    }
    g.town_garden.delivered = [0; 2];
    g.town_garden.stage = match project {
        Stage::Ground => {
            g.fertilizer = g.fertilizer.saturating_add(4);
            g.add_seeds(Crop::Turnip, 3);
            Stage::Beds
        }
        Stage::Beds => {
            g.rpg.points = g.rpg.points.saturating_add(1);
            g.add_seeds(Crop::Strawberry, 4);
            Stage::Pump
        }
        Stage::Pump => {
            g.rpg.learn(Card::BloomWard);
            g.town_garden.basket_day = g.day;
            Stage::Bloom
        }
        Stage::Bloom => unreachable!(),
    };
    Ok(format!("{} fertig · {}", project.title(), project.reward()))
}

pub fn gather(g: &mut Garden) -> Result<String, String> {
    if !g.town_garden.basket_ready(g.day) {
        return Err(g.town_garden.summary(g.day));
    }
    g.add_seeds(Crop::Sunflower, 3);
    g.fertilizer = g.fertilizer.saturating_add(2);
    g.town_garden.basket_day = g.day.saturating_add(3);
    Ok(format!("Erntekorb abgeholt · {}", Stage::Bloom.reward()))
}

#[derive(Component)]
pub struct WindLamp(pub f32);

pub fn sway(time: Res<Time>, mut lamps: Query<(&WindLamp, &mut Transform)>) {
    for (lamp, mut pose) in &mut lamps {
        let beat = time.elapsed_secs() * 1.6 + lamp.0;
        pose.rotation = Quat::from_rotation_z(beat.sin() * 0.078)
            * Quat::from_rotation_x((beat * 0.53).sin() * 0.032);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garden::Quality;

    fn reload(g: &Garden) -> Garden {
        Garden::decode(&serde_json::to_vec(g).unwrap()).unwrap()
    }

    #[test]
    fn a_neighborhood_garden_grows_from_individual_deliveries_and_returns_a_basket() {
        let mut g = Garden::default();
        assert!(deliver(&mut g, Supply::Boards).is_err());
        assert!(gather(&mut g).is_err());
        g.wood = 2;
        deliver(&mut g, Supply::Wood).unwrap();
        let mut g = reload(&g);
        assert_eq!(g.wood, 0);
        assert_eq!(g.town_garden.fraction(Supply::Wood), 0.4);
        g.wood = 9;
        deliver(&mut g, Supply::Wood).unwrap();
        assert_eq!(g.wood, 6);
        assert!(deliver(&mut g, Supply::Wood).is_err());
        let fertilizer = g.fertilizer;
        g.add_stock(Product::Crop(Crop::Carrot, Quality::Fine), 3);
        deliver(&mut g, Supply::Carrot).unwrap();
        assert_eq!(g.town_garden.stage, Stage::Beds);
        assert_eq!(g.fertilizer, fertilizer + 4);
        g.stone = 8;
        g.add_stock(Product::Crafted(Recipe::Boards), 4);
        let points = g.rpg.points;
        deliver(&mut g, Supply::Boards).unwrap();
        deliver(&mut g, Supply::Stone).unwrap();
        assert_eq!(g.rpg.points, points + 1);
        assert_eq!(g.town_garden.stage, Stage::Pump);
        let deck = g.rpg.deck();
        g.add_stock(Product::Crop(Crop::Sunflower, Quality::Normal), 4);
        g.add_stock(Product::Crafted(Recipe::Rivets), 3);
        deliver(&mut g, Supply::Sunflower).unwrap();
        deliver(&mut g, Supply::Rivets).unwrap();
        let mut g = reload(&g);
        assert_eq!(g.town_garden.stage, Stage::Bloom);
        assert!(g.rpg.knows(Card::BloomWard));
        assert_eq!(g.rpg.deck(), deck);
        let seeds = g.seed_count(Crop::Sunflower);
        gather(&mut g).unwrap();
        assert_eq!(g.seed_count(Crop::Sunflower), seeds + 3);
        let mut g = reload(&g);
        assert!(gather(&mut g).is_err());
        g.sleep();
        g.sleep();
        assert!(gather(&mut g).is_err());
        g.sleep();
        assert!(gather(&mut g).is_ok());
    }

    #[test]
    fn an_old_save_keeps_its_collection_deck_and_town_orders() {
        let mut g = Garden::default();
        g.rpg.learn(Card::MoonSlice);
        let mut old = serde_json::to_value(&g).unwrap();
        old["version"] = 17.into();
        old.as_object_mut().unwrap().remove("town_garden");
        old["rpg"]["cards"].as_array_mut().unwrap().pop();
        let migrated = Garden::decode(&serde_json::to_vec(&old).unwrap()).unwrap();
        assert_eq!(migrated.town_garden.stage, Stage::Ground);
        assert!(!migrated.rpg.knows(Card::BloomWard));
        assert!(migrated.rpg.knows(Card::MoonSlice));
        assert_eq!(g.rpg.deck(), migrated.rpg.deck());
    }
}
