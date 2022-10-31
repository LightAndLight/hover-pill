use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    transform::TransformBundle,
};

use crate::{cone::Cone, cylinder::Cylinder};

pub fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    radius: f32,
    length: f32,
) -> Entity {
    let cone_radius = 2.0 * radius;
    let cone_height = 2.0 * cone_radius;
    let cylinder_length = length - cone_height;

    commands
        .spawn()
        .insert_bundle(VisibilityBundle::default())
        .insert_bundle(TransformBundle::default())
        .insert(NotShadowCaster)
        .insert(NotShadowReceiver)
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(Cylinder {
                    radius,
                    vertices: 64,
                    length: cylinder_length,
                })),
                material: materials.add(StandardMaterial {
                    base_color: Color::NONE,
                    emissive: Color::rgb(0.0, 1.0, 0.0),
                    metallic: 0.0,
                    reflectance: 0.0,
                    ..default()
                }),
                transform: Transform::default(),
                ..default()
            });

            parent.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(Cone {
                    radius: cone_radius,
                    vertices: 64,
                    height: cone_height,
                })),
                material: materials.add(StandardMaterial {
                    base_color: Color::NONE,
                    emissive: Color::rgb(0.0, 1.0, 0.0),
                    metallic: 0.0,
                    reflectance: 0.0,
                    ..default()
                }),
                transform: Transform::default().with_translation((cylinder_length / 2.0) * Vec3::Z),
                ..default()
            });
        })
        .id()
}
