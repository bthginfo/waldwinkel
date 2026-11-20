use crate::{
    garden::{Crop, Garden},
    homestead::Product,
    regions::Material,
    waywatch::{Stage, Watch},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bench {
    Work,
    Forge,
    Loom,
}

impl Bench {
    pub fn name(self) -> &'static str {
        match self {
            Self::Work => "Werkbank",
            Self::Forge => "Schmiedetisch",
            Self::Loom => "Webrahmen",
        }
    }

    fn ready(self, g: &Garden) -> bool {
        match self {
            Self::Work => g.workshop.restored,
            Self::Forge => g.workshop.forge,
            Self::Loom => g.workshop.loom,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum Recipe {
    Boards,
    TideGlass,
    Rivets,
    Canvas,
    Oil,
}

impl Recipe {
    pub const ALL: [Self; 5] = [
        Self::Boards,
        Self::TideGlass,
        Self::Rivets,
        Self::Canvas,
        Self::Oil,
    ];

    pub fn name(self) -> &'static str {
        [
            "Fichtenbretter",
            "Muschelglas",
            "Kupfernieten",
            "Segeltuch",
            "Laternenöl",
        ][self as usize]
    }

    pub fn icon(self) -> &'static str {
        ["boards", "tideglass", "rivets", "canvas", "lampoil"][self as usize]
    }

    pub fn value(self) -> u32 {
        [4, 8, 6, 9, 10][self as usize]
    }

    pub fn sort(self) -> Bench {
        match self {
            Self::Boards | Self::TideGlass | Self::Oil => Bench::Work,
            Self::Rivets => Bench::Forge,
            Self::Canvas => Bench::Loom,
        }
    }

    pub(crate) fn source(self) -> Option<Watch> {
        match self {
            Self::Boards => None,
            Self::TideGlass => Some(Watch::Tide),
            Self::Rivets => Some(Watch::Kiln),
            Self::Canvas => Some(Watch::Wind),
            Self::Oil => Some(Watch::Fen),
        }
    }

    pub fn learned(self, g: &Garden) -> bool {
        self.source()
            .is_none_or(|site| site.stage(g) >= Stage::Surveyed)
    }

    pub(crate) fn ingredients(self) -> &'static [Need] {
        use crate::processing::Dish;
        use Need::*;
        match self {
            Self::Boards => &[Wood(3)],
            Self::TideGlass => &[Copper(1), Stock(Product::Shell, 2)],
            Self::Rivets => &[Copper(2), Stone(1)],
            Self::Canvas => &[
                Stock(Product::Regional(Material::MeadowFlax), 2),
                Stock(Product::Wool, 1),
            ],
            Self::Oil => &[
                Stock(Product::Prepared(Dish::SunflowerMeal), 1),
                Stock(Product::Regional(Material::MarshHerb), 1),
            ],
        }
    }

    pub fn yield_count(self) -> u32 {
        if matches!(self, Self::Boards | Self::Rivets | Self::Canvas) {
            2
        } else {
            1
        }
    }

    pub fn describe(self, g: &Garden, batches: u32) -> String {
        if let Some(site) = self.source().filter(|_| !self.learned(g)) {
            return format!("Bauplan beim Wegzeichen · {}", site.name());
        }
        format!(
            "Ergibt {} · {}",
            self.yield_count() * batches,
            detail(self.ingredients(), g, batches)
        )
    }

    pub fn can_make(self, g: &Garden, bench: Bench, batches: u32) -> bool {
        self.sort() == bench
            && bench.ready(g)
            && (1..=5).contains(&batches)
            && self.learned(g)
            && ready(self.ingredients(), g, batches)
            && g.stock(Product::Crafted(self))
                .checked_add(self.yield_count() * batches)
                .is_some()
    }

    pub fn make(self, g: &mut Garden, bench: Bench, batches: u32) -> Result<String, String> {
        if !self.can_make(g, bench, batches) {
            return Err(self.describe(g, batches.clamp(1, 5)));
        }
        take(self.ingredients(), g, batches);
        let amount = self.yield_count() * batches;
        g.add_stock(Product::Crafted(self), amount);
        Ok(format!("+{amount} {}", self.name()))
    }
}

#[derive(Clone, Copy)]
pub enum Need {
    Wood(u32),
    Stone(u32),
    Copper(u32),
    Coins(u32),
    Crop(Crop, u32),
    Stock(Product, u32),
}

impl Need {
    pub(crate) fn values(self, g: &Garden) -> (u32, u32, &'static str) {
        match self {
            Self::Wood(q) => (g.wood, q, "Holz"),
            Self::Stone(q) => (g.stone, q, "Stein"),
            Self::Copper(q) => (g.copper, q, "Kupfer"),
            Self::Coins(q) => (g.farm.coins, q, "Münzen"),
            Self::Crop(crop, q) => (g.crop_total(crop), q, crop.name()),
            Self::Stock(item, q) => (g.stock(item), q, item.name()),
        }
    }

    fn take(self, g: &mut Garden, batches: u32) {
        match self {
            Self::Wood(q) => g.wood -= q * batches,
            Self::Stone(q) => g.stone -= q * batches,
            Self::Copper(q) => g.copper -= q * batches,
            Self::Coins(q) => g.farm.coins -= q * batches,
            Self::Crop(crop, q) => {
                g.take_crop(crop, q * batches);
            }
            Self::Stock(item, q) => {
                g.take_stock(item, q * batches);
            }
        }
    }
}

pub fn detail(needs: &[Need], g: &Garden, batches: u32) -> String {
    needs
        .iter()
        .filter_map(|need| {
            let (have, amount, name) = need.values(g);
            let total = amount * batches;
            (total > 0).then(|| format!("{}/{total} {name}", have.min(total)))
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

pub fn ready(needs: &[Need], g: &Garden, batches: u32) -> bool {
    needs.iter().all(|need| {
        let (have, amount, _) = need.values(g);
        have >= amount * batches
    })
}

pub fn take(needs: &[Need], g: &mut Garden, batches: u32) {
    for &need in needs {
        need.take(g, batches);
    }
}
