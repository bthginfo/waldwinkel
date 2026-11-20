use crate::{
    garden::{Crop, Garden},
    homestead::Product,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Material {
    Wood,
    Stone,
    Copper,
    Fertilizer,
    Feed,
    Seed(Crop),
    Driftwood,
    Fittings,
}

impl Material {
    pub fn stocked(garden: &Garden) -> impl Iterator<Item = Self> + '_ {
        [
            Self::Wood,
            Self::Stone,
            Self::Copper,
            Self::Driftwood,
            Self::Fittings,
            Self::Fertilizer,
            Self::Feed,
        ]
        .into_iter()
        .chain(Crop::ALL.map(Self::Seed))
        .filter(|material| material.amount(garden) > 0)
    }

    pub fn amount(self, garden: &Garden) -> u32 {
        match self {
            Self::Wood => garden.wood,
            Self::Stone => garden.stone,
            Self::Copper => garden.copper,
            Self::Fertilizer => garden.fertilizer,
            Self::Feed => garden.farm.feed,
            Self::Seed(crop) => garden.seed_count(crop),
            Self::Driftwood => u32::from(
                garden.harbor.phase == crate::harbor::Phase::Damaged && garden.harbor.driftwood,
            ),
            Self::Fittings => u32::from(garden.harbor.fittings),
        }
    }

    pub fn name(self) -> String {
        match self {
            Self::Wood => "Holz",
            Self::Stone => "Stein",
            Self::Copper => "Kupfer",
            Self::Fertilizer => "Dünger",
            Self::Feed => "Tierfutter",
            Self::Driftwood => "Treibholzbündel",
            Self::Fittings => "Kupferbeschläge",
            Self::Seed(crop) => return format!("Saatgut · {}", crop.name()),
        }
        .into()
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Wood | Self::Driftwood => "wood",
            Self::Stone => "stone",
            Self::Copper => "copper",
            Self::Fertilizer => "fertilizer",
            Self::Feed => "feed",
            Self::Fittings => "fittings",
            Self::Seed(crop) => [
                "seed_carrot",
                "seed_turnip",
                "seed_pumpkin",
                "seed_strawberry",
                "seed_tomato",
                "seed_sunflower",
            ][crop as usize],
        }
    }
}

pub fn crop_icon(crop: Crop) -> &'static str {
    [
        "carrot",
        "turnip",
        "pumpkin",
        "strawberry",
        "tomato",
        "sunflower",
    ][crop as usize]
}

pub fn product_icon(product: Product) -> &'static str {
    match product {
        Product::Crop(crop, _) => crop_icon(crop),
        Product::Prepared(dish) => {
            ["soup", "stew", "jam", "compote", "flour", "meal"][dish as usize]
        }
        Product::Regional(material) => ["quartz", "flax", "mint"][material as usize],
        Product::Spoils(loot) => ["moss", "seal", "dust"][loot as usize],
        Product::Seasonal(season) => ["blossom", "sun_shell", "amber", "frost"][season as usize],
        Product::Fish => "fish",
        Product::Herring => "herring",
        Product::Apple => "apple",
        Product::Berry => "berry",
        Product::Egg => "egg",
        Product::Wool => "wool",
        Product::Silver => "silver",
        Product::Crystal => "crystal",
        Product::Mushroom => "mushroom",
        Product::Hazel => "hazel",
        Product::Shell => "shell",
    }
}
