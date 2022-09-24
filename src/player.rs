use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
};
use bevy_rapier3d::prelude::*;

use crate::{
    camera::CameraPlugin,
    controls::{move_controlled, rotate_controlled, Controlled, Forward, Speed},
    fuel::Fuel,
    hover::{end_hover, start_hover},
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

fn handle_keys(
    keys: Res<Input<KeyCode>>,
    mut query: Query<(
        &mut Controlled,
        &Fuel,
        &JumpImpulse,
        &mut ExternalImpulse,
        &mut ExternalForce,
    )>,
) {
    for (mut controlled, fuel, jump_impulse, mut external_impulse, mut external_force) in
        query.iter_mut()
    {
        if keys.just_pressed(KeyCode::W) {
            controlled.forward = true;
        }

        if keys.just_released(KeyCode::W) {
            controlled.forward = false;
        }

        if keys.just_pressed(KeyCode::A) {
            controlled.left = true;
        }

        if keys.just_released(KeyCode::A) {
            controlled.left = false;
        }

        if keys.just_pressed(KeyCode::S) {
            controlled.backward = true;
        }

        if keys.just_released(KeyCode::S) {
            controlled.backward = false;
        }

        if keys.just_pressed(KeyCode::D) {
            controlled.right = true;
        }

        if keys.just_released(KeyCode::D) {
            controlled.right = false;
        }

        if keys.just_pressed(KeyCode::Space) {
            start_hover(
                fuel,
                &mut controlled,
                &mut external_impulse,
                jump_impulse,
                &mut external_force,
            );
        }

        if keys.just_released(KeyCode::Space) {
            end_hover(&mut controlled, &mut external_impulse, &mut external_force);
        }
    }
}

fn set_controlled_rotating(
    mut windows: ResMut<Windows>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut query: Query<&mut Controlled>,
) {
    for mouse_button_event in mouse_button_events.iter() {
        if let MouseButton::Right = mouse_button_event.button {
            let window = windows.primary_mut();
            for mut controlled in query.iter_mut() {
                match mouse_button_event.state {
                    ButtonState::Pressed => {
                        controlled.rotating = true;
                        window.set_cursor_visibility(false);
                    }
                    ButtonState::Released => {
                        controlled.rotating = false;
                        window.set_cursor_visibility(true);
                    }
                };
            }
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(CameraPlugin)
            .add_startup_system(setup)
            .add_system(move_controlled)
            .add_system(rotate_controlled)
            .add_system(handle_keys)
            .add_system(set_controlled_rotating);
    }
}
