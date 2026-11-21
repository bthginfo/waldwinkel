use bevy::prelude::Vec2;

use crate::{garden::Garden, homestead::Product, regions::Material};

#[derive(Clone, Copy)]
pub enum Blueprint {
    Shop { coins: u32, quests: usize },
    Find(&'static str),
    Reward(&'static str),
}

impl Blueprint {
    const fn sold(coins: u32, quests: usize) -> Self {
        Self::Shop { coins, quests }
    }
}

pub struct Furniture {
    pub name: &'static str,
    pub model: &'static str,
    pub half: Vec2,
    pub height: f32,
    materials: [u32; 4],
    regional: Option<(Material, u32)>,
    pub blueprint: Blueprint,
}

impl Furniture {
    const fn new(
        name: &'static str,
        model: &'static str,
        shape: [f32; 3],
        materials: [u32; 4],
        blueprint: Blueprint,
    ) -> Self {
        Self {
            name,
            model,
            half: Vec2::new(shape[0], shape[1]),
            height: shape[2],
            materials,
            regional: None,
            blueprint,
        }
    }

    const fn regional(mut self, material: Material, count: u32) -> Self {
        self.regional = Some((material, count));
        self
    }

    pub fn source(&self, garden: &Garden) -> String {
        match self.blueprint {
            Blueprint::Find(hint) | Blueprint::Reward(hint) => hint.into(),
            Blueprint::Shop { coins, quests } if garden.town.completed() < quests => {
                format!("Bauplan für {coins} Münzen · {quests} Stadtaufträge nötig")
            }
            Blueprint::Shop { coins, .. } => format!("Bauplan am Markt · {coins} Münzen"),
        }
    }

    pub fn supplies(&self, garden: &Garden) -> String {
        crate::craft::detail(&self.ingredients(), garden, 1)
    }

    pub(crate) fn ingredients(&self) -> [crate::craft::Need; 5] {
        use crate::craft::Need;
        let [wood, stone, copper, wool] = self.materials;
        [
            Need::Wood(wood),
            Need::Stone(stone),
            Need::Copper(copper),
            Need::Stock(Product::Wool, wool),
            match self.regional {
                Some((material, quantity)) => Need::Stock(Product::Regional(material), quantity),
                None => Need::Wood(0),
            },
        ]
    }

    pub fn can_build(&self, g: &Garden) -> bool {
        crate::craft::ready(&self.ingredients(), g, 1)
    }

    pub fn can_buy(&self, garden: &Garden) -> bool {
        matches!(self.blueprint, Blueprint::Shop { coins, quests }
            if garden.farm.coins >= coins && garden.town.completed() >= quests)
            && garden.clock.market_open()
    }
}

pub const FERN_POT: usize = 5;
pub const ARMCHAIR: usize = 6;
pub const HARBOR_CHEST: usize = 9;
pub const VINE_SHELF: usize = 10;
pub const CUSHION_BENCH: usize = 12;
pub const WALL_HANGING: usize = 14;
pub const SEASONS_SHELF: usize = 18;
pub const HARBOR_LANTERN: usize = 19;
pub const WATCH_LAMP: usize = 20;

pub const EXPANSION_FURNITURE: usize = 21;
pub const TRAIL_FURNITURE: usize = 30;
pub const CATALOG: [Furniture; 33] = [
    Furniture::new(
        "Runder Tisch",
        "furniture_table",
        [0.77, 0.77, 1.36],
        [0, 0, 0, 0],
        Blueprint::Reward("Bauplan beim Instandsetzen der Werkbank"),
    ),
    Furniture::new(
        "Korbregal",
        "furniture_shelf",
        [0.73, 0.47, 1.78],
        [3, 0, 0, 0],
        Blueprint::Reward("Bauplan beim Instandsetzen der Werkbank"),
    ),
    Furniture::new(
        "Kachelofen",
        "furniture_stove",
        [0.47, 0.48, 1.99],
        [1, 3, 1, 0],
        Blueprint::Reward("Bauplan beim Instandsetzen der Werkbank"),
    ),
    Furniture::new(
        "Blaue Sitzbank",
        "furniture_blue_bench",
        [0.71, 0.38, 0.93],
        [4, 1, 0, 0],
        Blueprint::sold(9, 0),
    ),
    Furniture::new(
        "Laternenständer",
        "furniture_lantern",
        [0.31, 0.32, 1.59],
        [2, 0, 1, 0],
        Blueprint::sold(12, 0),
    ),
    Furniture::new(
        "Farntopf",
        "furniture_fern",
        [0.36, 0.36, 1.05],
        [1, 2, 0, 0],
        Blueprint::Find("Bauplan im Farnwald"),
    ),
    Furniture::new(
        "Lesesessel",
        "furniture_armchair",
        [0.57, 0.56, 1.16],
        [4, 0, 0, 2],
        Blueprint::Reward("Belohnung von Mara"),
    ),
    Furniture::new(
        "Patchworksofa",
        "furniture_sofa",
        [0.82, 0.61, 1.12],
        [6, 0, 0, 3],
        Blueprint::sold(26, 1),
    ),
    Furniture::new(
        "Glasvitrine",
        "furniture_display",
        [0.67, 0.38, 1.76],
        [5, 2, 2, 0],
        Blueprint::sold(32, 1),
    ),
    Furniture::new(
        "Hafentruhe",
        "furniture_harbor_chest",
        [0.63, 0.43, 0.78],
        [5, 0, 1, 0],
        Blueprint::Find("Bauplan in der Muschelbucht"),
    ),
    Furniture::new(
        "Rankregal",
        "furniture_vines",
        [0.62, 0.37, 1.76],
        [5, 1, 1, 0],
        Blueprint::Reward("Belohnung von Fiete"),
    ),
    Furniture::new(
        "Schreibtisch",
        "furniture_desk",
        [0.74, 0.48, 1.1],
        [5, 1, 2, 0],
        Blueprint::sold(23, 1),
    ),
    Furniture::new(
        "Kissenbank",
        "furniture_cushion",
        [0.72, 0.43, 0.71],
        [4, 0, 0, 2],
        Blueprint::Find("Bauplan am Rand des Farnwalds"),
    ),
    Furniture::new(
        "Blauer Ofen",
        "furniture_blue_stove",
        [0.51, 0.49, 1.98],
        [2, 5, 3, 0],
        Blueprint::sold(39, 2),
    ),
    Furniture::new(
        "Wollteppich",
        "furniture_rug",
        [0., 0., 0.14],
        [0, 0, 0, 3],
        Blueprint::Reward("Belohnung von Bo"),
    ),
    Furniture::new(
        "Kupferlampentisch",
        "furniture_copper_lamp",
        [0.56, 0.47, 1.4],
        [4, 1, 5, 0],
        Blueprint::sold(45, 3),
    ),
    Furniture::new(
        "Kräutervitrine",
        "furniture_herbs",
        [0.72, 0.43, 1.67],
        [6, 2, 3, 0],
        Blueprint::sold(49, 3),
    ),
    Furniture::new(
        "Walnussharfe",
        "furniture_harp",
        [0.61, 0.43, 1.68],
        [8, 0, 4, 2],
        Blueprint::sold(58, 3),
    ),
    Furniture::new(
        "Jahreszeitenregal",
        "furniture_seasons",
        [0.58, 0.34, 1.49],
        [7, 0, 2, 0],
        Blueprint::Reward("Belohnung für alle vier Dorftreffen"),
    ),
    Furniture::new(
        "Hafenlaterne",
        "furniture_harbor_lantern",
        [0.35, 0.27, 1.48],
        [3, 1, 2, 0],
        Blueprint::Reward("Belohnung für die Wiedereröffnung des Fischerstegs"),
    ),
    Furniture::new(
        "Wegwachtlampe",
        "furniture_watch_lamp",
        [0.39, 0.35, 1.68],
        [4, 2, 2, 0],
        Blueprint::Reward("Belohnung für Junas Inselkarte"),
    ),
    Furniture::new("Harzlaterne", "furniture_harzlaterne", [0.37, 0.32, 1.48], [4, 1, 2, 0],
        Blueprint::Reward("Die Harzspur")).regional(Material::GoldResin, 3),
    Furniture::new("Flaumbank", "furniture_flaumbank", [0.76, 0.4, 0.94], [6, 0, 0, 1],
        Blueprint::Reward("Decken für den ersten Frost")).regional(Material::RabbitDown, 2),
    Furniture::new("Bernsteinvitrine", "furniture_bernsteinvitrine", [0.65, 0.4, 1.69], [5, 0, 2, 0],
        Blueprint::Reward("Das goldene Blätterbuch")).regional(Material::Amber, 2),
    Furniture::new("Salbeitrog", "furniture_salbeitrog", [0.62, 0.39, 0.93], [2, 5, 0, 0],
        Blueprint::Reward("Pfad am Seil")).regional(Material::AlpineSage, 3),
    Furniture::new("Almhüttenbank", "furniture_almbank", [0.79, 0.44, 1.03], [7, 0, 0, 2],
        Blueprint::Reward("Frühstück am Wolkenrand")).regional(Material::StarOre, 2),
    Furniture::new("Sternwarte-Lampe", "furniture_sternlampe", [0.43, 0.37, 1.45], [4, 2, 5, 0],
        Blueprint::Reward("Sternkarte im Giebel")).regional(Material::SkyQuartz, 3),
    Furniture::new("Gezeitenwindspiel", "furniture_gezeitenwindspiel", [0.39, 0.32, 1.61], [3, 0, 2, 0],
        Blueprint::Reward("Wenn die Flut leise wird")).regional(Material::SaltBloom, 3),
    Furniture::new("Perlmuttkasten", "furniture_perlmuttkasten", [0.65, 0.42, 0.9], [5, 1, 2, 0],
        Blueprint::Reward("Frühe Eier am Anleger")).regional(Material::MotherPearl, 3),
    Furniture::new("Strandtafeltisch", "furniture_strandtisch", [0.77, 0.56, 1.1], [7, 0, 2, 0],
        Blueprint::Reward("Lichter am Seil")).regional(Material::Reefwood, 4),
    Furniture::new("Harzsammlerregal", "furniture_harzregal", [0.59, 0.35, 1.56], [6, 0, 2, 0],
        Blueprint::Find("Am westlichen Bachlauf im Bernsteinhain")).regional(Material::Ironwood, 3),
    Furniture::new("Gipfelglobus", "furniture_gipfelglobus", [0.45, 0.38, 1.02], [3, 4, 2, 0],
        Blueprint::Find("An der oberen Aussicht der Himmelstufen")).regional(Material::StarOre, 3),
    Furniture::new("Entenboje", "furniture_entenboje", [0.46, 0.46, 0.95], [4, 0, 2, 0],
        Blueprint::Find("Am Außenriff von Morgenriff")).regional(Material::DuckFeather, 2),
];

pub fn learn(garden: &mut Garden, design: usize) -> bool {
    let already = std::mem::replace(&mut garden.blueprints[design], true);
    !already
}

pub fn buy(garden: &mut Garden, index: usize) -> Result<String, String> {
    let design = &CATALOG[index];
    let Blueprint::Shop { coins, .. } = design.blueprint else {
        return Err(design.source(garden));
    };
    if garden.blueprints[index] {
        return Err("Diesen Bauplan kennst du bereits.".into());
    }
    if !design.can_buy(garden) {
        return Err(design.source(garden));
    }
    garden.farm.coins -= coins;
    garden.farm.trades = garden.farm.trades.saturating_add(1);
    learn(garden, index);
    Ok(format!("Bauplan gelernt · {}", design.name))
}

pub fn build(garden: &mut Garden, index: usize) -> Result<String, String> {
    let design = &CATALOG[index];
    if garden.crafted[index] {
        return Err("Das Möbel ist fertig. Wähle einen Platz in der Hütte.".into());
    }
    if !garden.blueprints[index] {
        return Err(design.source(garden));
    }
    if !design.can_build(garden) {
        return Err(format!(
            "Dir fehlen Materialien · {}",
            design.supplies(garden)
        ));
    }
    crate::craft::take(&design.ingredients(), garden, 1);
    garden.crafted[index] = true;
    Ok(format!(
        "{} hergestellt. Wähle einen Platz in der Hütte.",
        design.name
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_purchased_blueprint_allows_one_craft_and_charges_materials() {
        let mut g = Garden::default();
        g.wood = 10;
        g.stone = 8;
        assert!(build(&mut g, 3).is_err());
        assert!(!g.crafted[3]);
        assert!(buy(&mut g, 3).is_ok());
        assert!(build(&mut g, 3).is_ok());
        assert!(g.crafted[3]);
        assert!(build(&mut g, 3).is_err());
        assert_eq!((g.wood, g.stone, g.farm.coins), (6, 7, 11));
    }
}
