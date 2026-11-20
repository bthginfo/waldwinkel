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
}

#[derive(Clone, Copy)]
enum Ingredient {
    Crop(Crop, u32),
    Stock(Product, u32),
}

impl Ingredient {
    fn requirement(self, garden: &Garden) -> (u32, u32, &'static str) {
        match self {
            Self::Crop(crop, count) => (garden.crop_total(crop), count, crop.name()),
            Self::Stock(item, count) => (garden.stock(item), count, item.name()),
        }
    }

    fn consume(self, garden: &mut Garden) {
        match self {
            Self::Crop(crop, count) => garden.take_crop(crop, count),
            Self::Stock(item, count) => garden.take_stock(item, count),
        };
    }
}

impl Dish {
    pub const ALL: [Self; 6] = [
        Self::CarrotSoup,
        Self::TomatoStew,
        Self::BerryJam,
        Self::StrawberryCompote,
        Self::HazelFlour,
        Self::SunflowerMeal,
    ];

    pub fn name(self) -> &'static str {
        match self {
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
            Self::CarrotSoup | Self::TomatoStew => Method::Cook,
            Self::BerryJam | Self::StrawberryCompote => Method::Preserve,
            Self::HazelFlour | Self::SunflowerMeal => Method::Mill,
        }
    }

    pub fn value(self) -> u32 {
        [13, 18, 10, 14, 13, 15][self as usize]
    }

    fn ingredients(self) -> &'static [Ingredient] {
        use Ingredient::{Crop as Field, Stock};
        match self {
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
        self.ingredients().iter().all(|ingredient| {
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

pub struct Preparation {
    pub dish: Dish,
    progress: usize,
    start: f32,
    last_cycle: i32,
    last_step: Option<f32>,
    mistakes: u8,
    pub message: &'static str,
}

impl Preparation {
    const PERIOD: f32 = 3.2;
    const HEAT_RANGE: std::ops::RangeInclusive<f32> = 0.36..=0.73;
    const SEAL_RANGE: std::ops::RangeInclusive<f32> = 0.52..=0.86;
    const SEASONING: [&'static str; 3] = ["Zitrone", "Honig", "Kräuter"];

    pub fn new(dish: Dish, now: f32) -> Self {
        Self {
            dish,
            start: now,
            progress: 0,
            last_cycle: -1,
            last_step: None,
            mistakes: 0,
            message: "",
        }
    }

    pub fn heat(&self, now: f32) -> f32 {
        let phase = ((now - self.start) / Self::PERIOD).fract();
        1.0 - (phase * 2.0 - 1.0).abs()
    }

    pub fn instructions(&self) -> String {
        match self.dish.method() {
            Method::Cook if self.progress == 0 => "Schneide die Zutaten und gib sie in den Topf.".into(),
            Method::Cook if self.progress < 4 =>
                "Rühre einmal pro Runde im grünen Bereich. Warte, bis der Zeiger wieder zurückläuft.".into(),
            Method::Cook => "Schöpfe die fertige Suppe ab.".into(),
            Method::Preserve if self.progress < 3 => {
                let [a, b, c] = self.dish.seasoning().map(|index| Self::SEASONING[index]);
                format!("Fülle das Glas in dieser Reihenfolge. {a} → {b} → {c}")
            }
            Method::Preserve => "Verschließe das Glas, sobald der Zeiger im grünen Bereich ist.".into(),
            Method::Mill => format!(
                "Kurble links und rechts im Wechsel. Als Nächstes {}.",
                if self.progress % 2 == 0 { "links" } else { "rechts" }
            ),
        }
    }

    pub fn buttons(&self) -> &'static [&'static str] {
        match self.dish.method() {
            Method::Cook if self.progress == 0 => &["Schneiden und zugeben"],
            Method::Cook if self.progress < 4 => &["Rühren"],
            Method::Cook => &["Abschöpfen"],
            Method::Preserve if self.progress < 3 => &Self::SEASONING,
            Method::Preserve => &["Glas verschließen"],
            Method::Mill => &["Links drehen", "Rechts drehen"],
        }
    }

    pub fn progress(&self) -> (usize, usize) {
        let total = match self.dish.method() {
            Method::Cook => 5,
            Method::Preserve => 4,
            Method::Mill => 6,
        };
        (self.progress, total)
    }

    pub fn timing_range(&self) -> Option<std::ops::RangeInclusive<f32>> {
        match self.dish.method() {
            Method::Cook if (1..4).contains(&self.progress) => Some(Self::HEAT_RANGE),
            Method::Preserve if self.progress == 3 => Some(Self::SEAL_RANGE),
            _ => None,
        }
    }

    pub fn pulse(&self, now: f32) -> f32 {
        self.last_step.map_or(0., |at| (1. - (now-at)/0.38).clamp(0., 1.).powi(2))
    }

    pub fn try_step(&mut self, input: usize, now: f32) -> bool {
        if self.progress().0 >= self.progress().1 {
            return false;
        }
        if self.last_step.is_some_and(|at| now-at < 0.23) {
            return false;
        }
        let success = match self.dish.method() {
            Method::Cook if self.timing_range().is_some() => {
                let cycle = ((now - self.start) / Self::PERIOD).floor() as i32;
                if cycle == self.last_cycle {
                    self.message = "Warte auf die nächste Runde.";
                    return false;
                }
                let good = input == 0 && Self::HEAT_RANGE.contains(&self.heat(now));
                if good {
                    self.last_cycle = cycle;
                }
                good
            }
            Method::Cook => input == 0,
            Method::Preserve if self.progress < 3 => input == self.dish.seasoning()[self.progress],
            Method::Preserve => input == 0 && Self::SEAL_RANGE.contains(&self.heat(now)),
            Method::Mill => input == self.progress % 2,
        };
        if success {
            self.last_step = Some(now);
            self.progress += 1;
            self.message = "";
        } else {
            self.mistakes = self.mistakes.saturating_add(1);
            self.message = match self.dish.method() {
                Method::Cook => "Versuche es im grünen Bereich.",
                Method::Preserve if self.progress == 3 => "Warte auf den grünen Bereich.",
                Method::Preserve => "Schau auf die Reihenfolge und wähle die nächste Zutat.",
                Method::Mill => "Wechsle die Seite.",
            };
        }
        success
    }

    pub fn finish(&mut self, garden: &mut Garden) -> Option<u32> {
        let (progress, total) = self.progress();
        if progress != total || !self.dish.ready(garden) {
            return None;
        }
        let fine = self.dish.ingredients().iter().any(|ingredient| {
            matches!(*ingredient, Ingredient::Crop(crop, count)
                if garden.stock(Product::Crop(crop, crate::garden::Quality::Normal)) < count)
        });
        let practiced = crate::logbook::Path::Kitchen.rank(garden) >= 2;
        let servings = 1 + u32::from((fine || practiced) && self.mistakes == 0);
        for ingredient in self.dish.ingredients() {
            ingredient.consume(garden);
        }
        garden.add_stock(Product::Prepared(self.dish), servings);
        garden.logbook.meal(self.dish);
        if let Some(prologue) = &mut garden.prologue {
            prologue.cooked = true;
        }
        // A completed action cannot consume ingredients again if another input arrives.
        self.progress += 1;
        Some(servings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garden::Quality;

    #[test]
    fn cooking_uses_three_distinct_rounds_and_preserves_stock_on_mistakes() {
        let mut garden = Garden::default();
        garden.add_stock(Product::Crop(Crop::Carrot, Quality::Fine), 2);
        garden.add_stock(Product::Mushroom, 1);
        let mut game = Preparation::new(Dish::CarrotSoup, 0.);
        assert!(game.try_step(0, 0.10));
        assert_eq!(game.finish(&mut garden), None);
        assert!(!game.try_step(0, 0.37));
        for _ in 0..3 {
            game.try_step(0, 0.87);
        }
        assert_eq!(game.progress().0, 2);
        game.try_step(0, Preparation::PERIOD + 0.87);
        game.try_step(0, Preparation::PERIOD * 2. + 0.87);
        game.try_step(0, Preparation::PERIOD * 2. + 1.25);
        assert_eq!(game.finish(&mut garden), Some(1));
        assert_eq!(game.finish(&mut garden), None);
        assert_eq!(garden.stock(Product::Prepared(Dish::CarrotSoup)), 1);
        assert_eq!(garden.crop_total(Crop::Carrot), 0);
    }

    #[test]
    fn preserving_and_milling_have_different_inputs() {
        let mut preserve = Preparation::new(Dish::BerryJam, 0.);
        let mut mill = Preparation::new(Dish::HazelFlour, 0.);
        for (i, input) in [0, 1, 0, 1, 0, 1].into_iter().enumerate() {
            preserve.try_step(input, i as f32);
            mill.try_step(input, i as f32);
        }
        assert_eq!(preserve.progress(), (1, 4));
        assert_eq!(mill.progress(), (6, 6));
        preserve.try_step(2, 6.);
        preserve.try_step(1, 6.4);
        assert_eq!(preserve.progress(), (3, 4));
        preserve.try_step(0, 7.5);
        assert_eq!(preserve.progress(), (4, 4));
    }
}
