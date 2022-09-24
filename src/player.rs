use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::prelude::*;

use crate::{
    camera::CameraPlugin,
    controls::{Controlled, ControlsPlugin, Forward, Speed},
    fuel::Fuel,
    jump::JumpImpulse,
};

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let capsule_radius = 0.5;
    let capsule_depth = 2.0 * capsule_radius;

    let initial_jump_impulse = 5. * Vec3::Y;

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
                radius: capsule_radius,
                depth: capsule_depth,
                ..default()
            })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.3).into()),
            transform: Transform::from_xyz(0.0, 3.0 * capsule_depth, 0.0),
            ..default()
        })
        .insert(Collider::capsule_y(capsule_depth / 2.0, capsule_radius))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(RigidBody::Dynamic)
        .insert(ColliderMassProperties::Density(1.0))
        .insert(ExternalForce::default())
        .insert(ExternalImpulse::default())
        .insert(JumpImpulse {
            value: initial_jump_impulse,
        })
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Forward { value: Vec3::X })
        .insert(Speed { value: 2.0 })
        .insert(Controlled {
            rotating: false,
            forward: false,
            backward: false,
            left: false,
            right: false,
            hovering: false,
        })
        .insert(Fuel { value: 1.0 })
        .with_children(|parent| {
            crate::camera::setup(parent);
        });
}

pub fn move_controlled(
    time: Res<Time>,
    mut controlled_query: Query<(&Controlled, &Speed, &Forward, &mut Transform)>,
) {
    let delta_seconds = time.delta_seconds();

    for (controlled, speed, forward, mut transform) in controlled_query.iter_mut() {
        let mut movement = Vec3::ZERO;

        if controlled.forward {
            movement += delta_seconds * speed.value * forward.value;
        }

        if controlled.backward {
            movement += delta_seconds * speed.value * -forward.value;
        }

        let right = forward.value.cross(Vec3::Y).normalize();

        if controlled.left {
            movement += delta_seconds * speed.value * -right;
        }

        if controlled.right {
            movement += delta_seconds * speed.value * right;
        }

        transform.translation += movement;
    }
}

pub fn rotate_controlled(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<(&Controlled, &mut Forward, &mut Transform, &Children)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Controlled>)>,
) {
    let delta: Vec2 = mouse_motion_events
        .iter()
        .fold(Vec2::ZERO, |delta, mouse_motion_event| {
            delta + mouse_motion_event.delta
        });

    for (controlled, mut forward, mut transform, children) in query.iter_mut() {
        if controlled.rotating {
            let rotation = Quat::from_rotation_y(0.005 * -delta.x);
            forward.value = rotation * forward.value;
            transform.rotate(rotation);

            for child in children.iter() {
                if let Ok(mut camera_transform) = camera_query.get_mut(*child) {
                    let camera_rotation = Quat::from_axis_angle(Vec3::Z, 0.005 * -delta.y);

                    camera_transform.rotate_around(Vec3::ZERO, camera_rotation);
                }
            }
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ControlsPlugin)
            .add_plugin(CameraPlugin)
            .add_startup_system(setup)
            .add_system(move_controlled)
            .add_system(rotate_controlled);
    }
}
