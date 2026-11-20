use bevy::prelude::*;

use crate::{
    Adventure, Area, GardenFocus,
    garden::{Action, Crop, Garden},
    workshop::ToolKind,
    world::PLOTS,
};

const ROW_WIDTH: usize = 3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldTool {
    #[default]
    Auto,
    Hoe,
    Seeds,
    Water,
    Harvest,
    Fertilizer,
    Axe,
    Pickaxe,
}

impl FieldTool {
    pub fn name(self) -> &'static str {
        match self {
            Self::Auto => "Automatisch",
            Self::Hoe => "Hacke",
            Self::Seeds => "Saatgut",
            Self::Water => "Gießkanne",
            Self::Harvest => "Erntekorb",
            Self::Fertilizer => "Dünger",
            Self::Axe => "Axt",
            Self::Pickaxe => "Spitzhacke",
        }
    }

    /// A single contextual stroke can use the needed tool without changing the
    /// tool equipped for sweeping a row. The explicit fertilizer shortcut stays separate.
    pub fn for_bed(self, garden: &Garden, plot: usize, crop: Crop) -> Self {
        if self.action(garden, plot, crop).is_some() { self } else { Self::Auto }
    }

    pub fn action(self, garden: &Garden, plot: usize, crop: Crop) -> Option<Action> {
        if self == Self::Fertilizer {
            return (!garden.clock.night() && garden.fertilizable(plot) && garden.fertilizer > 0)
                .then_some(Action::Fertilize);
        }
        let action = garden.action(plot, crop)?;
        match self {
            Self::Auto => Some(action),
            Self::Hoe if action == Action::Till => Some(action),
            Self::Seeds if action == Action::Sow => Some(action),
            Self::Water if action == Action::Water => Some(action),
            Self::Harvest if action == Action::Harvest => Some(action),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Stroke {
    targets: [usize; ROW_WIDTH],
    count: usize,
    pub next: Option<usize>,
}

impl Stroke {
    pub fn targets(&self) -> &[usize] {
        &self.targets[..self.count]
    }
}

/// A row is a set of three neighboring beds. The active tool stays on that
/// row and picks the next eligible bed after the current stroke.
pub fn stroke(garden: &Garden, plot: usize, crop: Crop, action: Action, tool: FieldTool) -> Stroke {
    let mut result = Stroke {
        targets: [plot; ROW_WIDTH],
        count: 1,
        next: None,
    };
    if tool == FieldTool::Auto || action == Action::Fertilize {
        return result;
    }

    let sweep = match action {
        Action::Till => ToolKind::Hoe.level(garden),
        Action::Water => ToolKind::Can.level(garden),
        _ => 0,
    };
    let wide = usize::from(sweep) + 1;
    let row_start = plot / ROW_WIDTH * ROW_WIDTH;
    let mut neighbors = [row_start, row_start + 1, row_start + 2];
    neighbors.sort_by_key(|&other| (other.abs_diff(plot), other < plot));
    for neighbor in neighbors {
        if neighbor == plot || garden.action(neighbor, crop) != Some(action) {
            continue;
        }
        if result.count < wide {
            result.targets[result.count] = neighbor;
            result.count += 1;
        } else if action != Action::Sow || garden.seed_count(crop) > 1 {
            result.next = Some(neighbor);
            break;
        }
    }
    result
}

#[derive(Component)]
pub struct BedPreview(usize);

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let strip = meshes.add(Cuboid::new(0.27, 0.017, 0.06));
    let color = |base_color| StandardMaterial {
        base_color,
        unlit: true,
        ..default()
    };
    let next = materials.add(color(Color::srgb_u8(111, 224, 194)));
    let sweep = materials.add(color(Color::srgb_u8(171, 232, 238)));
    for i in 0..3 {
        let material = if i == 0 { &next } else { &sweep };
        commands
            .spawn((BedPreview(i), Transform::default(), Visibility::Hidden))
            .with_children(|frame| {
                for x in [-0.72, 0.72] {
                    for z in [-0.72, 0.72] {
                        for angle in [0., std::f32::consts::FRAC_PI_2] {
                            let point = if angle == 0. {
                                Vec3::new(x * 0.85, 0., z)
                            } else {
                                Vec3::new(x, 0., z * 0.85)
                            };
                            frame.spawn((
                                Mesh3d(strip.clone()),
                                MeshMaterial3d(material.clone()),
                                Transform::from_translation(point)
                                    .with_rotation(Quat::from_rotation_y(angle)),
                            ));
                        }
                    }
                }
            });
    }
}

pub fn show(
    focus: Res<GardenFocus>,
    adventure: Res<Adventure>,
    garden: Res<Garden>,
    mut frames: Query<(&BedPreview, &mut Transform, &mut Visibility)>,
) {
    let preview = focus
        .selected
        .filter(|_| adventure.area == Area::Island && !adventure.map)
        .and_then(|plot| {
            let action = focus.tool.action(&garden, plot, adventure.crop)?;
            Some(stroke(&garden, plot, adventure.crop, action, focus.tool))
        });
    for (frame, mut transform, mut visible) in &mut frames {
        let target = preview.as_ref().and_then(|plan| {
            if frame.0 == 0 {
                plan.next
            } else {
                plan.targets().get(frame.0).copied()
            }
        });
        if let Some(index) = target {
            let point = PLOTS[index];
            transform.translation = Vec3::new(point.x, 0.14, point.y);
        }
        visible.set_if_neq(if target.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garden::Plot;

    #[test]
    fn tending_one_bed_follows_its_state_and_preserves_row_tools() {
        let mut garden = Garden::default();
        let crop = Crop::Carrot;
        garden.plots[0] = Plot::Tilled;
        assert_eq!(FieldTool::Seeds.for_bed(&garden, 0, crop), FieldTool::Seeds);
        garden.work(0, crop, Action::Sow).unwrap();
        let tool = FieldTool::Seeds.for_bed(&garden, 0, crop);
        assert_eq!(tool.action(&garden, 0, crop), Some(Action::Water));
        assert_eq!(stroke(&garden,0,crop,Action::Water,tool).targets(), &[0]);
        assert_eq!(FieldTool::Water.for_bed(&garden,0,crop), FieldTool::Water);
        garden.fertilizer = 2;
        garden.clock = crate::day::FarmClock::from_minutes(22. * 60.);
        assert_eq!(FieldTool::Fertilizer.action(&garden, 0, crop), None);
    }

    #[test]
    fn silver_can_waters_only_eligible_beds_in_its_own_row() {
        let mut garden = Garden::default();
        let mut tools = [0u8; 4];
        tools[ToolKind::Can as usize] = 2;
        garden.workshop = serde_json::from_value(serde_json::json!({"tools": tools})).unwrap();
        garden.plots.fill(Plot::Sown {
            crop: Crop::Carrot,
            growth: 0,
            wet: false,
        });
        garden.work(1, Crop::Carrot, Action::Water).unwrap();
        let stroke = stroke(&garden, 0, Crop::Carrot, Action::Water, FieldTool::Water);
        assert_eq!(stroke.targets(), &[0, 2]);
        assert_eq!(stroke.next, None);
        assert_eq!(garden.action(3, Crop::Carrot), Some(Action::Water));
    }

    #[test]
    fn fixed_tool_shows_next_bed_and_stops_before_using_unavailable_seed() {
        let mut garden = Garden::default();
        garden.plots.fill(Plot::Tilled);
        let plan = stroke(&garden, 0, Crop::Carrot, Action::Sow, FieldTool::Seeds);
        assert_eq!(plan.targets(), &[0]);
        assert_eq!(plan.next, Some(1));
        garden.seeds = 1;
        assert_eq!(
            stroke(&garden, 0, Crop::Carrot, Action::Sow, FieldTool::Seeds).next,
            None
        );
        assert_eq!(FieldTool::Water.action(&garden, 0, Crop::Carrot), None);
    }
}
