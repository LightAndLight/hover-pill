use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(500.0, 0.1, 500.0))),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform::from_translation(Vec3::new(0., -0.05, 0.)),
            ..default()
        })
        .insert(Collider::cuboid(250.0, 0.05, 250.0));

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(1.5, 0.5, 1.5),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 0.5, 0.5));
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(1.5, 0.5, -1.5),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 0.5, 0.5));
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(-1.5, 0.5, 1.5),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 0.5, 0.5));
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(-1.5, 0.5, -1.5),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 0.5, 0.5));

    commands.spawn_bundle(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0)),
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            shadow_projection: OrthographicProjection {
                left: -10.,
                right: 10.,
                bottom: -10.,
                top: 10.,
                near: -10.,
                far: 10.,
                ..default()
            },
            ..default()
        },
        ..default()
    });
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}
