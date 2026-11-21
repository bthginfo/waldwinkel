use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    duel::Card,
    garden::{Crop, Garden},
    homestead::Product,
    regions::Material,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum District {
    Forest,
    Beach,
    Town,
    Copper,
    Wind,
    Moor,
    Amber,
    Heights,
    Reef,
}

impl District {
    pub const ALL: [Self; 9] = [
        Self::Forest,
        Self::Beach,
        Self::Town,
        Self::Copper,
        Self::Wind,
        Self::Moor,
        Self::Amber,
        Self::Heights,
        Self::Reef,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Forest => "Farnwald",
            Self::Beach => "Muschelbucht",
            Self::Town => "Eichenhafen",
            Self::Copper => "Kupferklamm",
            Self::Wind => "Windweiden",
            Self::Moor => "Nebelmoor",
            Self::Amber => "Bernsteinhain",
            Self::Heights => "Himmelstufen",
            Self::Reef => "Morgenriff",
        }
    }
    pub fn model(self) -> Option<&'static str> {
        Some(match self {
            Self::Forest => "district_forest",
            Self::Beach => "district_beach",
            Self::Town => "district_town",
            Self::Copper => "district_copper",
            Self::Wind => "district_wind",
            Self::Moor => "district_moor",
            _ => return None,
        })
    }
    pub fn origin(self) -> Vec2 {
        match self {
            Self::Forest => Vec2::new(-51., -8.),
            Self::Beach => Vec2::new(4., 47.),
            Self::Town => Vec2::new(49., 4.),
            Self::Copper => Vec2::new(137., -5.),
            Self::Wind => Vec2::new(28., 130.),
            Self::Moor => Vec2::new(-137., -13.),
            Self::Amber => Vec2::new(-251., -16.),
            Self::Heights => Vec2::new(250., -28.),
            Self::Reef => Vec2::new(26., 239.),
        }
    }

    pub fn expansion(self) -> Option<crate::expansion::Region> {
        use crate::expansion::Region;
        match self {
            Self::Amber => Some(Region::Amber),
            Self::Heights => Some(Region::Heights),
            Self::Reef => Some(Region::Reef),
            _ => None,
        }
    }

    pub fn accessible(self, garden: &Garden) -> bool {
        self.expansion().is_none_or(|area| area.open(garden))
    }

    pub fn former_origin(self) -> Vec2 {
        Vec2::new(248. + self as u8 as f32 * 80., 0.)
    }

    pub fn waypoint(self) -> Vec2 {
        self.origin()
            + match self {
                Self::Forest => Vec2::new(9.60, 2.45),
                Self::Beach => Vec2::new(-2.30, 0.51),
                Self::Town => Vec2::new(0.0, 5.57),
                Self::Copper => Vec2::new(-12., 13.),
                Self::Wind => Vec2::new(-11., -14.),
                Self::Moor => Vec2::new(-7., 5.),
                Self::Amber => return Vec2::new(-246., -11.),
                Self::Heights => return Vec2::new(249., -20.),
                Self::Reef => return Vec2::new(28., 232.),
            }
    }

    pub fn at(point: Vec2) -> Option<Self> {
        if let Some(region) = crate::expansion::Region::ALL.into_iter().find(|area| area.boundary(point)) {
            Some(region.district())
        } else if point.x >= 78. && point.y < 70. {
            Some(Self::Copper)
        } else if point.y >= 76. {
            Some(Self::Wind)
        } else if point.x <= -87. {
            Some(Self::Moor)
        } else if point.x < -27. && point.y < 19. {
            Some(Self::Forest)
        } else if point.y > 33. && point.x < 29. {
            Some(Self::Beach)
        } else if point.x > 32.
            && (point.y < 25. || (54. ..=60.).contains(&point.x) && point.y <= 37.)
        {
            Some(Self::Town)
        } else {
            None
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::Forest => "Pilze und Haselnüsse wachsen an beiden Bachufern.",
            Self::Beach => "Suche im Sand nach Muscheln und Treibholz.",
            Self::Town => "Tippe eine Figur an, um zu ihr zu gehen.",
            Self::Copper => "Brücken führen zu Quarz, Werkhöfen und Runas Schmiede.",
            Self::Wind => "Folge den Wiesenwegen zu Nele, der Mühle und den Flachsfeldern.",
            Self::Moor => "Sammle Nebelminze an den trockenen Ufern und sprich mit Edda.",
            Self::Amber => "Am Harzpfad findest du Goldharz. Mit der Silberaxt kannst du Eisenholz schlagen.",
            Self::Heights => "Salbei wächst an den unteren Terrassen. Zu den Erzadern führt der Serpentinenweg.",
            Self::Reef => "Salzblüten blühen an den Tümpeln. Perlmutt findest du an den flachen Felsbänken.",
        }
    }
}

#[derive(Clone, Copy)]
pub enum FindLoot {
    Product(Product),
    Wood,
    Seeds(Crop),
    Blueprint(usize),
    Card(Card),
}

#[derive(Clone, Copy)]
pub struct Find {
    pub district: District,
    pub local: Vec2,
    pub loot: FindLoot,
    pub quantity: u32,
}

impl Find {
    const fn regional(district: District, local: Vec2, material: Material) -> Self {
        Self { district, local, loot: FindLoot::Product(Product::Regional(material)), quantity: 2 }
    }

    pub fn position(self) -> Vec2 {
        self.district.origin() + self.local
    }
    pub fn name(self) -> &'static str {
        match self.loot {
            FindLoot::Product(Product::Mushroom) => "Pilze",
            FindLoot::Product(Product::Hazel) => "Haselnüsse",
            FindLoot::Product(Product::Shell) => "Muscheln",
            FindLoot::Product(Product::Grass) => "Wildgras",
            FindLoot::Product(Product::Regional(material)) => material.name(),
            FindLoot::Seeds(_) => "Wildes Saatgut",
            FindLoot::Blueprint(_) => "Möbelbauplan",
            FindLoot::Card(_) => "Kartenfund",
            _ => "Treibholz",
        }
    }
    pub fn model(self) -> &'static str {
        match self.loot {
            FindLoot::Product(Product::Mushroom) => "find_mushroom",
            FindLoot::Product(Product::Hazel) => "find_hazel",
            FindLoot::Product(Product::Shell) => "find_shell",
            FindLoot::Product(Product::Grass) => "find_grass",
            FindLoot::Product(Product::Regional(material)) => material.model(),
            FindLoot::Seeds(_) => "find_seeds",
            FindLoot::Blueprint(_) | FindLoot::Card(_) => "find_blueprint",
            _ => "find_driftwood",
        }
    }
    pub fn height(self) -> f32 {
        if matches!(self.loot, FindLoot::Seeds(_)) {
            0.39
        } else {
            0.25
        }
    }

    pub fn available(self, garden: &Garden, index: usize) -> bool {
        if !self.district.accessible(garden) { return false; }
        match self.loot {
            FindLoot::Blueprint(plan) => !garden.blueprints[plan],
            FindLoot::Card(card) => !garden.rpg.knows(card),
            _ => garden.town.find_ready(index, garden.day),
        }
    }

    pub fn reward(self) -> String {
        match self.loot {
            FindLoot::Seeds(crop) => format!("+{} Samen · {}", self.quantity, crop.name()),
            FindLoot::Blueprint(plan) => {
                format!("Bauplan · {}", crate::furniture::CATALOG[plan].name)
            }
            FindLoot::Card(card) => format!("Karte · {}", card.name()),
            _ => format!("+{} {}", self.quantity, self.name()),
        }
    }
}

pub const FINDS: [Find; 90] = [
    Find {
        district: District::Forest,
        local: Vec2::new(-6.8, 2.4),
        loot: FindLoot::Product(Product::Mushroom),
        quantity: 2,
    },
    Find {
        district: District::Forest,
        local: Vec2::new(-4.50, -2.70),
        loot: FindLoot::Product(Product::Mushroom),
        quantity: 2,
    },
    Find {
        district: District::Forest,
        local: Vec2::new(3.62, -2.82),
        loot: FindLoot::Product(Product::Mushroom),
        quantity: 2,
    },
    Find {
        district: District::Forest,
        local: Vec2::new(7.65, 2.8),
        loot: FindLoot::Product(Product::Mushroom),
        quantity: 2,
    },
    Find {
        district: District::Forest,
        local: Vec2::new(-5.65, 4.75),
        loot: FindLoot::Product(Product::Hazel),
        quantity: 2,
    },
    Find {
        district: District::Forest,
        local: Vec2::new(5.1, -3.9),
        loot: FindLoot::Product(Product::Hazel),
        quantity: 2,
    },
    Find {
        district: District::Forest,
        local: Vec2::new(6.8, 1.1),
        loot: FindLoot::Product(Product::Hazel),
        quantity: 2,
    },
    Find {
        district: District::Beach,
        local: Vec2::new(2.75, 4.15),
        loot: FindLoot::Product(Product::Shell),
        quantity: 2,
    },
    Find {
        district: District::Beach,
        local: Vec2::new(4.25, 1.86),
        loot: FindLoot::Product(Product::Shell),
        quantity: 2,
    },
    Find {
        district: District::Beach,
        local: Vec2::new(4.6, -1.7),
        loot: FindLoot::Product(Product::Shell),
        quantity: 2,
    },
    Find {
        district: District::Beach,
        local: Vec2::new(-1.1, 4.8),
        loot: FindLoot::Product(Product::Shell),
        quantity: 2,
    },
    Find {
        district: District::Beach,
        local: Vec2::new(-6.2, 2.8),
        loot: FindLoot::Wood,
        quantity: 2,
    },
    Find {
        district: District::Beach,
        local: Vec2::new(-3.0, 4.55),
        loot: FindLoot::Wood,
        quantity: 2,
    },
    Find {
        district: District::Forest,
        local: Vec2::new(-16.0, -11.5),
        loot: FindLoot::Product(Product::Mushroom),
        quantity: 2,
    },
    Find {
        district: District::Forest,
        local: Vec2::new(-13.3, 11.1),
        loot: FindLoot::Product(Product::Hazel),
        quantity: 2,
    },
    Find {
        district: District::Forest,
        local: Vec2::new(12.45, -11.6),
        loot: FindLoot::Product(Product::Mushroom),
        quantity: 2,
    },
    Find {
        district: District::Forest,
        local: Vec2::new(-17.3, -18.95),
        loot: FindLoot::Product(Product::Hazel),
        quantity: 2,
    },
    Find {
        district: District::Forest,
        local: Vec2::new(15.2, 11.96),
        loot: FindLoot::Product(Product::Mushroom),
        quantity: 2,
    },
    Find {
        district: District::Forest,
        local: Vec2::new(-16.2, 11.15),
        loot: FindLoot::Product(Product::Mushroom),
        quantity: 2,
    },
    Find {
        district: District::Beach,
        local: Vec2::new(-14.1, 11.8),
        loot: FindLoot::Product(Product::Shell),
        quantity: 2,
    },
    Find {
        district: District::Beach,
        local: Vec2::new(-18.0, 5.4),
        loot: FindLoot::Wood,
        quantity: 2,
    },
    Find {
        district: District::Beach,
        local: Vec2::new(-7.6, 15.2),
        loot: FindLoot::Product(Product::Shell),
        quantity: 2,
    },
    Find {
        district: District::Beach,
        local: Vec2::new(0.2, 16.1),
        loot: FindLoot::Product(Product::Shell),
        quantity: 2,
    },
    Find {
        district: District::Beach,
        local: Vec2::new(-17.8, -1.7),
        loot: FindLoot::Wood,
        quantity: 2,
    },
    Find {
        district: District::Beach,
        local: Vec2::new(-9.3, 9.2),
        loot: FindLoot::Product(Product::Shell),
        quantity: 2,
    },
    Find {
        district: District::Forest,
        local: Vec2::new(-6.2, 3.7),
        loot: FindLoot::Seeds(Crop::Carrot),
        quantity: 2,
    },
    Find {
        district: District::Forest,
        local: Vec2::new(7.1, -1.45),
        loot: FindLoot::Seeds(Crop::Tomato),
        quantity: 2,
    },
    Find {
        district: District::Beach,
        local: Vec2::new(4., 2.75),
        loot: FindLoot::Seeds(Crop::Sunflower),
        quantity: 2,
    },
    Find {
        district: District::Forest,
        local: Vec2::new(-6.55, -1.2),
        loot: FindLoot::Blueprint(crate::furniture::FERN_POT),
        quantity: 1,
    },
    Find {
        district: District::Beach,
        local: Vec2::new(2.52, -3.18),
        loot: FindLoot::Blueprint(crate::furniture::HARBOR_CHEST),
        quantity: 1,
    },
    Find {
        district: District::Forest,
        local: Vec2::new(-14.8, 13.18),
        loot: FindLoot::Blueprint(crate::furniture::CUSHION_BENCH),
        quantity: 1,
    },
    Find {
        district: District::Copper,
        local: Vec2::new(-8.0, 9.0),
        loot: FindLoot::Product(Product::Regional(Material::GorgeQuartz)),
        quantity: 2,
    },
    Find {
        district: District::Copper,
        local: Vec2::new(-21.0, 14.0),
        loot: FindLoot::Product(Product::Regional(Material::GorgeQuartz)),
        quantity: 2,
    },
    Find {
        district: District::Copper,
        local: Vec2::new(-7.0, -14.0),
        loot: FindLoot::Product(Product::Regional(Material::GorgeQuartz)),
        quantity: 2,
    },
    Find {
        district: District::Copper,
        local: Vec2::new(11.0, 7.0),
        loot: FindLoot::Product(Product::Regional(Material::GorgeQuartz)),
        quantity: 2,
    },
    Find {
        district: District::Copper,
        local: Vec2::new(19.0, -4.0),
        loot: FindLoot::Product(Product::Regional(Material::GorgeQuartz)),
        quantity: 2,
    },
    Find {
        district: District::Copper,
        local: Vec2::new(21.0, 21.0),
        loot: FindLoot::Product(Product::Regional(Material::GorgeQuartz)),
        quantity: 2,
    },
    Find {
        district: District::Wind,
        local: Vec2::new(-15.0, -12.0),
        loot: FindLoot::Product(Product::Regional(Material::MeadowFlax)),
        quantity: 2,
    },
    Find {
        district: District::Wind,
        local: Vec2::new(-23.0, -15.0),
        loot: FindLoot::Product(Product::Regional(Material::MeadowFlax)),
        quantity: 2,
    },
    Find {
        district: District::Wind,
        local: Vec2::new(-3.0, -9.0),
        loot: FindLoot::Product(Product::Regional(Material::MeadowFlax)),
        quantity: 2,
    },
    Find {
        district: District::Wind,
        local: Vec2::new(8.0, 4.0),
        loot: FindLoot::Product(Product::Regional(Material::MeadowFlax)),
        quantity: 2,
    },
    Find {
        district: District::Wind,
        local: Vec2::new(17.0, 18.0),
        loot: FindLoot::Product(Product::Regional(Material::MeadowFlax)),
        quantity: 2,
    },
    Find {
        district: District::Wind,
        local: Vec2::new(-12.0, 17.0),
        loot: FindLoot::Product(Product::Regional(Material::MeadowFlax)),
        quantity: 2,
    },
    Find {
        district: District::Moor,
        local: Vec2::new(-6.0, 4.0),
        loot: FindLoot::Product(Product::Regional(Material::MarshHerb)),
        quantity: 2,
    },
    Find {
        district: District::Moor,
        local: Vec2::new(-8.0, -3.0),
        loot: FindLoot::Product(Product::Regional(Material::MarshHerb)),
        quantity: 2,
    },
    Find {
        district: District::Moor,
        local: Vec2::new(-22.0, 8.0),
        loot: FindLoot::Product(Product::Regional(Material::MarshHerb)),
        quantity: 2,
    },
    Find {
        district: District::Moor,
        local: Vec2::new(0.5, -16.0),
        loot: FindLoot::Product(Product::Regional(Material::MarshHerb)),
        quantity: 2,
    },
    Find {
        district: District::Moor,
        local: Vec2::new(14.0, -8.0),
        loot: FindLoot::Product(Product::Regional(Material::MarshHerb)),
        quantity: 2,
    },
    Find {
        district: District::Moor,
        local: Vec2::new(22.0, 9.0),
        loot: FindLoot::Product(Product::Regional(Material::MarshHerb)),
        quantity: 2,
    },
    Find {
        district: District::Forest,
        local: Vec2::new(8.73, 3.11),
        loot: FindLoot::Card(Card::ForageTea),
        quantity: 1,
    },
    Find {
        district: District::Copper,
        local: Vec2::new(-8.87, 10.52),
        loot: FindLoot::Card(Card::QuartzThrust),
        quantity: 1,
    },
    Find { district: District::Forest, local: Vec2::new(46.9, 15.7), loot: FindLoot::Product(Product::Grass), quantity: 2 },
    Find { district: District::Forest, local: Vec2::new(53.4, 17.9), loot: FindLoot::Product(Product::Grass), quantity: 2 },
    Find { district: District::Forest, local: Vec2::new(56.4, 18.9), loot: FindLoot::Product(Product::Grass), quantity: 2 },
    Find { district: District::Forest, local: Vec2::new(8.6, 5.0), loot: FindLoot::Product(Product::Grass), quantity: 2 },
    Find { district: District::Beach, local: Vec2::new(-6.1, -3.0), loot: FindLoot::Product(Product::Grass), quantity: 2 },
    Find { district: District::Wind, local: Vec2::new(-12.2, -12.5), loot: FindLoot::Product(Product::Grass), quantity: 3 },
    Find::regional(District::Amber, Vec2::new(8., 2.), Material::GoldResin),
    Find::regional(District::Amber, Vec2::new(4., 5.), Material::GoldResin),
    Find::regional(District::Amber, Vec2::new(-3., 7.), Material::GoldResin),
    Find::regional(District::Amber, Vec2::new(-8., 1.), Material::GoldResin),
    Find::regional(District::Amber, Vec2::new(-10., 15.), Material::GoldResin),
    Find::regional(District::Amber, Vec2::new(-23., 12.), Material::GoldResin),
    Find::regional(District::Amber, Vec2::new(-24., -7.), Material::GoldResin),
    Find::regional(District::Amber, Vec2::new(7., -11.), Material::GoldResin),
    Find::regional(District::Amber, Vec2::new(14., 14.), Material::GoldResin),
    Find::regional(District::Amber, Vec2::new(-7., -15.), Material::GoldResin),
    Find::regional(District::Heights, Vec2::new(-14., 14.), Material::AlpineSage),
    Find::regional(District::Heights, Vec2::new(-9., 7.), Material::AlpineSage),
    Find::regional(District::Heights, Vec2::new(-5., 14.), Material::AlpineSage),
    Find::regional(District::Heights, Vec2::new(-12., 0.), Material::AlpineSage),
    Find::regional(District::Heights, Vec2::new(6., -10.), Material::AlpineSage),
    Find::regional(District::Heights, Vec2::new(13., -18.), Material::AlpineSage),
    Find::regional(District::Heights, Vec2::new(8., 11.), Material::AlpineSage),
    Find::regional(District::Heights, Vec2::new(21., -27.), Material::AlpineSage),
    Find::regional(District::Heights, Vec2::new(17., -6.), Material::AlpineSage),
    Find::regional(District::Heights, Vec2::new(4., -24.), Material::AlpineSage),
    Find::regional(District::Reef, Vec2::new(8., -10.), Material::SaltBloom),
    Find::regional(District::Reef, Vec2::new(-5., -12.), Material::SaltBloom),
    Find::regional(District::Reef, Vec2::new(-12., -3.), Material::SaltBloom),
    Find::regional(District::Reef, Vec2::new(-11., 12.), Material::SaltBloom),
    Find::regional(District::Reef, Vec2::new(-22., 9.), Material::SaltBloom),
    Find::regional(District::Reef, Vec2::new(-24., -8.), Material::SaltBloom),
    Find::regional(District::Reef, Vec2::new(13., 0.), Material::SaltBloom),
    Find::regional(District::Reef, Vec2::new(14., 16.), Material::SaltBloom),
    Find::regional(District::Reef, Vec2::new(8., 18.), Material::SaltBloom),
    Find::regional(District::Reef, Vec2::new(-25., 14.), Material::SaltBloom),
    Find { district: District::Amber, local: Vec2::new(-35., 8.), loot: FindLoot::Blueprint(crate::furniture::TRAIL_FURNITURE), quantity: 1 },
    Find { district: District::Heights, local: Vec2::new(19., -36.), loot: FindLoot::Blueprint(crate::furniture::TRAIL_FURNITURE + 1), quantity: 1 },
    Find { district: District::Reef, local: Vec2::new(22., 15.), loot: FindLoot::Blueprint(crate::furniture::TRAIL_FURNITURE + 2), quantity: 1 },
];

pub fn collect(g: &mut Garden, index: usize) -> Option<String> {
    let find = *FINDS.get(index)?;
    if !find.available(g, index) {
        return None;
    }
    g.town.record_pick(index, g.day);
    match find.loot {
        FindLoot::Product(item) => g.add_stock(item, find.quantity),
        FindLoot::Seeds(crop) => g.add_seeds(crop, find.quantity),
        FindLoot::Blueprint(plan) => {
            crate::furniture::learn(g, plan);
        }
        FindLoot::Card(card) => {
            g.rpg.learn(card);
        }
        FindLoot::Wood => g.wood = g.wood.saturating_add(find.quantity),
    }
    Some(find.reward())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::walkable;

    #[test]
    fn waypoints_and_gathering_sites_are_walkable() {
        for district in District::ALL {
            assert!(walkable(district.waypoint()), "{district:?}");
        }
        for find in FINDS {
            assert!(
                walkable(find.position()),
                "{} at {}",
                find.name(),
                find.position()
            );
        }
    }

    #[test]
    fn bridges_cross_the_stream_and_the_beach_stops_at_water() {
        let forest = District::Forest.origin();
        assert!(!walkable(forest + Vec2::new(-1.15, 0.0)));
        assert!(walkable(forest + Vec2::new(-0.79, 0.81)));
        assert!(!walkable(Vec2::new(22.5, 53.8)));
    }
}
