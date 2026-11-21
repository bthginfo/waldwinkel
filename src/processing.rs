use crate::{
    garden::{Crop, Garden},
    homestead::Product,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Cook,
    Preserve,
    Mill,
}

impl Method {
    pub fn title(self) -> &'static str {
        match self {
            Self::Cook => "Kochen",
            Self::Preserve => "Einmachen",
            Self::Mill => "Mahlen",
        }
    }

    pub fn art(self) -> &'static str {
        match self {
            Self::Cook => "ui/station_cook.png",
            Self::Preserve => "ui/station_preserve.png",
            Self::Mill => "ui/station_mill.png",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dish {
    CarrotSoup,
    TomatoStew,
    BerryJam,
    StrawberryCompote,
    HazelFlour,
    SunflowerMeal,
    HarborBroth,
}

#[derive(Clone, Copy)]
enum Ingredient {
    Crop(Crop, u32),
    Stock(Product, u32),
    Fish,
}

impl Ingredient {
    fn requirement(self, garden: &Garden) -> (u32, u32, &'static str) {
        match self {
            Self::Fish => (garden.fish.saturating_add(garden.stock(Product::Herring)), 1, "Fisch oder Hafenhering"),
            Self::Crop(crop, count) => (garden.crop_total(crop), count, crop.name()),
            Self::Stock(item, count) => (garden.stock(item), count, item.name()),
        }
    }

    fn consume(self, garden: &mut Garden) {
        match self {
            Self::Fish => garden.take_stock(if garden.fish > 0 {Product::Fish} else {Product::Herring}, 1),
            Self::Crop(crop, count) => garden.take_crop(crop, count),
            Self::Stock(item, count) => garden.take_stock(item, count),
        };
    }
}

impl Dish {
    pub const ALL: [Self; 7] = [
        Self::CarrotSoup,
        Self::TomatoStew,
        Self::BerryJam,
        Self::StrawberryCompote,
        Self::HazelFlour,
        Self::SunflowerMeal,
        Self::HarborBroth,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::HarborBroth => "Bos Hafenbrühe",
            Self::CarrotSoup => "Möhren-Pilz-Suppe",
            Self::TomatoStew => "Tomateneintopf",
            Self::BerryJam => "Waldbeerkonfitüre",
            Self::StrawberryCompote => "Erdbeerkompott",
            Self::HazelFlour => "Haselnussmehl",
            Self::SunflowerMeal => "Sonnenblumenschrot",
        }
    }

    pub fn method(self) -> Method {
        match self {
            Self::CarrotSoup | Self::TomatoStew | Self::HarborBroth => Method::Cook,
            Self::BerryJam | Self::StrawberryCompote => Method::Preserve,
            Self::HazelFlour | Self::SunflowerMeal => Method::Mill,
        }
    }

    pub fn value(self) -> u32 {
        [13, 18, 10, 14, 13, 15, 14][self as usize]
    }

    fn ingredients(self) -> &'static [Ingredient] {
        use Ingredient::{Crop as Field, Stock};
        match self {
            Self::HarborBroth => &[Ingredient::Fish, Field(Crop::Carrot, 1)],
            Self::CarrotSoup => &[Field(Crop::Carrot, 2), Stock(Product::Mushroom, 1)],
            Self::TomatoStew => &[Field(Crop::Tomato, 2), Stock(Product::Egg, 1)],
            Self::BerryJam => &[Stock(Product::Berry, 2), Stock(Product::Apple, 1)],
            Self::StrawberryCompote => &[Field(Crop::Strawberry, 2), Stock(Product::Hazel, 1)],
            Self::HazelFlour => &[Stock(Product::Hazel, 3)],
            Self::SunflowerMeal => &[Field(Crop::Sunflower, 2)],
        }
    }

    pub fn supplies(self, garden: &Garden) -> String {
        self.ingredients()
            .iter()
            .map(|ingredient| {
                let (stock, needed, name) = ingredient.requirement(garden);
                format!("{}/{} {name}", stock.min(needed), needed)
            })
            .collect::<Vec<_>>()
            .join(" · ")
    }

    pub fn ready(self, garden: &Garden) -> bool {
        (self != Self::HarborBroth || garden.fish_supper.learned()) && self.ingredients().iter().all(|ingredient| {
            let (stock, count, _) = ingredient.requirement(garden);
            stock >= count
        })
    }

    fn seasoning(self) -> [usize; 3] {
        match self {
            Self::BerryJam => [0, 2, 1],
            _ => [2, 1, 0],
        }
    }
}

// The seventh dish was added in save version 17. Keep the six existing indices.
pub(crate) fn read_pantry<'de, D: serde::Deserializer<'de>>(reader: D) -> Result<[u32; Dish::ALL.len()], D::Error> {
    use serde::{Deserialize, de::Error};
    let amounts = Vec::<u32>::deserialize(reader)?;
    if !(6..=Dish::ALL.len()).contains(&amounts.len()) {
        return Err(D::Error::custom("Ungültiger Küchenvorrat"));
    }
    let mut pantry = [0; Dish::ALL.len()];
    pantry[..amounts.len()].copy_from_slice(&amounts);
    Ok(pantry)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Chop,
    Simmer,
    Garnish,
    Fill,
    Seal,
    Grind,
    Done,
}

pub struct Preparation {
    pub dish: Dish,
    progress: usize,
    last_step: Option<f32>,
    level: f32,
    infusion: f32,
    stirs: u8,
    steady_turns: u8,
    mistakes: u8,
    pub message: &'static str,
}

impl Dish {
    fn cuts(self) -> [(usize, &'static str); 3] {
        match self {
            Self::CarrotSoup => [(0, "Möhre würfeln · Fein schneiden"), (1, "Pilz · Grob teilen"), (0, "Möhre würfeln · Fein schneiden")],
            Self::TomatoStew => [(1, "Tomate · Grob teilen"), (1, "Tomate · Grob teilen"), (0, "Kräuter · Fein schneiden")],
            _ => [(0, "Möhre würfeln · Fein schneiden"), (1, "Fisch · Grob teilen"), (0, "Kräuter · Fein schneiden")],
        }
    }

    fn garnish(self) -> (usize, &'static str) {
        match self {
            Self::HarborBroth => (1, "Der kräftigen Brühe fehlt etwas Frisches."),
            Self::TomatoStew => (2, "Geröstetes rundet die Säure der Tomaten ab."),
            _ => (0, "Zu den erdigen Pilzen passt etwas Würziges."),
        }
    }
}

impl Preparation {
    pub fn new(dish: Dish) -> Self {
        Self {
            dish,
            progress: 0,
            last_step: None,
            level: 0.,
            infusion: 0.,
            stirs: 0,
            steady_turns: 0,
            mistakes: 0,
            message: "",
        }
    }

    pub fn phase(&self) -> Phase {
        match self.dish.method() {
            Method::Cook => match self.progress {
                0..=2 => Phase::Chop,
                3 => Phase::Simmer,
                4 => Phase::Garnish,
                _ => Phase::Done,
            },
            Method::Preserve if self.progress < 3 => Phase::Fill,
            Method::Preserve if self.progress == 3 => Phase::Seal,
            Method::Mill if self.progress < self.turns() => Phase::Grind,
            _ => Phase::Done,
        }
    }

    fn turns(&self) -> usize {
        if self.dish == Dish::SunflowerMeal { 8 } else { 6 }
    }

    pub fn title(&self) -> String {
        match self.phase() {
            Phase::Chop => format!("Schneidebrett · {}/3", self.progress + 1),
            Phase::Simmer => format!("Herd · {} % Gargrad · {}/2 gerührt",
                (self.infusion * 100.).floor() as u32, self.stirs.min(2)),
            Phase::Garnish => "Anrichten · Abschmecken".into(),
            Phase::Fill => format!("Einmachen · Schicht {}/3", self.progress + 1),
            Phase::Seal => "Einmachen · Glas entlüften".into(),
            Phase::Grind => format!("Handmühle · {}/{} Umdrehungen", self.progress, self.turns()),
            Phase::Done => "Fertig".into(),
        }
    }

    pub fn instructions(&self) -> &'static str {
        match self.phase() {
            Phase::Chop => self.dish.cuts()[self.progress].1,
            Phase::Simmer => "Halte die Glut im hellen Bereich und rühre mindestens zweimal.",
            Phase::Garnish => self.dish.garnish().1,
            Phase::Fill => match self.dish.seasoning()[self.progress] {
                0 => "Gib den frischen Ansatz ins Glas.",
                1 => "Fülle den süßen Sirup ein.",
                _ => "Gib die würzige Schicht dazu.",
            },
            Phase::Seal => "Drücke die Luft heraus. Verschließe das Glas im hellen Bereich.",
            Phase::Grind if self.progress.is_multiple_of(2) =>
                "Drehe nach links. Gleichmäßiges Kurbeln hält das Mahlwerk in Schwung.",
            Phase::Grind =>
                "Drehe nach rechts. Gleichmäßiges Kurbeln hält das Mahlwerk in Schwung.",
            Phase::Done => "",
        }
    }

    pub fn buttons(&self) -> &'static [&'static str] {
        match self.phase() {
            Phase::Chop => &["Fein schneiden", "Grob teilen"],
            Phase::Simmer => &["Anfachen", "Rühren"],
            Phase::Garnish => &["Waldkräuter", "Hafenzeste", "Röstkerne"],
            Phase::Fill => &["Frisch", "Süß", "Würzig"],
            Phase::Seal => &["Glas schließen", "Entlüften"],
            Phase::Grind => &["Links drehen", "Rechts drehen"],
            Phase::Done => &[],
        }
    }

    pub fn action_hint(&self, index: usize) -> &'static str {
        match (self.phase(), index) {
            (Phase::Simmer, 0) => "Glut erhöhen",
            (Phase::Simmer, 1) => "Garen und abkühlen",
            (Phase::Garnish, 0) => "Würzig",
            (Phase::Garnish, 1) => "Frisch",
            (Phase::Garnish, 2) => "Mild",
            (Phase::Seal, 1) => "Luft herausdrücken",
            (Phase::Grind, i) if i == self.progress % 2 => "Jetzt drehen",
            _ => "",
        }
    }

    pub fn gauge(&self) -> Option<(f32, std::ops::RangeInclusive<f32>)> {
        let range = match self.phase() {
            Phase::Simmer => match self.dish {
                Dish::TomatoStew => 0.48..=0.82,
                Dish::HarborBroth => 0.40..=0.70,
                _ => 0.37..=0.72,
            },
            Phase::Seal => 0.0..=0.24,
            Phase::Grind => 0.30..=0.83,
            _ => return None,
        };
        Some((self.level, range))
    }

    pub fn advance(&mut self, delta: f32) {
        match self.phase() {
            Phase::Simmer => {
                let (heat, range) = self.gauge().unwrap();
                self.level = (heat - delta * 0.06).max(0.18);
                if range.contains(&self.level) && self.stirs > 0 {
                    self.infusion = (self.infusion + delta * 0.42).min(1.);
                }
            }
            Phase::Grind => self.level = (self.level - delta * 0.21).max(0.),
            _ => {}
        }
    }

    pub fn illustration(&self) -> String {
        match self.phase() {
            Phase::Chop if self.dish == Dish::HarborBroth => "ui/broth_board.png".into(),
            Phase::Simmer => "ui/broth_pot.png".into(),
            Phase::Garnish if self.dish == Dish::HarborBroth => "ui/broth_bowls.png".into(),
            Phase::Garnish => format!("ui/items/{}.png",
                crate::inventory::product_icon(Product::Prepared(self.dish))),
            _ => self.dish.method().art().into(),
        }
    }

    pub fn pulse(&self, now: f32) -> f32 {
        self.last_step.map_or(0., |at| (1. - (now - at) / 0.38).clamp(0., 1.).powi(2))
    }

    pub fn try_step(&mut self, input: usize, now: f32) -> bool {
        if input >= self.buttons().len() || self.last_step.is_some_and(|at| now - at < 0.16) {
            return false;
        }
        self.last_step = Some(now);
        self.message = "";
        match self.phase() {
            Phase::Chop if input == self.dish.cuts()[self.progress].0 => {
                self.progress += 1;
                if self.phase() == Phase::Simmer {
                    self.level = 0.26;
                }
            }
            Phase::Chop => return self.miss("Schau auf den Schnitt am Brett und versuch es noch einmal."),
            Phase::Simmer if input == 0 => {
                self.level = (self.level + 0.29).min(1.);
                if self.level > self.gauge().unwrap().1.end() + 0.075 {
                    self.mistakes = self.mistakes.saturating_add(1);
                    self.message = "Zu heiß · Rühre, um den Topf abzukühlen.";
                }
            }
            Phase::Simmer => {
                let (heat, range) = self.gauge().unwrap();
                if heat < *range.start() {
                    self.message = "Die Glut ist zu schwach · Fache sie an.";
                } else {
                    self.level = (heat - 0.13).max(0.18);
                    if range.contains(&heat) {
                        self.stirs = self.stirs.saturating_add(1);
                        self.infusion = (self.infusion + 0.22).min(1.);
                        if self.stirs >= 2 && self.infusion >= 1. {
                            self.progress += 1;
                        }
                    } else {
                        self.message = "Der Topf kühlt ab · Rühre bei mittlerer Glut weiter.";
                    }
                }
            }
            Phase::Garnish => {
                if input != self.dish.garnish().0 {
                    self.mistakes = self.mistakes.saturating_add(1);
                }
                self.progress += 1;
            }
            Phase::Fill if input == self.dish.seasoning()[self.progress] => {
                self.progress += 1;
                if self.phase() == Phase::Seal {
                    self.level = 0.84;
                }
            }
            Phase::Fill => return self.miss("Die nächste Schicht steht über dem Glas."),
            Phase::Seal if input == 1 => self.level = (self.level - 0.33).max(0.),
            Phase::Seal if self.level <= 0.24 => self.progress += 1,
            Phase::Seal => return self.miss("Im Glas ist noch Luft. Entlüfte es erst."),
            Phase::Grind if input == self.progress % 2 => {
                self.level = (self.level + 0.20).min(1.);
                if self.gauge().unwrap().1.contains(&self.level) {
                    self.steady_turns += 1;
                }
                self.progress += 1;
            }
            Phase::Grind => {
                self.level *= 0.7;
                return self.miss("Wechsle die Seite, damit das Mahlwerk weiterläuft.");
            }
            Phase::Done => return false,
        }
        true
    }

    fn miss(&mut self, message: &'static str) -> bool {
        self.mistakes = self.mistakes.saturating_add(1);
        self.message = message;
        false
    }

    pub fn finish(&mut self, garden: &mut Garden) -> Option<u32> {
        let end = match self.dish.method() {
            Method::Cook => 5,
            Method::Preserve => 4,
            Method::Mill => self.turns(),
        };
        if self.progress != end || !self.dish.ready(garden) {
            return None;
        }
        let fine = self.dish.ingredients().iter().any(|ingredient| {
            matches!(*ingredient, Ingredient::Crop(crop, count)
                if garden.stock(Product::Crop(crop, crate::garden::Quality::Normal)) < count)
        });
        let practiced = crate::logbook::Path::Kitchen.rank(garden) >= 2;
        let method = self.dish.method();
        let clean = self.mistakes == 0 || (fine || practiced) && self.mistakes <= 1;
        let servings = 1 + u32::from(clean && (method != Method::Mill || self.steady_turns >= 3));
        for ingredient in self.dish.ingredients() {
            ingredient.consume(garden);
        }
        garden.add_stock(Product::Prepared(self.dish), servings);
        garden.logbook.meal(self.dish);
        if self.dish == Dish::HarborBroth {
            garden.fish_supper.finish_first_meal(garden.day, garden.clock.minute());
        } else if let Some(prologue) = &mut garden.prologue {
            prologue.cooked = true;
        }
        self.progress += 1;
        Some(servings)
    }

    #[cfg(test)]
    pub(crate) fn cook_to_plate_for_test(&mut self) -> f32 {
        let mut now = 0.5;
        for (action, _) in self.dish.cuts() {
            assert!(self.try_step(action, now));
            now += 0.5;
        }
        for _ in 0..30 {
            let (heat, range) = self.gauge().unwrap();
            let action = usize::from(heat >= *range.start());
            assert!(self.try_step(action, now));
            if self.phase() == Phase::Garnish {
                return now + 0.5;
            }
            self.advance(0.5);
            now += 0.5;
        }
        panic!("the stove must eventually finish");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garden::Quality;

    #[test]
    fn cooking_rewards_three_different_tasks_and_uses_ingredients_once() {
        let mut garden = Garden::default();
        garden.add_stock(Product::Crop(Crop::Carrot, Quality::Normal), 2);
        garden.add_stock(Product::Mushroom, 1);
        let mut game = Preparation::new(Dish::CarrotSoup);
        assert_eq!(game.finish(&mut garden), None);
        assert_eq!(garden.crop_total(Crop::Carrot), 2);
        let now = game.cook_to_plate_for_test();
        assert!(game.try_step(0, now));
        assert_eq!(game.finish(&mut garden), Some(2));
        assert_eq!(game.finish(&mut garden), None);
        assert_eq!(garden.stock(Product::Prepared(Dish::CarrotSoup)), 2);
        assert_eq!(garden.crop_total(Crop::Carrot), 0);
    }

    #[test]
    fn heat_requires_active_stirring_and_the_jar_requires_venting() {
        let mut soup = Preparation::new(Dish::CarrotSoup);
        for (step, time) in [(0, 0.), (1, 0.5), (0, 1.)] {
            assert!(soup.try_step(step, time));
        }
        assert!(soup.try_step(0, 1.5));
        soup.advance(1.);
        assert_eq!(soup.infusion, 0.);
        assert_eq!(soup.phase(), Phase::Simmer);

        let mut jar = Preparation::new(Dish::BerryJam);
        for (step, time) in [(0, 0.), (2, 0.5), (1, 1.)] {
            assert!(jar.try_step(step, time));
        }
        assert!(!jar.try_step(0, 1.5));
        assert!(jar.try_step(1, 2.));
        assert!(jar.try_step(1, 2.5));
        assert!(jar.try_step(0, 3.));
        assert_eq!(jar.phase(), Phase::Done);

        let mut mill = Preparation::new(Dish::HazelFlour);
        for i in 0..6 {
            assert!(mill.try_step(i % 2, i as f32 * 0.45));
            mill.advance(0.45);
        }
        assert_eq!(mill.phase(), Phase::Done);
        assert!(mill.steady_turns >= 3);
    }
}
