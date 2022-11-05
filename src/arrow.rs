use std::f32::consts::PI;

use bevy::{
    ecs::system::EntityCommands,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    transform::TransformBundle,
};
use bevy_rapier3d::{
    prelude::Collider,
    rapier::prelude::{ColliderBuilder, SharedShape},
};

use crate::{cone::Cone, cylinder::Cylinder};

pub fn spawn_generic<'w: 'a, 's: 'a, 'a>(
    spawn: impl FnOnce() -> EntityCommands<'w, 's, 'a>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    radius: f32,
    length: f32,
    transform: Transform,
) -> Entity {
    let cone_radius = 2.0 * radius;
    let cone_height = 2.0 * cone_radius;
    let cylinder_length = length - cone_height;

    spawn()
        .insert_bundle(VisibilityBundle::default())
        .insert_bundle(TransformBundle::default())
        .with_children(|parent| {
            parent
                .spawn_bundle(PbrBundle {
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
                    transform,
                    ..default()
                })
                .insert(NotShadowCaster)
                .insert(NotShadowReceiver)
                .with_children(|parent| {
                    parent
                        .spawn_bundle(TransformBundle::from(Transform::from_rotation(
                            Quat::from_rotation_x(PI / 2.0),
                        )))
                        .insert(Collider::cylinder(cylinder_length / 2.0, radius));
                });

            parent
                .spawn_bundle(PbrBundle {
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
                    transform: transform
                        * Transform::default().with_translation((cylinder_length / 2.0) * Vec3::Z),
                    ..default()
                })
                .insert(NotShadowCaster)
                .insert(NotShadowReceiver)
                .with_children(|parent| {
                    parent
                        .spawn_bundle(TransformBundle::from(
                            Transform::from_translation((cylinder_length - cone_height) * Vec3::Z)
                                * Transform::from_rotation(Quat::from_rotation_x(PI / 2.0)),
                        ))
                        .insert(Collider::cone(cone_height / 2.0, cone_radius));
                });
        })
        .id()
}

pub fn spawn<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    radius: f32,
    length: f32,
    transform: Transform,
) -> Entity {
    spawn_generic(
        || commands.spawn(),
        meshes,
        materials,
        radius,
        length,
        transform,
    )
}

pub fn spawn_child<'w, 's, 'a>(
    commands: &mut ChildBuilder<'w, 's, 'a>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    radius: f32,
    length: f32,
    transform: Transform,
) -> Entity {
    spawn_generic(
        || commands.spawn(),
        meshes,
        materials,
        radius,
        length,
        transform,
    )
}
