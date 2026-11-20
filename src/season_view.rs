use bevy::{
    gltf::{Gltf, GltfMesh},
    prelude::*,
    render::mesh::VertexAttributeValues,
};

use crate::{
    Adventure, Garden,
    weather::{Season, Sky},
};

#[derive(Component)]
pub struct SeasonLayer(pub Season);

#[derive(Resource)]
pub struct GroundSource(Handle<Gltf>);

pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(GroundSource(assets.load("models/world_ground.glb")));
}

pub fn visibility(
    garden: Res<Garden>,
    adventure: Res<Adventure>,
    mut layers: Query<(&SeasonLayer, &mut Visibility)>,
) {
    let season = Season::at(garden.day);
    for (layer, mut visibility) in &mut layers {
        let next = if layer.0 == season && adventure.area.outdoors() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != next {
            *visibility = next;
        }
    }
}

#[derive(Default)]
pub struct GroundPalette {
    current: Option<Season>,
    wet: Option<bool>,
    patches: Vec<(Handle<Mesh>, Vec<[f32; 4]>)>,
}

pub fn ground(
    garden: Res<Garden>,
    source: Res<GroundSource>,
    gltfs: Res<Assets<Gltf>>,
    primitives: Res<Assets<GltfMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut palette: Local<GroundPalette>,
) {
    let season = Season::at(garden.day);
    let wet = Sky::at(garden.day) == Sky::Rain;
    if palette.current == Some(season) && palette.wet == Some(wet) {
        return;
    }
    let Some(source) = gltfs.get(&source.0) else {
        return;
    };
    if palette.wet != Some(wet) {
        for handle in &source.materials {
            if let Some(material) = materials.get_mut(handle) {
                material.base_color = if wet {
                    Color::srgb(0.89, 0.93, 0.92)
                } else {
                    Color::WHITE
                };
            }
        }
        palette.wet = Some(wet);
    }
    if palette.current == Some(season) {
        return;
    }
    if palette.patches.is_empty() {
        let Some(originals) = source
            .meshes
            .iter()
            .map(|handle| primitives.get(handle))
            .collect::<Option<Vec<_>>>()
        else {
            return;
        };
        let Some(patches) = originals
            .iter()
            .flat_map(|original| &original.primitives)
            .map(|primitive| {
                let VertexAttributeValues::Float32x4(colors) = meshes
                    .get(&primitive.mesh)?
                    .attribute(Mesh::ATTRIBUTE_COLOR)?
                else {
                    return None;
                };
                Some((primitive.mesh.clone(), colors.clone()))
            })
            .collect::<Option<Vec<_>>>()
        else {
            return;
        };
        palette.patches = patches;
    }
    for (handle, original) in &palette.patches {
        let Some(mesh) = meshes.get_mut(handle) else {
            continue;
        };
        let Some(VertexAttributeValues::Float32x3(vertices)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            continue;
        };
        let colors = original
            .iter()
            .zip(vertices)
            .map(|(&color, vertex)| soil_color(color, vertex[1], season))
            .collect::<Vec<_>>();
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
    palette.current = Some(season);
}

fn soil_color([r, g, b, alpha]: [f32; 4], y: f32, season: Season) -> [f32; 4] {
    let grass = ((y + 0.47) * 2.22).clamp(0., 1.);
    let [red, green, blue] = match season {
        Season::Spring => [r * 0.97, g * 1.063, b * 1.046],
        Season::Summer => [r * 1.048, g * 1.017, b * 0.91],
        Season::Autumn => [r * 1.18, g * 0.94, b * 0.91],
        Season::Winter => {
            let shade = (0.66 + r * 0.28 + g * 0.60 + b * 0.12).min(1.0);
            [0.76 * shade, 0.85 * shade, 0.84 * shade]
        }
    };
    [
        r.lerp(red, grass),
        g.lerp(green, grass),
        b.lerp(blue, grass),
        alpha,
    ]
}
