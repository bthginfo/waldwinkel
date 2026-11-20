use std::collections::HashMap;

use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    scene::{SceneInstanceReady, SceneSpawner},
};

use crate::{adventure::Adventure, garden::Garden, terrain, weather::Sky};

pub type FigureMaterial = ExtendedMaterial<StandardMaterial, FigureSurface>;

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct FigureSurface {
    #[uniform(100)]
    environment: Vec4,
}

impl MaterialExtension for FigureSurface {
    fn fragment_shader() -> ShaderRef {
        "shaders/figure.wgsl".into()
    }
}

#[derive(Component)]
pub struct LightKit(Vec<Handle<FigureMaterial>>);

#[derive(Component)]
pub struct Eye {
    opening: f32,
    phase: f32,
    period: f32,
}

pub fn bind(
    trigger: Trigger<SceneInstanceReady>,
    scenes: Res<SceneSpawner>,
    nodes: Query<(&Name, &Transform), Without<Mesh3d>>,
    meshes: Query<&MeshMaterial3d<StandardMaterial>>,
    source: Res<Assets<StandardMaterial>>,
    mut surfaces: ResMut<Assets<FigureMaterial>>,
    mut commands: Commands,
) {
    let mut kits = HashMap::new();
    for entity in scenes.iter_instance_entities(trigger.event().instance_id) {
        if let Ok((name, transform)) = nodes.get(entity)
            && let Some((figure, _)) = name
                .as_str()
                .strip_suffix("_eye")
                .and_then(|label| label.rsplit_once('_'))
        {
            let seed = figure
                .bytes()
                .fold(1_u32, |sum, c| sum.wrapping_mul(31).wrapping_add(c.into()));
            commands.entity(entity).insert(Eye {
                opening: transform.scale.y,
                phase: (seed % 71) as f32 * 0.063,
                period: 3.4 + (seed % 29) as f32 * 0.073,
            });
        }
        let Ok(mesh) = meshes.get(entity) else {
            continue;
        };
        let Some(base) = source.get(mesh.id()) else {
            continue;
        };
        let material = kits.entry(mesh.id()).or_insert_with(|| {
            surfaces.add(FigureMaterial {
                base: base.clone(),
                extension: FigureSurface {
                    environment: Vec4::new(1., 1., 0., 0.),
                },
            })
        });
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert(MeshMaterial3d(material.clone()));
    }
    if !kits.is_empty() {
        commands
            .entity(trigger.target())
            .insert(LightKit(kits.into_values().collect()));
    }
}

pub fn animate_eyes(time: Res<Time>, mut eyes: Query<(&Eye, &mut Transform)>) {
    for (eye, mut transform) in &mut eyes {
        let blink = ((time.elapsed_secs() + eye.phase) % eye.period / 0.18).min(1.);
        let closure = (blink * std::f32::consts::PI).sin().max(0.);
        transform.scale.y = eye.opening * (1. - closure.powi(2) * 0.89);
    }
}

pub fn shade(
    garden: Res<Garden>,
    adventure: Res<Adventure>,
    time: Res<Time>,
    figures: Query<(&LightKit, &GlobalTransform)>,
    mut surfaces: ResMut<Assets<FigureMaterial>>,
) {
    let outdoor = adventure.area.outdoors();
    let dusk = if outdoor { garden.clock.dusk() } else { 0. };
    let clear = match Sky::at(garden.day) {
        Sky::Sun => 1.,
        Sky::Mist => 0.63,
        Sky::Snow | Sky::Rain => 0.51,
    };
    let ease = 1. - (-time.delta_secs() * 9.).exp();
    for (kit, root) in &figures {
        let point = root.translation().xz();
        let cover = terrain::weather_cover(point) - terrain::ground(point);
        let shade = ((cover - 1.04) * 0.55).clamp(0., 1.);
        let profile = if outdoor {
            Vec4::new(
                clear * (1. - 0.64 * shade) * (1. - 0.77 * dusk),
                1. - 0.25 * shade,
                dusk,
                0.,
            )
        } else {
            Vec4::new(0.55, 1.09, 0., 1.)
        };
        for handle in &kit.0 {
            if surfaces.get(handle).is_some_and(|material| {
                material.extension.environment.distance_squared(profile) > 0.000025
            }) && let Some(material) = surfaces.get_mut(handle)
            {
                material.extension.environment = material.extension.environment.lerp(profile, ease);
            }
        }
    }
}
