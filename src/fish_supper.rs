use serde::{Deserialize, Serialize};

use crate::{
    evening_market,
    garden::{Crop, Garden, Quality},
    homestead::Product,
    processing::Dish,
};

pub const TITLE: &str = "Eine Schale für den Hafen";
pub const PRICE: u32 = 18;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Stage {
    #[default]
    Unseen,
    NextMorning,
    Visiting,
    Cooking,
    Served,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Supper {
    pub stage: Stage,
    first_evening: u32,
    sold_on: Option<u32>,
}

impl Supper {
    pub fn record_catch(&mut self) -> bool {
        if self.stage != Stage::Unseen {
            return false;
        }
        self.stage = Stage::NextMorning;
        true
    }

    pub fn wake(&mut self) {
        if self.stage == Stage::NextMorning {
            self.stage = Stage::Visiting;
        }
    }

    pub fn at_farm(&self) -> bool {
        matches!(self.stage, Stage::Visiting | Stage::Cooking)
    }

    pub fn learned(&self) -> bool {
        matches!(self.stage, Stage::Cooking | Stage::Served)
    }

    pub fn listed(&self) -> bool {
        self.stage == Stage::Served
    }

    pub fn finish_first_meal(&mut self, day: u32, minute: u16) {
        if self.stage == Stage::Cooking {
            self.first_evening = day.saturating_add(u32::from(minute >= evening_market::OPENS));
            self.stage = Stage::Served;
        }
    }

    pub fn offered_today(&self, day: u32) -> bool {
        self.listed() && day >= self.first_evening
    }

    pub fn waiting_at_farm(&self, day: u32, minute: u16) -> bool {
        self.listed()
            && (day < self.first_evening
                || day == self.first_evening && minute < evening_market::OPENS)
    }

    pub fn detail(&self) -> &'static str {
        match self.stage {
            Stage::Unseen => "Wirf am Angelsteg deine Leine aus.",
            Stage::NextMorning => {
                "Schlaf in deiner Hütte. Bo kommt am nächsten Morgen zur Hofküche."
            }
            Stage::Visiting => {
                "Bo wartet bei deiner Hofküche. Sprich ihn auf deinen ersten Fang an."
            }
            Stage::Cooking => {
                "Koche mit Bo an der Hofküche. Du brauchst einen Fisch oder Hafenhering und eine Möhre."
            }
            Stage::Served => {
                "Erfüllt · Bos Hafenbrühe gibt es täglich von 18:00 bis 21:30 am Brunnen in Eichenhafen."
            }
        }
    }
}

pub fn learn(garden: &mut Garden) -> Result<String, String> {
    if garden.fish_supper.stage != Stage::Visiting {
        return Err(garden.fish_supper.detail().into());
    }
    garden.fish_supper.stage = Stage::Cooking;
    garden.add_stock(Product::Crop(Crop::Carrot, Quality::Normal), 1);
    Ok("Rezept gelernt · Bos Hafenbrühe · Bo legt eine Möhre dazu.".into())
}

pub fn stocked(garden: &Garden) -> bool {
    garden.fish_supper.offered_today(garden.day) && garden.fish_supper.sold_on != Some(garden.day)
}

pub fn can_buy(garden: &Garden) -> bool {
    stocked(garden)
        && evening_market::hours(garden)
        && garden.farm.coins >= PRICE
        && garden.stock(Product::Prepared(Dish::HarborBroth)) < u32::MAX
}

pub fn buy(garden: &mut Garden) -> Result<String, String> {
    if !can_buy(garden) {
        return Err("Prüfe Öffnungszeit, Tagesvorrat und Münzen.".into());
    }
    garden.farm.coins -= PRICE;
    garden.fish_supper.sold_on = Some(garden.day);
    garden.add_stock(Product::Prepared(Dish::HarborBroth), 1);
    Ok("Eine Schale Hafenbrühe eingepackt.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        day::FarmClock,
        duel::{Battle, Ration},
        harbor::{self, Fishing},
        processing::Preparation,
    };

    fn reload(garden: &Garden) -> Garden {
        Garden::decode(&serde_json::to_vec(garden).unwrap()).unwrap()
    }

    #[test]
    fn a_catch_then_sleep_then_cook_unlocks_the_evening_meal_across_saves() {
        let broth = Product::Prepared(Dish::HarborBroth);
        let mut garden = Garden::default();
        assert!(!Dish::HarborBroth.ready(&garden));
        assert!(learn(&mut garden).is_err());
        assert!(harbor::catch(&mut garden, Fishing::Landing).contains("besucht"));
        garden.take_stock(Product::Fish, 1);
        garden = reload(&garden);
        assert_eq!(garden.fish_supper.stage, Stage::NextMorning);
        garden.sleep();
        assert_eq!(garden.fish_supper.stage, Stage::Visiting);
        learn(&mut garden).unwrap();
        assert!(learn(&mut garden).is_err());
        assert_eq!(garden.crop_total(Crop::Carrot), 1);
        garden = reload(&garden);
        assert!(!Dish::HarborBroth.ready(&garden));
        assert!(!harbor::catch(&mut garden, Fishing::Seaward).contains("besucht"));
        assert!(Dish::HarborBroth.ready(&garden));
        let mut cooking = Preparation::new(Dish::HarborBroth);
        assert!(!cooking.try_step(1, 0.1));
        assert_eq!(cooking.finish(&mut garden), None);
        assert_eq!(garden.stock(Product::Herring), 1);
        assert!(!evening_market::stall_present(&garden));
        let at = cooking.cook_to_plate_for_test();
        assert!(cooking.try_step(0, at));
        assert_eq!(cooking.finish(&mut garden), Some(1));
        assert_eq!(cooking.finish(&mut garden), None);
        assert_eq!(garden.stock(Product::Herring), 0);
        assert_eq!(garden.crop_total(Crop::Carrot), 0);
        assert_eq!(garden.stock(broth), 1);
        garden = reload(&garden);
        assert!(evening_market::stall_present(&garden));
        assert!(!evening_market::serving(&garden));
        assert!(
            garden
                .fish_supper
                .waiting_at_farm(garden.day, garden.clock.minute())
        );
        garden.clock = FarmClock::from_minutes(evening_market::OPENS.into());
        assert!(evening_market::serving(&garden));
        assert!(!evening_market::trading(&garden));
        garden.farm.coins = PRICE - 1;
        assert!(evening_market::buy(&mut garden, Ration::Broth).is_err());
        garden.farm.coins = 2 * PRICE;
        assert!(evening_market::buy(&mut garden, Ration::Soup).is_err());
        evening_market::buy(&mut garden, Ration::Broth).unwrap();
        garden = reload(&garden);
        assert_eq!(garden.stock(broth), 2);
        assert_eq!(garden.farm.coins, PRICE);
        assert!(evening_market::buy(&mut garden, Ration::Broth).is_err());
        garden.sleep();
        garden.clock = FarmClock::from_minutes(evening_market::CLOSES.into());
        assert!(!evening_market::can_buy(&garden, Ration::Broth));
        garden.clock = FarmClock::from_minutes(evening_market::OPENS.into());
        assert!(evening_market::can_buy(&garden, Ration::Broth));
        assert!(
            !garden
                .fish_supper
                .waiting_at_farm(garden.day, garden.clock.minute())
        );
        let mut fight = Battle::new(1, &garden.rpg);
        fight.hp = fight.max_hp - 10;
        let guard = fight.guard;
        assert!(fight.eat(Ration::Broth));
        assert_eq!(fight.hp, fight.max_hp - 4);
        assert_eq!(fight.guard, guard + 4);
        assert!(!fight.eat(Ration::Broth));
    }

    #[test]
    fn old_pantry_and_catches_survive_the_update() {
        let mut old = serde_json::to_value(Garden::default()).unwrap();
        old["version"] = 16.into();
        old.as_object_mut().unwrap().remove("fish_supper");
        old["prepared"] = serde_json::json!([3, 1, 0, 2, 8, 4]);
        for (fish, expected) in [(0, Stage::Unseen), (1, Stage::NextMorning)] {
            old["fish"] = fish.into();
            let garden = Garden::decode(&serde_json::to_vec(&old).unwrap()).unwrap();
            assert_eq!(garden.fish_supper.stage, expected);
            assert_eq!(
                Dish::ALL.map(|dish| garden.stock(Product::Prepared(dish))),
                [3, 1, 0, 2, 8, 4, 0, 0, 0]
            );
        }
    }

    #[test]
    fn a_late_first_meal_makes_its_debut_next_evening_only_once() {
        let mut garden = Garden::default();
        garden.fish_supper.stage = Stage::Cooking;
        garden.clock = FarmClock::from_minutes(evening_market::OPENS.into());
        garden
            .fish_supper
            .finish_first_meal(garden.day, garden.clock.minute());
        assert!(!evening_market::serving(&garden));
        assert!(
            garden
                .fish_supper
                .waiting_at_farm(garden.day, garden.clock.minute())
        );
        garden.sleep();
        assert!(
            garden
                .fish_supper
                .waiting_at_farm(garden.day, garden.clock.minute())
        );
        garden.clock = FarmClock::from_minutes(evening_market::OPENS.into());
        garden
            .fish_supper
            .finish_first_meal(garden.day, garden.clock.minute());
        assert!(evening_market::serving(&garden));
        assert!(
            !garden
                .fish_supper
                .waiting_at_farm(garden.day, garden.clock.minute())
        );
    }
}
