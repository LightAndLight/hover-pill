use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::prelude::*;

use crate::{
    camera::CameraBundle,
    controls::{Controlled, Forward, Speed},
    fuel::{Fuel, FuelChanged},
    hover::Hovering,
    jump::JumpImpulse,
};

pub const CAPSULE_RADIUS: f32 = 0.5;
pub const CAPSULE_DEPTH: f32 = 2.0 * CAPSULE_RADIUS;
pub const CAPSULE_COLOR: Color = Color::rgb(0.8, 0.7, 0.3);

pub fn spawn_player(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    transform: Transform,
    fuel_changed: &mut EventWriter<FuelChanged>,
) -> Entity {
    let initial_jump_impulse = 5. * Vec3::Y;

    let fuel = Fuel { value: 1.0 };

    let entity = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
                radius: CAPSULE_RADIUS,
                depth: CAPSULE_DEPTH,
                ..default()
            })),
            material: materials.add(CAPSULE_COLOR.into()),
            transform,
            ..default()
        })
        .insert(Collider::capsule_y(CAPSULE_DEPTH / 2.0, CAPSULE_RADIUS))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(RigidBody::Dynamic)
        .insert(ColliderMassProperties::Density(1.0))
        .insert(ExternalForce::default())
        .insert(ExternalImpulse::default())
        .insert(JumpImpulse {
            value: initial_jump_impulse,
        })
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Forward { value: Vec3::Z })
        .insert(Speed { value: 3.5 })
        .insert(Controlled::default())
        .insert(fuel)
        .insert(Hovering { value: false })
        .with_children(|parent| {
            parent.spawn_bundle(CameraBundle::new(Transform::from_xyz(0.0, 4.0, -5.0)));
        })
        .id();

    fuel_changed.send(FuelChanged {
        new_value: fuel.value,
    });

    entity
}

fn move_controlled(
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

fn rotate_controlled(
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
                    let camera_rotation = Quat::from_axis_angle(Vec3::X, 0.005 * delta.y);

                    camera_transform.rotate_around(Vec3::ZERO, camera_rotation);
                }
            }
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_controlled)
            .add_system(rotate_controlled);
    }
}
