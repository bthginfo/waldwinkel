use bevy::{
    input::{
        ButtonState,
        mouse::MouseButtonInput,
        touch::{TouchInput, TouchPhase},
    },
    prelude::*,
    render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages},
    window::PrimaryWindow,
};
use serde::{Deserialize, Serialize};

use crate::{
    Adventure, Area, Garden, GardenFocus, Notice,
    adventure::{HOME, SLOTS, commit},
    controller::Controller,
    furniture::CATALOG,
    garden::SavePath,
    homestead::Animal,
    session::Session,
    ui::{self, UiFonts},
    workshop,
    world::{Gardener, IslandCamera, PLOTS},
};

const CELL: f32 = 0.50;
const YARD_MIN: Vec2 = Vec2::new(-12.5, -4.5);
const YARD_MAX: Vec2 = Vec2::new(15.5, 15.5);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Pose {
    cell: [i16; 2],
    quarter: u8,
}

impl Pose {
    fn at(point: Vec2, quarter: u8) -> Self {
        let grid = (point / CELL).round();
        Self {
            cell: [grid.x as i16, grid.y as i16],
            quarter: quarter % 4,
        }
    }

    pub fn point(self) -> Vec2 {
        Vec2::new(self.cell[0] as f32, self.cell[1] as f32) * CELL
    }

    pub fn angle(self) -> f32 {
        f32::from(self.quarter % 4) * std::f32::consts::FRAC_PI_2
    }

    pub fn offset(self, vector: Vec2) -> Vec2 {
        let (sin, cos) = self.angle().sin_cos();
        Vec2::new(
            cos * vector.x + sin * vector.y,
            -sin * vector.x + cos * vector.y,
        )
    }

    fn half(self, half: Vec2) -> Vec2 {
        if self.quarter.is_multiple_of(2) {
            half
        } else {
            half.yx()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Furniture {
    pub design: usize,
    pub inside: bool,
    pub pose: Pose,
}

impl Furniture {
    pub fn here(self, area: Area) -> bool {
        (self.inside && area == Area::Home) || (!self.inside && area.outdoors())
    }

    fn half(self) -> Vec2 {
        self.pose.half(CATALOG[self.design].half)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Building {
    Bench,
    Forge,
    Loom,
    Coop,
    Barn,
}

impl Building {
    pub const ALL: [Self; 5] = [Self::Bench, Self::Forge, Self::Loom, Self::Coop, Self::Barn];

    pub fn animal(kind: Animal) -> Self {
        match kind {
            Animal::Hen => Self::Coop,
            Animal::Sheep => Self::Barn,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Bench => "Werkbank",
            Self::Forge => "Schmiedetisch",
            Self::Loom => "Webrahmen",
            Self::Coop => "Hühnerstall",
            Self::Barn => "Schafscheune",
        }
    }

    pub fn model(self) -> &'static str {
        match self {
            Self::Bench => "furniture_bench",
            Self::Forge => "workshop_forge",
            Self::Loom => "workshop_loom",
            Self::Coop => "farm_coop",
            Self::Barn => "farm_barn",
        }
    }

    pub fn built(self, garden: &Garden) -> bool {
        match self {
            Self::Bench => garden.workshop.restored,
            Self::Forge => garden.workshop.forge,
            Self::Loom => garden.workshop.loom,
            Self::Coop => garden.farm.housing[0],
            Self::Barn => garden.farm.housing[1],
        }
    }

    pub fn default_point(self) -> Vec2 {
        match self {
            Self::Bench => workshop::BENCH,
            Self::Forge => workshop::FORGE,
            Self::Loom => workshop::LOOM,
            Self::Coop => Animal::Hen.site(),
            Self::Barn => Animal::Sheep.site(),
        }
    }

    pub fn point(self, garden: &Garden) -> Vec2 {
        garden.building_layout[self as usize].map_or(self.default_point(), Pose::point)
    }

    pub fn angle(self, garden: &Garden) -> f32 {
        garden.building_layout[self as usize].map_or(0., Pose::angle)
    }

    pub fn offset(self, garden: &Garden, offset: Vec2) -> Vec2 {
        garden.building_layout[self as usize].map_or(offset, |pose| pose.offset(offset))
    }

    fn half(self) -> Vec2 {
        match self {
            Self::Coop | Self::Barn => Vec2::new(1.57, 1.22),
            _ => Vec2::new(0.71, 0.56),
        }
    }

    fn height(self) -> f32 {
        match self {
            Self::Coop | Self::Barn => 2.16,
            _ => 1.15,
        }
    }

    fn footprint(self, garden: &Garden) -> Vec2 {
        garden.building_layout[self as usize].map_or(self.half(), |pose| pose.half(self.half()))
    }

    fn obstacle(self, garden: &Garden) -> bool {
        self.built(garden) || matches!(self, Self::Bench | Self::Coop | Self::Barn)
    }
}

fn overlaps(point: Vec2, center: Vec2, half: Vec2) -> bool {
    (point - center).abs().cmplt(half).all()
}

pub fn blocked(garden: &Garden, area: Area, point: Vec2) -> bool {
    (area.outdoors()
        && Building::ALL.into_iter().any(|building| {
            building.obstacle(garden)
                && overlaps(point, building.point(garden), building.footprint(garden))
        }))
        || (area == Area::Home
            && garden.furniture.iter().zip(SLOTS).any(|(&kind, at)| {
                kind.checked_sub(1)
                    .and_then(|i| CATALOG.get(i as usize))
                    .is_some_and(|design| overlaps(point, HOME + at, design.half))
            }))
        || garden.decorations.iter().copied().any(|piece| {
            piece.here(area)
                && CATALOG.get(piece.design).is_some()
                && overlaps(point, piece.pose.point(), piece.half())
        })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Item {
    Furniture(usize),
    Building(Building),
}

impl Item {
    fn label(self) -> &'static str {
        match self {
            Self::Furniture(i) => CATALOG[i].name,
            Self::Building(item) => item.name(),
        }
    }

    fn model(self) -> &'static str {
        match self {
            Self::Furniture(i) => CATALOG[i].model,
            Self::Building(item) => item.model(),
        }
    }

    fn half(self) -> Vec2 {
        match self {
            Self::Furniture(i) => CATALOG[i].half,
            Self::Building(item) => item.half(),
        }
    }

    fn height(self) -> f32 {
        match self {
            Self::Furniture(i) => CATALOG[i].height,
            Self::Building(item) => item.height(),
        }
    }
}

#[derive(Clone, Copy)]
enum Source {
    New,
    Legacy(usize),
    Placed(usize),
    Building,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Original {
    Legacy(usize),
    Placed(usize),
    Building(Building),
}

#[derive(Clone, Copy)]
struct Proposal {
    area: Area,
    item: Item,
    source: Source,
    pose: Pose,
}

#[derive(Resource, Default)]
pub struct Layout {
    proposal: Option<Proposal>,
}

impl Layout {
    pub fn open(&self) -> bool {
        self.proposal.is_some()
    }

    pub fn place(&mut self, area: Area, design: usize, start: Vec2, garden: &Garden) -> bool {
        if let Some((index, item)) = garden
            .decorations
            .iter()
            .copied()
            .enumerate()
            .find(|(_, item)| item.design == design)
        {
            if !item.here(area) {
                return false;
            }
            self.move_furniture(area, index, item);
            return true;
        }
        if let Some(index) = garden
            .furniture
            .iter()
            .position(|&item| usize::from(item) == design + 1)
        {
            if area != Area::Home {
                return false;
            }
            self.move_legacy(index, design);
            return true;
        }
        self.proposal = Some(Proposal {
            area,
            item: Item::Furniture(design),
            source: Source::New,
            pose: Pose::at(start - Vec2::Y * 1.5, 0),
        });
        true
    }

    pub fn move_furniture(&mut self, area: Area, index: usize, item: Furniture) {
        self.proposal = Some(Proposal {
            area,
            item: Item::Furniture(item.design),
            source: Source::Placed(index),
            pose: item.pose,
        });
    }

    pub fn move_legacy(&mut self, index: usize, design: usize) {
        self.proposal = Some(Proposal {
            area: Area::Home,
            item: Item::Furniture(design),
            source: Source::Legacy(index),
            pose: Pose::at(HOME + SLOTS[index], 0),
        });
    }

    pub fn move_building(&mut self, area: Area, building: Building, garden: &Garden) {
        self.proposal = Some(Proposal {
            area,
            item: Item::Building(building),
            source: Source::Building,
            pose: garden.building_layout[building as usize]
                .unwrap_or_else(|| Pose::at(building.default_point(), 0)),
        });
    }
}

impl Proposal {
    fn original(self) -> Option<Original> {
        match (self.source, self.item) {
            (Source::Legacy(index), _) => Some(Original::Legacy(index)),
            (Source::Placed(index), _) => Some(Original::Placed(index)),
            (Source::Building, Item::Building(building)) => Some(Original::Building(building)),
            _ => None,
        }
    }

    fn reason(self, adventure: &Adventure, garden: &Garden, player: Vec2) -> Option<&'static str> {
        let point = self.pose.point();
        let half = self.pose.half(self.item.half()) + Vec2::splat(0.10);
        let mut samples = Vec::new();
        for x in [-1., -0.5, 0., 0.5, 1.] {
            for y in [-1., -0.5, 0., 0.5, 1.] {
                samples.push(point + half * Vec2::new(x, y));
            }
        }
        if self.area.outdoors() {
            if !(point - half).cmpge(YARD_MIN).all() || !(point + half).cmple(YARD_MAX).all() {
                return Some("Bleib mit dem ganzen Stück innerhalb des Hofes.");
            }
            let camera_front = Vec2::new(-0.35, 0.94);
            let over_bed = PLOTS.iter().take(garden.active_beds()).any(|&bed| {
                let displacement = point - bed;
                let hidden = displacement.dot(camera_front);
                overlaps(point, bed, half + Vec2::splat(0.75))
                    || (hidden > 0.
                        && hidden < self.item.height() * 1.5 + half.length()
                        && displacement.perp_dot(camera_front).abs() < half.length() + 0.62)
            });
            if over_bed {
                return Some("Lass die Beete und den Blick darauf frei.");
            }
        }
        if overlaps(player, point, half + Vec2::splat(0.35)) {
            return Some("Hier steht der Tanuki. Wähle ein benachbartes Feld.");
        }
        if samples
            .iter()
            .any(|&at| !adventure.static_walkable(at, garden))
        {
            return Some("Hier stehen ein Hindernis oder eine Wand.");
        }
        if self.area.outdoors()
            && Building::ALL.into_iter().any(|building| {
                self.item != Item::Building(building)
                    && building.obstacle(garden)
                    && overlaps(
                        point,
                        building.point(garden),
                        half + building.footprint(garden),
                    )
            })
        {
            return Some("Hier steht bereits ein Gebäude.");
        }
        if self.area == Area::Home
            && garden
                .furniture
                .iter()
                .zip(SLOTS)
                .enumerate()
                .any(|(i, (&kind, at))| {
                    !matches!(self.source, Source::Legacy(current) if current == i)
                        && kind
                            .checked_sub(1)
                            .and_then(|k| CATALOG.get(k as usize))
                            .is_some_and(|item| overlaps(point, HOME + at, half + item.half))
                })
        {
            return Some("Hier steht bereits ein Möbelstück.");
        }
        if garden
            .decorations
            .iter()
            .copied()
            .enumerate()
            .any(|(i, item)| {
                item.here(self.area)
                    && !matches!(self.source, Source::Placed(index) if index == i)
                    && overlaps(point, item.pose.point(), half + item.half())
            })
        {
            return Some("Hier steht bereits ein Möbelstück.");
        }
        None
    }
}

#[derive(Component)]
pub struct Grid(bool);
#[derive(Component)]
pub struct Panel;
#[derive(Component)]
pub struct Detail;
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum ButtonAction {
    Place,
    Turn,
    Close,
    Store,
}

fn grid_mesh(min: Vec2, max: Vec2, inside: bool) -> Mesh {
    let mut vertices = Vec::new();
    let pos = |p: Vec2| {
        let y = if inside {
            0.030
        } else {
            crate::terrain::ground(p) + 0.035
        };
        [p.x, y, p.y]
    };
    for x in (min.x / CELL).round() as i16..=(max.x / CELL).round() as i16 {
        for z in (min.y / CELL).round() as i16..=(max.y / CELL).round() as i16 {
            let point = CELL * Vec2::new(x as f32, z as f32);
            if point.x < max.x {
                vertices.extend([pos(point), pos(point + CELL * Vec2::X)]);
            }
            if point.y < max.y {
                vertices.extend([pos(point), pos(point + CELL * Vec2::Y)]);
            }
        }
    }
    Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
}

pub fn setup(
    mut commands: Commands,
    fonts: Res<UiFonts>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ink = materials.add(StandardMaterial {
        base_color: Color::srgba(0.77, 0.91, 0.79, 0.29),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    for (inside, min, max) in [
        (
            true,
            HOME + Vec2::new(-7.5, -2.5),
            HOME + Vec2::new(7.5, 3.),
        ),
        (false, YARD_MIN, YARD_MAX),
    ] {
        commands.spawn((
            Grid(inside),
            Mesh3d(meshes.add(grid_mesh(min, max, inside))),
            MeshMaterial3d(ink.clone()),
            Visibility::Hidden,
        ));
    }
    commands
        .spawn((
            Panel,
            GlobalZIndex(95),
            BackgroundColor(ui::cream()),
            BorderRadius::all(Val::Px(16.)),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                bottom: Val::Px(14.),
                left: Val::Percent(19.),
                right: Val::Percent(19.),
                padding: UiRect::all(Val::Px(10.)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(5.),
                ..default()
            },
        ))
        .with_children(|panel| {
            panel.spawn((Detail, fonts.bold("", 16.)));
            panel
                .spawn(Node {
                    width: Val::Percent(100.),
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(8.),
                    row_gap: Val::Px(4.),
                    ..default()
                })
                .with_children(|row| {
                    for (action, caption) in [
                        (ButtonAction::Place, "Platzieren"),
                        (ButtonAction::Turn, "Drehen"),
                        (ButtonAction::Store, "Einlagern"),
                        (ButtonAction::Close, "Abbrechen"),
                    ] {
                        row.spawn((action, ui::dialog_button(102.)))
                            .with_child(fonts.button(caption, 14.));
                    }
                });
        });
}

#[derive(Default)]
pub struct EditorPointer {
    already_editing: bool,
    finger: Option<u64>,
    mouse_down: bool,
}

pub fn input(
    mut layout: ResMut<Layout>,
    controller: Res<Controller>,
    keys: Res<ButtonInput<KeyCode>>,
    session: Res<Session>,
    mut adventure: ResMut<Adventure>,
    mut garden: ResMut<Garden>,
    mut focus: ResMut<GardenFocus>,
    player: Single<&Gardener>,
    camera: Single<(&Camera, &GlobalTransform), With<IslandCamera>>,
    window: Single<&Window, With<PrimaryWindow>>,
    buttons: Query<(&ButtonAction, &Interaction)>,
    input: (
        EventReader<TouchInput>,
        EventReader<MouseButtonInput>,
        Local<EditorPointer>,
    ),
    clock: Res<Time>,
    path: Res<SavePath>,
    mut notice: ResMut<Notice>,
) {
    let (mut touch, mut mouse, mut pointer) = input;
    let Some(mut proposal) = layout.proposal else {
        *pointer = EditorPointer::default();
        touch.clear();
        mouse.clear();
        return;
    };
    if !pointer.already_editing {
        pointer.already_editing = true;
        touch.clear();
        mouse.clear();
    }
    let mut finger = None;
    for event in touch.read() {
        match event.phase {
            TouchPhase::Started if pointer.finger.is_none() => pointer.finger = Some(event.id),
            TouchPhase::Ended | TouchPhase::Canceled if pointer.finger == Some(event.id) => {
                pointer.finger = None;
                if event.phase == TouchPhase::Ended {
                    finger = Some(event.position);
                }
            }
            _ => {}
        }
    }
    let mut click = false;
    for event in mouse
        .read()
        .filter(|event| event.button == MouseButton::Left)
    {
        match event.state {
            ButtonState::Pressed => pointer.mouse_down = true,
            ButtonState::Released => click |= std::mem::take(&mut pointer.mouse_down),
        }
    }
    if !session.playing() || proposal.area != adventure.area {
        layout.proposal = None;
        return;
    }
    let pressed = buttons
        .iter()
        .find_map(|(&action, &state)| (state == Interaction::Pressed).then_some(action));
    if controller.back || pressed == Some(ButtonAction::Close) || keys.just_pressed(KeyCode::Escape)
    {
        layout.proposal = None;
        return;
    }
    let mut step = controller.step;
    for (key, axis) in [
        (KeyCode::ArrowLeft, Vec2::NEG_X),
        (KeyCode::ArrowRight, Vec2::X),
        (KeyCode::ArrowUp, Vec2::NEG_Y),
        (KeyCode::ArrowDown, Vec2::Y),
    ] {
        if keys.just_pressed(key) {
            step = axis;
        }
    }
    let mut next = proposal.pose.point() + CELL * step;
    let at = finger.or_else(|| click.then(|| window.cursor_position()).flatten());
    if pressed.is_none()
        && let Some(at) = at
        && at.y < window.height() * 0.76
        && let Ok(ray) = camera.0.viewport_to_world(camera.1, at)
        && let Some(point) = crate::world::ground_point(ray, adventure.area, &garden)
    {
        next = point;
    }
    let (min, max) = if adventure.area.outdoors() {
        (YARD_MIN, YARD_MAX)
    } else {
        (HOME + Vec2::new(-7., -2.5), HOME + Vec2::new(7., 2.75))
    };
    let turn = controller.rotate
        || pressed == Some(ButtonAction::Turn)
        || keys.just_pressed(KeyCode::KeyR);
    proposal.pose = Pose::at(next.clamp(min, max), proposal.pose.quarter + u8::from(turn));
    layout.proposal = Some(proposal);
    let place = controller.accept
        || pressed == Some(ButtonAction::Place)
        || keys.just_pressed(KeyCode::Enter);
    if pressed == Some(ButtonAction::Store) || controller.store {
        match proposal.source {
            Source::Legacy(i) => garden.furniture[i] = 0,
            Source::Placed(i) => {
                garden.decorations.remove(i);
            }
            _ => return,
        }
    } else if place
        && proposal
            .reason(&adventure, &garden, player.position)
            .is_none()
    {
        if let Item::Building(building) = proposal.item {
            garden.building_layout[building as usize] = Some(proposal.pose);
        } else if let Item::Furniture(design) = proposal.item {
            let item = Furniture {
                design,
                inside: adventure.area == Area::Home,
                pose: proposal.pose,
            };
            match proposal.source {
                Source::Placed(i) => garden.decorations[i] = item,
                Source::Legacy(i) => {
                    garden.furniture[i] = 0;
                    garden.decorations.push(item);
                }
                _ => garden.decorations.push(item),
            }
        }
    } else {
        return;
    }
    layout.proposal = None;
    adventure.focus = None;
    focus.selected = None;
    focus.pad_lock = false;
    commit(
        &garden,
        &path,
        &mut notice,
        clock.elapsed_secs(),
        format!("{} · Einrichtung gespeichert", proposal.item.label()),
        None,
        None,
    );
}

#[derive(Default)]
pub struct Drawing {
    placed: Vec<Furniture>,
    area: Option<Area>,
    root: Option<Entity>,
    ghost: Option<(Item, Entity)>,
}

pub fn draw(
    mut commands: Commands,
    layout: Res<Layout>,
    adventure: Res<Adventure>,
    garden: Res<Garden>,
    player: Single<&Gardener>,
    assets: Res<AssetServer>,
    controller: Res<Controller>,
    mut drawing: Local<Drawing>,
    mut grid: Query<(&Grid, &mut Visibility)>,
    mut panel: Single<&mut Node, (With<Panel>, Without<ButtonAction>)>,
    mut label: Single<&mut Text, With<Detail>>,
    mut buttons: Query<
        (&ButtonAction, &mut Node, &Interaction, &mut BackgroundColor),
        Without<Panel>,
    >,
) {
    let area = adventure.area;
    let height = |area: Area, point: Vec2| {
        if area.outdoors() {
            crate::world::exterior_ground(point)
        } else {
            0.079
        }
    };
    let scene = |name: &str| SceneRoot(assets.load(format!("models/{name}.glb#Scene0")));
    let transform = |area: Area, pose: Pose| {
        let p = pose.point();
        Transform::from_xyz(p.x, height(area, p), p.y)
            .with_rotation(Quat::from_rotation_y(pose.angle()))
    };
    if drawing.area != Some(area) || drawing.placed != garden.decorations {
        if let Some(root) = drawing.root.take() {
            commands.entity(root).despawn();
        }
        drawing.root = Some(
            commands
                .spawn(Transform::default())
                .with_children(|furniture| {
                    for (index, piece) in garden
                        .decorations
                        .iter()
                        .copied()
                        .enumerate()
                        .filter(|(_, p)| p.here(area))
                    {
                        if let Some(design) = CATALOG.get(piece.design) {
                            furniture.spawn((
                                Original::Placed(index),
                                scene(design.model),
                                transform(area, piece.pose),
                            ));
                        }
                    }
                })
                .id(),
        );
        drawing.area = Some(area);
        drawing.placed.clone_from(&garden.decorations);
    }
    let proposal = layout.proposal;
    for (Grid(inside), mut visibility) in &mut grid {
        visibility.set_if_neq(
            if proposal.is_some_and(|p| (p.area == Area::Home) == *inside) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            },
        );
    }
    if drawing
        .ghost
        .is_some_and(|(item, _)| !proposal.is_some_and(|p| p.item == item))
    {
        if let Some((_, ghost)) = drawing.ghost.take() {
            commands.entity(ghost).despawn();
        }
    }
    let Some(proposal) = proposal else {
        panel.display = Display::None;
        return;
    };
    if let Some((_, ghost)) = drawing.ghost {
        commands
            .entity(ghost)
            .insert(transform(proposal.area, proposal.pose));
    } else {
        let ghost = commands
            .spawn((
                scene(proposal.item.model()),
                transform(proposal.area, proposal.pose),
            ))
            .id();
        drawing.ghost = Some((proposal.item, ghost));
    }
    panel.display = Display::Flex;
    let blocked = proposal.reason(&adventure, &garden, player.position);
    let store = matches!(proposal.source, Source::Legacy(_) | Source::Placed(_));
    let help = if controller.active && store {
        "Stick · Bewegen   X · Drehen   A · Platzieren   Y · Einlagern   B · Zurück"
    } else if controller.active {
        "Stick · Bewegen   X · Drehen   A · Platzieren   B · Zurück"
    } else {
        "Tippe ein Feld im Raster an. Drehe oder platziere das Stück."
    };
    let status = blocked.map_or(proposal.item.label().into(), |why| {
        format!("{} · {why}", proposal.item.label())
    });
    let text = format!("{status}\n{help}");
    if label.0 != text {
        label.0 = text;
    }
    for (&action, mut node, &hover, mut color) in &mut buttons {
        node.display = if action == ButtonAction::Store && !store {
            Display::None
        } else {
            Display::Flex
        };
        color.0 = ui::button_color(hover, action != ButtonAction::Place || blocked.is_none());
    }
}

pub fn originals(layout: Res<Layout>, mut scenes: Query<(&Original, &mut Visibility)>) {
    let hidden = layout.proposal.and_then(Proposal::original);
    for (original, mut visibility) in &mut scenes {
        visibility.set_if_neq(if hidden == Some(*original) {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editing_hides_only_the_original_and_cancel_restores_it() {
        let mut layout = Layout::default();
        layout.move_legacy(0, 0);

        let mut app = App::new();
        app.insert_resource(layout).add_systems(Update, originals);
        let original = app
            .world_mut()
            .spawn((Original::Legacy(0), Visibility::Inherited))
            .id();
        let neighbour = app
            .world_mut()
            .spawn((Original::Legacy(1), Visibility::Inherited))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(original),
            Some(&Visibility::Hidden)
        );
        assert_eq!(
            app.world().get::<Visibility>(neighbour),
            Some(&Visibility::Inherited)
        );

        app.world_mut().resource_mut::<Layout>().proposal = None;
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(original),
            Some(&Visibility::Inherited)
        );
    }
}
