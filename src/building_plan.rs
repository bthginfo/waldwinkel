use serde::{Deserialize, Serialize};

use crate::{
    adventure::Target,
    craft::{self, Bench, Need, Recipe},
    furniture::{Blueprint, CATALOG},
    garden::Garden,
    homestead::{Product, Village},
    regions::{Frontier, Material},
    town::Npc,
    travel::District,
    workshop::{Project, ToolKind},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Plan {
    Furniture(usize),
    Project(Project),
    Tool {
        tool: ToolKind,
        tier: u8,
    },
    Material {
        recipe: Recipe,
        batches: u8,
        quantity: u32,
    },
}

impl Plan {
    pub fn valid(self) -> bool {
        match self {
            Self::Furniture(index) => index < CATALOG.len(),
            Self::Project(Project::Room(index)) => index < 2,
            Self::Tool { tier, .. } => (1..=2).contains(&tier),
            Self::Material { batches, .. } => (1..=5).contains(&batches),
            _ => true,
        }
    }

    pub fn finished(self, garden: &Garden) -> bool {
        match self {
            Self::Furniture(index) => garden.crafted[index],
            Self::Project(project) => project.built(garden),
            Self::Tool { tool, tier } => tool.level(garden) >= tier,
            Self::Material {
                recipe, quantity, ..
            } => garden.stock(Product::Crafted(recipe)) >= quantity,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Furniture(index) => CATALOG[index].name,
            Self::Project(project) => project.name(),
            Self::Tool { tool, .. } => tool.name(),
            Self::Material { recipe, .. } => recipe.name(),
        }
    }

    fn ingredients(self, garden: &Garden) -> (Vec<Need>, u32) {
        match self {
            Self::Furniture(index) => (CATALOG[index].ingredients().to_vec(), 1),
            Self::Project(project) => (project.ingredients().to_vec(), 1),
            Self::Tool { tool, tier } => (tool.ingredients(tier).to_vec(), 1),
            Self::Material {
                recipe,
                batches,
                quantity,
            } => {
                let missing = quantity.saturating_sub(garden.stock(Product::Crafted(recipe)));
                let batches = missing.div_ceil(recipe.yield_count()).min(batches.into());
                (recipe.ingredients().to_vec(), batches)
            }
        }
    }

    fn station(self) -> Target {
        match self {
            Self::Tool { tier: 2, .. } => Target::Forge,
            Self::Material { recipe, .. } => match recipe.sort() {
                Bench::Work => Target::Bench,
                Bench::Forge => Target::Forge,
                Bench::Loom => Target::Loom,
            },
            _ => Target::Bench,
        }
    }

    fn unlock(self, garden: &Garden) -> Option<(String, Option<Target>)> {
        match self {
            Self::Furniture(index) if !garden.blueprints[index] => {
                let model = &CATALOG[index];
                Some((
                    model.source(garden),
                    match model.blueprint {
                        Blueprint::Shop { .. } => Some(Target::PatternStall),
                        _ => None,
                    },
                ))
            }
            Self::Project(Project::Room(index)) if !garden.workshop.rooms[index] => Some((
                "Bauplan am Stand in Eichenhafen".into(),
                Some(Target::PatternStall),
            )),
            Self::Material { recipe, .. } if !recipe.learned(garden) => {
                recipe.source().map(|watch| {
                    (
                        format!("Bauplan am Wegzeichen · {}", watch.name()),
                        Some(Target::WatchTrace(watch)),
                    )
                })
            }
            _ if self != Self::Project(Project::Restore) && !garden.workshop.restored => Some((
                "Zuerst die Werkbank instand setzen".into(),
                Some(Target::Bench),
            )),
            _ if self.station() == Target::Forge && !garden.workshop.forge => Some((
                "Schmiedetisch an der Werkbank bauen".into(),
                Some(Target::Bench),
            )),
            _ if self.station() == Target::Loom && !garden.workshop.loom => Some((
                "Webrahmen an der Werkbank bauen".into(),
                Some(Target::Bench),
            )),
            _ => None,
        }
    }

    pub fn summary(self, garden: &Garden) -> String {
        if self.finished(garden) {
            return "Fertig".into();
        }
        let (ingredients, batches) = self.ingredients(garden);
        craft::detail(&ingredients, garden, batches)
    }

    pub fn detail(self, garden: &Garden) -> String {
        if self.finished(garden) {
            return "Bauziel erfüllt".into();
        }
        let (ingredients, batches) = self.ingredients(garden);
        let mut lines: Vec<String> = self
            .unlock(garden)
            .map(|(text, _)| text)
            .into_iter()
            .collect();
        let missing = ingredients.into_iter().filter_map(|need| {
            let (have, amount, name) = need.values(garden);
            let missing = (amount * batches).saturating_sub(have);
            (missing > 0).then(|| {
                let verb = if missing == 1 { "fehlt" } else { "fehlen" };
                format!("{missing} {name} {verb} · {}", source(need, garden).0)
            })
        });
        lines.extend(missing);
        if lines.is_empty() {
            "Material vollständig · bereit zur Fertigung".into()
        } else {
            lines.join("\n")
        }
    }

    pub fn target(self, garden: &Garden) -> Option<Target> {
        if self.finished(garden) {
            return None;
        }
        if let Some((_, target)) = self.unlock(garden) {
            return target;
        }
        let (ingredients, batches) = self.ingredients(garden);
        Some(
            ingredients
                .into_iter()
                .find_map(|need| {
                    let (have, amount, _) = need.values(garden);
                    (have < amount * batches).then_some(source(need, garden).1)
                })
                .unwrap_or_else(|| self.station()),
        )
    }
}

fn source(need: Need, garden: &Garden) -> (&'static str, Target) {
    let mine = Target::Portal;
    let forest = Target::Travel(District::Forest);
    let beach = Target::Travel(District::Beach);
    match need {
        Need::Wood(_) => ("Bäume am Hof und im Farnwald", forest),
        Need::Stone(_) => ("Findlinge im Farnwald und Minenstollen", mine),
        Need::Copper(_) => ("Kupferadern im Minenstollen", mine),
        Need::Coins(_) => ("Verkaufstruhe am Hof oder Miro", Target::SaleChest),
        Need::Crop(_, _) | Need::Stock(Product::Crop(_, _), _) => {
            ("Anbau am Hof · Saatgut bei Lina", Target::Person(Npc::Lina))
        }
        Need::Stock(product, _) => match product {
            Product::Silver => ("Silberadern ab Mine 3", mine),
            Product::Crystal => ("Kristalladern in der tiefen Mine", mine),
            Product::Mushroom | Product::Hazel | Product::Berry => {
                ("Sammelplätze im Farnwald", forest)
            }
            Product::Shell => ("Sammelplätze in der Muschelbucht", beach),
            Product::Fish => ("Angelsteg", Target::Pier),
            Product::Herring => ("Äußerer Fischersteg in Eichenhafen", Target::HarborBoard),
            Product::Wool => (
                "Schafe am Hof",
                Target::Village(Village::Animal(crate::homestead::Animal::Sheep)),
            ),
            Product::Egg => (
                "Hühner am Hof",
                Target::Village(Village::Animal(crate::homestead::Animal::Hen)),
            ),
            Product::Apple => ("Obstwiese am Hof", Target::Village(Village::Orchard(0))),
            Product::Prepared(_) => ("Küche am Hof", Target::Kitchen),
            Product::Crafted(recipe) => (
                recipe.sort().name(),
                match recipe.sort() {
                    Bench::Work => Target::Bench,
                    Bench::Forge => Target::Forge,
                    Bench::Loom => Target::Loom,
                },
            ),
            Product::Regional(material) => {
                let (area, place) = match material {
                    Material::MeadowFlax => (Frontier::Wind, "Flachs auf den Windweiden"),
                    Material::MarshHerb => (Frontier::Moor, "Kräuter im Nebelmoor"),
                    _ => (Frontier::Copper, "Sammelplätze in der Kupferklamm"),
                };
                (
                    place,
                    if area.unlocked(garden) {
                        Target::Travel(area.district())
                    } else {
                        Target::Gate(area)
                    },
                )
            }
            Product::Spoils(_) => ("Wegwachten und Dämmergänge", Target::DuskGate),
            Product::Seasonal(season) => (season.find_hint(), forest),
            Product::Crop(_, _) => unreachable!(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pinned_recipe_survives_a_save_and_marks_only_missing_supplies() {
        let mut garden = Garden::default();
        garden.workshop.restored = true;
        let plan = Plan::Material {
            recipe: Recipe::Boards,
            batches: 2,
            quantity: 4,
        };
        garden.plan = Some(plan);
        let mut saved: Garden =
            serde_json::from_str(&serde_json::to_string(&garden).unwrap()).unwrap();
        assert_eq!(saved.plan, Some(plan));
        assert!(plan.detail(&saved).contains("6 Holz fehlen"));
        assert_eq!(plan.target(&saved), Some(Target::Travel(District::Forest)));
        saved.wood = 6;
        assert_eq!(plan.target(&saved), Some(Target::Bench));
        Recipe::Boards.make(&mut saved, Bench::Work, 1).unwrap();
        saved.wood = 0;
        assert!(plan.detail(&saved).contains("3 Holz fehlen"));
        saved.wood = 3;
        Recipe::Boards.make(&mut saved, Bench::Work, 1).unwrap();
        assert!(plan.finished(&saved));
    }
}
