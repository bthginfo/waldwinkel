use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    garden::{Crop, Garden},
    homestead::Product,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum District {
    Forest,
    Beach,
    Town,
}

impl District {
    pub const ALL: [Self; 3] = [Self::Forest, Self::Beach, Self::Town];

    pub fn name(self) -> &'static str {
        match self {
            Self::Forest => "Farnwald",
            Self::Beach => "Muschelbucht",
            Self::Town => "Eichenhafen",
        }
    }
    pub fn model(self) -> &'static str {
        match self {
            Self::Forest => "district_forest",
            Self::Beach => "district_beach",
            Self::Town => "district_town",
        }
    }
    pub fn origin(self) -> Vec2 {
        match self {
            Self::Forest => Vec2::new(-51., -8.),
            Self::Beach => Vec2::new(4., 47.),
            Self::Town => Vec2::new(49., 4.),
        }
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
            }
    }

    pub fn at(point: Vec2) -> Option<Self> {
        if point.x < -27. && point.y < 19. {
            Some(Self::Forest)
        } else if point.y > 33. && point.x < 29. {
            Some(Self::Beach)
        } else if point.x > 32. && point.y < 25. {
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
        }
    }
}

#[derive(Clone, Copy)]
pub enum FindLoot {
    Product(Product),
    Wood,
    Seeds(Crop),
    Blueprint(usize),
}

#[derive(Clone, Copy)]
pub struct Find {
    pub district: District,
    pub local: Vec2,
    pub loot: FindLoot,
    pub quantity: u32,
}

impl Find {
    pub fn position(self) -> Vec2 {
        self.district.origin() + self.local
    }
    pub fn name(self) -> &'static str {
        match self.loot {
            FindLoot::Product(Product::Mushroom) => "Pilze",
            FindLoot::Product(Product::Hazel) => "Haselnüsse",
            FindLoot::Product(Product::Shell) => "Muscheln",
            FindLoot::Seeds(_) => "Wildes Saatgut",
            FindLoot::Blueprint(_) => "Möbelbauplan",
            _ => "Treibholz",
        }
    }
    pub fn model(self) -> &'static str {
        match self.loot {
            FindLoot::Product(Product::Mushroom) => "find_mushroom",
            FindLoot::Product(Product::Hazel) => "find_hazel",
            FindLoot::Product(Product::Shell) => "find_shell",
            FindLoot::Seeds(_) => "find_seeds",
            FindLoot::Blueprint(_) => "find_blueprint",
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
        match self.loot {
            FindLoot::Blueprint(plan) => !garden.blueprints[plan],
            _ => garden.town.find_ready(index, garden.day),
        }
    }

    pub fn reward(self) -> String {
        match self.loot {
            FindLoot::Seeds(crop) => format!("+{} Samen · {}", self.quantity, crop.name()),
            FindLoot::Blueprint(plan) => {
                format!("Bauplan · {}", crate::furniture::CATALOG[plan].name)
            }
            _ => format!("+{} {}", self.quantity, self.name()),
        }
    }
}

pub const FINDS: [Find; 31] = [
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
        local: Vec2::new(-17.3, -20.1),
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
