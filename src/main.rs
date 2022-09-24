use std::f32::consts::PI;

use bevy::{
    input::{
        mouse::{MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel},
        ButtonState,
    },
    prelude::*,
    winit::WinitSettings,
};
use bevy_atmosphere::prelude::{AtmosphereCamera, AtmospherePlugin};
use bevy_rapier3d::{prelude::*, render::RapierDebugRenderPlugin};

#[derive(Component)]
struct Speed {
    value: f32,
}

#[derive(Component)]
struct JumpImpulse {
    value: Vec3,
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
    hovering: bool,
}

#[derive(Component)]
struct Camera;

#[derive(Component)]
struct Zoom;

#[derive(Component)]
struct Fuel {
    value: f32,
}

#[derive(Component)]
struct FuelBar;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    rapier_config: Res<RapierConfiguration>,
) {
    let capsule_radius = 0.5;
    let capsule_depth = 2.0 * capsule_radius;

    let initial_jump_impulse = 5. * Vec3::Y;

    info!("gravity: {}", rapier_config.gravity);

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
        // .insert(Camera { value: camera })
        .with_children(|parent| {
            let camera_looking_at = Vec3::new(0.0, 1.0, 0.0);

            let transform =
                Transform::from_xyz(-5.0, 4.0, 0.0).looking_at(camera_looking_at, Vec3::Y);

            parent
                .spawn_bundle(Camera3dBundle {
                    projection: PerspectiveProjection {
                        fov: (60.0 / 360.0) * 2.0 * PI,
                        ..default()
                    }
                    .into(),
                    transform,
                    ..default()
                })
                .insert(Camera)
                .insert(AtmosphereCamera(None))
                .insert(Zoom);
        });

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

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.25,
                sectors: 4,
                stacks: 3,
            })),
            material: materials.add(Color::rgb(0.4, 0.4, 1.).into()),
            transform: Transform::from_translation(Vec3::new(2., 2., 2.))
                .with_rotation(Quat::from_rotation_x(PI / 2.)),
            ..default()
        })
        .insert(Collider::ball(0.25))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Sensor)
        .insert(RefuelBall { amount: 0.20 });

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

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(300.0), Val::Px(30.0)),
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        position_type: PositionType::Absolute,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    color: Color::BLACK.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                ..default()
                            },
                            color: Color::rgb(0.4, 0.4, 1.0).into(),
                            ..default()
                        })
                        .insert(FuelBar);
                });

            parent.spawn_bundle(TextBundle::from_section(
                "fuel",
                TextStyle {
                    font: asset_server.load("fonts/DejaVuSansMono.ttf"),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            ));
        });
}

fn move_entities(
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

fn handle_keys(
    keys: Res<Input<KeyCode>>,
    // mut query: Query<(&mut Controlled, &JumpImpulse, &mut ExternalImpulse)>,
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

fn start_hover(
    fuel: &Fuel,
    controlled: &mut Controlled,
    external_impulse: &mut ExternalImpulse,
    jump_impulse: &JumpImpulse,
    external_force: &mut ExternalForce,
) {
    debug!("start_hover: fuel: {:?}", fuel.value);
    if fuel.value > 0.0 {
        debug!("start_hover: hovering");
        controlled.hovering = true;

        external_impulse.impulse = jump_impulse.value;
        external_force.force = 12. * Vec3::Y;
    }
}

fn end_hover(
    controlled: &mut Controlled,
    external_impulse: &mut ExternalImpulse,
    external_force: &mut ExternalForce,
) {
    controlled.hovering = false;
    external_impulse.impulse = Vec3::ZERO;
    external_force.force = Vec3::ZERO;
}

struct FuelChanged {
    new_value: f32,
}

fn use_fuel_to_hover(
    time: Res<Time>,
    mut query: Query<(
        &mut Controlled,
        &mut Fuel,
        &mut ExternalImpulse,
        &mut ExternalForce,
    )>,
    mut fuel_changed: EventWriter<FuelChanged>,
) {
    for (mut controlled, mut fuel, mut external_impulse, mut external_force) in &mut query {
        if controlled.hovering {
            subtract_fuel(&mut fuel, time.delta_seconds() * 0.1, &mut fuel_changed);

            if fuel.value <= 0. {
                end_hover(&mut controlled, &mut external_impulse, &mut external_force)
            }
        }
    }
}

fn subtract_fuel(fuel: &mut Mut<Fuel>, amount: f32, fuel_changed: &mut EventWriter<FuelChanged>) {
    fuel.value = (fuel.value - amount).clamp(0.0, 1.0);
    fuel_changed.send(FuelChanged {
        new_value: fuel.value,
    });
}

fn add_fuel(fuel: &mut Mut<Fuel>, amount: f32, fuel_changed: &mut EventWriter<FuelChanged>) {
    fuel.value = (fuel.value + amount).clamp(0.0, 1.0);
    fuel_changed.send(FuelChanged {
        new_value: fuel.value,
    });
}

fn update_fuel_bar(
    mut fuel_changed: EventReader<FuelChanged>,
    mut query: Query<&mut Style, With<FuelBar>>,
) {
    for fuel_changed in fuel_changed.iter() {
        for mut style in &mut query {
            style.size.width = Val::Percent(fuel_changed.new_value * 100.0);
        }
    }
}

fn scroll_zoom(
    mut scroll_events: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<Zoom>>,
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

    for mut transform in query.iter_mut() {
        let translation = transform.translation;
        transform.translation += 0.08 * scroll_amount * -translation;
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
                    let camera_rotation = Quat::from_axis_angle(Vec3::Z, 0.005 * -delta.y);

                    camera_transform.rotate_around(Vec3::ZERO, camera_rotation);
                }
            }
        }
    }
}

fn display_collision_events(mut collision_events: EventReader<CollisionEvent>) {
    for collision_event in collision_events.iter() {
        debug!("collision event: {:?}", collision_event);
    }
}

#[derive(Component)]
struct RefuelBall {
    amount: f32,
}

fn consume_refuel_balls(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut fuel_query: Query<&mut Fuel, Without<RefuelBall>>,
    ball_query: Query<&RefuelBall>,
    mut fuel_changed: EventWriter<FuelChanged>,
) {
    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            let (fuel_entity, ball_entity) = if fuel_query.contains(*entity1) {
                (*entity1, *entity2)
            } else {
                (*entity2, *entity1)
            };

            if let (Ok(mut fuel), Ok(refuel_ball)) =
                (fuel_query.get_mut(fuel_entity), ball_query.get(ball_entity))
            {
                add_fuel(&mut fuel, refuel_ball.amount, &mut fuel_changed);
                commands.entity(ball_entity).despawn();
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AtmospherePlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .insert_resource(WinitSettings::game())
        .add_event::<FuelChanged>()
        .add_startup_system(setup)
        .add_system(move_entities)
        .add_system(scroll_zoom)
        .add_system(rotate_controlled)
        .add_system(handle_keys)
        .add_system(set_controlled_rotating)
        .add_system(use_fuel_to_hover)
        .add_system(update_fuel_bar)
        .add_system(display_collision_events)
        .add_system(consume_refuel_balls)
        .run()
}
