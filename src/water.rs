use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    scene::SceneInstanceReady,
};

pub type WaterMaterial = ExtendedMaterial<StandardMaterial, Ripples>;

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct Ripples {
    // Frequency, scroll speed and glint intensity. Geometry and vertex tint come from Blender.
    #[uniform(100)]
    pub surface: Vec4,
}

impl MaterialExtension for Ripples {
    fn fragment_shader() -> ShaderRef {
        "shaders/water.wgsl".into()
    }
}

pub fn bind(
    trigger: Trigger<SceneInstanceReady>,
    children: Query<&Children>,
    meshes: Query<(&Name, &MeshMaterial3d<StandardMaterial>)>,
    materials: Res<Assets<StandardMaterial>>,
    mut water: ResMut<Assets<WaterMaterial>>,
    mut commands: Commands,
) {
    for entity in children.iter_descendants(trigger.target()) {
        let Ok((name, material)) = meshes.get(entity) else {
            continue;
        };
        let label = name.as_str();
        let surface = if label.starts_with("Gezeitenwasser") {
            Vec4::new(2.6, 0.48, 0.20, 0.)
        } else if label.starts_with("Der Farnbach") || label.starts_with("Bachlauf am Außenweg") {
            Vec4::new(6.4, 0.72, 0.15, 0.)
        } else if label.starts_with("Teich · Wasseroberfläche") || label.starts_with("Gezeitentümpel") {
            Vec4::new(7.6, 0.36, 0.27, 0.)
        } else if label.starts_with("Brunnen · Wasseroberfläche") {
            Vec4::new(9.2, 0.61, 0.24, 0.)
        } else {
            continue;
        };
        let Some(base) = materials.get(material.id()) else {
            continue;
        };
        let material = water.add(WaterMaterial {
            base: base.clone(),
            extension: Ripples { surface },
        });
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert(MeshMaterial3d(material));
    }
}
