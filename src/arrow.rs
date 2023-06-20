use std::f32::consts::PI;

use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};
use bevy_rapier3d::prelude::Collider;

use crate::{cone::Cone, cylinder::Cylinder};

#[derive(Copy, Clone, Component)]
pub struct Arrow;

pub fn spawn(
    commands: &mut ChildBuilder,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    radius: f32,
    length: f32,
    transform: Transform,
) -> Entity {
    let cone_radius = 2.0 * radius;
    let cone_height = 2.0 * cone_radius;
    let cylinder_length = length - cone_height;

    let material = materials.add(StandardMaterial {
        base_color: Color::NONE,
        emissive: Color::rgb(0.0, 1.0, 0.0),
        metallic: 0.0,
        reflectance: 0.0,
        ..default()
    });

    commands
        .spawn((
            Arrow,
            SpatialBundle {
                transform,
                ..default()
            },
            Collider::compound(vec![
                (
                    Vec3::ZERO,
                    Quat::from_rotation_x(PI / 2.0),
                    Collider::cylinder(cylinder_length / 2.0, radius),
                ),
                (
                    ((cone_height + cylinder_length) / 2.0) * Vec3::Z,
                    Quat::from_rotation_x(PI / 2.0),
                    Collider::cone(cone_height / 2.0, cone_radius),
                ),
            ]),
        ))
        .with_children(|parent| {
            parent
                .spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(Cylinder {
                        radius,
                        vertices: 64,
                        length: cylinder_length,
                    })),
                    material: material.clone(),
                    ..default()
                })
                .insert(NotShadowCaster)
                .insert(NotShadowReceiver);

            parent
                .spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(Cone {
                        radius: cone_radius,
                        vertices: 64,
                        height: cone_height,
                    })),
                    material,
                    transform: Transform::from_translation((cylinder_length / 2.0) * Vec3::Z),
                    ..default()
                })
                .insert(NotShadowCaster)
                .insert(NotShadowReceiver);
        })
        .id()
}
