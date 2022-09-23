use std::f32::consts::PI;

use bevy::{
    input::{
        mouse::{MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel},
        ButtonState,
    },
    prelude::*,
};
use bevy_atmosphere::prelude::{AtmosphereCamera, AtmospherePlugin};

#[derive(Component)]
struct Speed {
    value: f32,
}

#[derive(Component)]
struct Forward {
    value: Vec3,
}

#[derive(Component)]
struct Controlled {
    rotating: bool,
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
}

#[derive(Component)]
struct Camera {
    value: Entity,
}

#[derive(Component)]
struct Zoom;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let camera = commands
        .spawn_bundle(Camera3dBundle {
            projection: PerspectiveProjection {
                fov: (60.0 / 360.0) * 2.0 * PI,
                ..default()
            }
            .into(),
            transform: Transform::from_xyz(-5.0, 4.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(AtmosphereCamera(None))
        .insert(Forward {
            value: Vec3::ZERO
                - Vec3 {
                    x: -5.0,
                    y: 4.0,
                    z: 0.0,
                },
        })
        .insert(Zoom)
        .id();

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
                radius: 0.5,
                depth: 1.0,
                ..default()
            })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.3).into()),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        })
        .insert(Forward { value: Vec3::X })
        .insert(Speed { value: 2.0 })
        .insert(Controlled {
            rotating: false,
            forward: false,
            backward: false,
            left: false,
            right: false,
        })
        .insert(Camera { value: camera });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 500.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(1.5, 0.5, 1.5),
        ..default()
    });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(1.5, 0.5, -1.5),
        ..default()
    });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(-1.5, 0.5, 1.5),
        ..default()
    });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(-1.5, 0.5, -1.5),
        ..default()
    });

    commands.spawn_bundle(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0 + 0.05)), // Transform::from_xyz(0.0, 10.0, 0.0),
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        ..default()
    });
}

fn move_entities(
    time: Res<Time>,
    mut controlled_query: Query<(
        &Controlled,
        &Speed,
        &Forward,
        &mut Transform,
        Option<&Camera>,
    )>,
    mut camera_query: Query<&mut Transform, Without<Controlled>>,
) {
    let delta_seconds = time.delta_seconds();

    for (controlled, speed, forward, mut transform, camera) in controlled_query.iter_mut() {
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

        if let Some(camera) = camera {
            let mut camera_transform = camera_query.get_mut(camera.value).unwrap();
            camera_transform.translation += movement;
        }
    }
}

fn handle_keys(keys: Res<Input<KeyCode>>, mut query: Query<&mut Controlled>) {
    for mut controlled in query.iter_mut() {
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
    }
}

fn scroll_zoom(
    mut scroll_events: EventReader<MouseWheel>,
    mut query: Query<(&Forward, &mut Transform), With<Zoom>>,
) {
    let scroll_amount: f32 = scroll_events
        .iter()
        .map(|scroll_event| match scroll_event.unit {
            MouseScrollUnit::Line => scroll_event.y,
            unit => {
                warn!("unsupported scroll unit: {:?}", unit);
                0.0
            }
        })
        .sum();

    for (forward, mut transform) in query.iter_mut() {
        transform.translation += 0.08 * scroll_amount * forward.value;
    }
}

fn set_controlled_rotating(
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut query: Query<&mut Controlled>,
) {
    for mouse_button_event in mouse_button_events.iter() {
        if let MouseButton::Right = mouse_button_event.button {
            for mut controlled in query.iter_mut() {
                controlled.rotating = match mouse_button_event.state {
                    ButtonState::Pressed => true,
                    ButtonState::Released => false,
                };
            }
        }
    }
}

fn rotate_controlled(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<(
        &Controlled,
        &mut Forward,
        &mut Transform,
        Option<&mut Camera>,
    )>,
    mut camera_query: Query<(&mut Forward, &mut Transform), Without<Controlled>>,
) {
    let delta: Vec2 = mouse_motion_events
        .iter()
        .fold(Vec2::ZERO, |delta, mouse_motion_event| {
            delta + mouse_motion_event.delta
        });

    for (controlled, mut forward, mut transform, camera) in query.iter_mut() {
        if controlled.rotating {
            let right = forward.value.cross(Vec3::Y);

            let rotation = Quat::from_rotation_y(0.005 * -delta.x);
            forward.value = rotation * forward.value;
            transform.rotate(rotation);

            if let Some(camera) = camera {
                let (mut camera_forward, mut camera_transform) =
                    camera_query.get_mut(camera.value).unwrap();

                let camera_rotation = rotation * Quat::from_axis_angle(right, 0.005 * -delta.y);

                camera_transform.rotate_around(transform.translation, camera_rotation);
                camera_forward.value = camera_rotation * camera_forward.value;
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AtmospherePlugin)
        .add_startup_system(setup)
        .add_system(move_entities)
        .add_system(scroll_zoom)
        .add_system(rotate_controlled)
        .add_system(handle_keys)
        .add_system(set_controlled_rotating)
        .run()
}
