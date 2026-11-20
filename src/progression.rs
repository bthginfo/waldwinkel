use crate::{
    duel::{Card, Gear},
    garden::Garden,
    homestead::Product,
    regions::{Frontier, Material},
    town::Require,
};

pub const CARDS_FOR_SALE: [Card; 3] = [Card::FlaxWrap, Card::RainStep, Card::StoneEcho];

pub fn card_price(card: Card) -> Option<u32> {
    match card {
        Card::FlaxWrap => Some(24),
        Card::RainStep => Some(18),
        Card::StoneEcho => Some(50),
        _ => None,
    }
}

pub fn buy_card(garden: &mut Garden, card: Card) -> Result<String, String> {
    let price = card_price(card).ok_or("Diese Karte ist am Stand nicht erhältlich.")?;
    if garden.rpg.knows(card) {
        return Err("Die Karte ist bereits in deiner Sammlung.".into());
    }
    if garden.farm.coins < price {
        return Err(format!("Für diese Karte brauchst du {price} Münzen."));
    }
    garden.farm.coins -= price;
    garden.rpg.learn(card);
    Ok(format!(
        "„{}“ liegt jetzt in deinem Kartenbinder.",
        card.name()
    ))
}

pub fn quest_card(area: Frontier, step: u8) -> Option<Card> {
    match (area, step) {
        (Frontier::Copper, 2) => Some(Card::CopperArc),
        (Frontier::Copper, 3) => Some(Card::HearthMeal),
        _ => None,
    }
}

pub fn restore_quest_cards(garden: &mut Garden) {
    for area in Frontier::ALL {
        for step in 1..=garden.regions.story_steps(area).min(3) {
            if let Some(card) = quest_card(area, step) {
                garden.rpg.learn(card);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GearSource {
    Workbench,
    Smith,
}

impl GearSource {
    fn price(self, gear: Gear) -> u32 {
        match self {
            Self::Workbench => [0, 12, 26, 46][gear as usize],
            Self::Smith => [0, 48, 105, 185][gear as usize],
        }
    }

    fn needs(self, gear: Gear) -> &'static [Require] {
        use Require::{Copper, Product as Stock, Wood};
        match (self, gear) {
            (Self::Smith, _) | (_, Gear::TrailStick) => &[],
            (_, Gear::CopperSickle) => &[Copper(2), Wood(2)],
            (_, Gear::SilverGuard) => &[Stock(Product::Silver, 2), Copper(3)],
            (_, Gear::CrystalBlade) => &[
                Stock(Product::Crystal, 2),
                Stock(Product::Silver, 2),
                Stock(Product::Regional(Material::GorgeQuartz), 2),
            ],
        }
    }

    fn minimum_level(self, gear: Gear) -> u32 {
        [0, 0, 5, 10][gear as usize]
    }

    pub fn detail(self, garden: &Garden, gear: Gear) -> String {
        let stats = format!(
            "+{} Schaden · +{} Widerstand",
            gear.damage(),
            gear.resistance()
        );
        if garden.rpg.gear_owned[gear as usize] {
            return if garden.rpg.gear == gear {
                format!("Ausgerüstet · {stats}")
            } else {
                format!("Im Besitz · {stats}")
            };
        }
        let level = self.minimum_level(gear);
        if garden.rpg.cleared < level {
            return format!("Ab Dämmergang {level} · {stats}");
        }
        let mut cost = format!("{} Münzen", self.price(gear));
        for requirement in self.needs(gear) {
            cost.push_str(" · ");
            cost.push_str(&requirement.progress(garden));
        }
        format!("{stats}\n{cost}")
    }

    pub fn ready(self, garden: &Garden, gear: Gear) -> bool {
        garden.rpg.gear != gear
            && (garden.rpg.gear_owned[gear as usize]
                || garden.rpg.cleared >= self.minimum_level(gear)
                    && garden.farm.coins >= self.price(gear)
                    && self
                        .needs(gear)
                        .iter()
                        .all(|item| item.available(garden) >= item.quantity()))
    }

    pub fn equip(self, garden: &mut Garden, gear: Gear) -> Result<String, String> {
        if !self.ready(garden, gear) {
            return Err(self.detail(garden, gear));
        }
        if !garden.rpg.gear_owned[gear as usize] {
            for material in self.needs(gear) {
                material.take(garden);
            }
            garden.farm.coins -= self.price(gear);
            garden.rpg.gear_owned[gear as usize] = true;
        }
        garden.rpg.gear = gear;
        Ok(format!("Ausgerüstet · {}", gear.name()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cards_and_gear_do_not_charge_twice_and_upgrades_require_progress() {
        let mut garden = Garden::default();
        garden.farm.coins = 200;
        garden.copper = 5;
        garden.wood = 3;
        assert!(buy_card(&mut garden, Card::FlaxWrap).is_ok());
        assert!(buy_card(&mut garden, Card::FlaxWrap).is_err());
        assert_eq!(garden.farm.coins, 176);
        assert!(
            GearSource::Workbench
                .equip(&mut garden, Gear::CopperSickle)
                .is_ok()
        );
        assert_eq!((garden.farm.coins, garden.copper, garden.wood), (164, 3, 1));
        assert!(
            GearSource::Smith
                .equip(&mut garden, Gear::SilverGuard)
                .is_err()
        );
        garden.rpg.cleared = 5;
        assert!(
            GearSource::Smith
                .equip(&mut garden, Gear::SilverGuard)
                .is_ok()
        );
        assert!(
            GearSource::Workbench
                .equip(&mut garden, Gear::CopperSickle)
                .is_ok()
        );
        assert_eq!((garden.farm.coins, garden.copper), (59, 3));
    }
}
