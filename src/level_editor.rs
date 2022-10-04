use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion},
    prelude::*,
};

use crate::{
    camera::Zoom,
    level::{self, Level},
    ui::{self, UI},
};

pub struct LoadEvent {
    pub path: String,
}

enum CurrentLevel {
    Loading { path: String, handle: Handle<Level> },
    Loaded { path: String, world: Vec<Entity> },
}

fn handle_load_event(
    mut commands: Commands,
    mut events: EventReader<LoadEvent>,
    asset_server: Res<AssetServer>,
) {
    for event in events.iter() {
        let handle = asset_server.load(&event.path);
        commands.insert_resource(CurrentLevel::Loading {
            path: event.path.clone(),
            handle,
        });
    }
}

#[derive(Component)]
struct Pan {
    panning: bool,
}

fn handle_left_click(
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut pan_query: Query<&mut Pan>,
) {
    for event in mouse_button_events.iter() {
        if let MouseButton::Left = event.button {
            match event.state {
                bevy::input::ButtonState::Pressed => {
                    for mut pan in &mut pan_query {
                        pan.panning = true;
                    }
                }
                bevy::input::ButtonState::Released => {
                    for mut pan in &mut pan_query {
                        pan.panning = false;
                    }
                }
            }
        }
    }
}

#[derive(Component)]
struct Rotate {
    rotating: bool,
}

fn handle_right_click(
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut rotate_query: Query<&mut Rotate>,
) {
    for event in mouse_button_events.iter() {
        if let MouseButton::Right = event.button {
            match event.state {
                bevy::input::ButtonState::Pressed => {
                    for mut rotate in &mut rotate_query {
                        rotate.rotating = true;
                    }
                }
                bevy::input::ButtonState::Released => {
                    for mut rotate in &mut rotate_query {
                        rotate.rotating = false;
                    }
                }
            }
        }
    }
}

fn handle_drag_panning(
    mut mouse_move_events: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &Pan), Without<Camera>>,
) {
    for event in mouse_move_events.iter() {
        let delta = event.delta;
        // delta.x points to the right
        // delta.y points to the bottom

        for (mut transform, pan) in &mut query {
            if pan.panning {
                // Assume the camera is always looking in the -Z direction (into the screen)
                // See [note: implicit camera direction]
                let look_direction = transform.rotation * -Vec3::Z;

                let left = look_direction.cross(-Vec3::Y).normalize();
                let up = Vec3::Y;
                let scale = 0.05;
                transform.translation += scale * (delta.x * left + delta.y * up);
            }
        }
    }
}

fn handle_drag_rotating(
    mut mouse_move_events: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &Rotate)>,
) {
    for event in mouse_move_events.iter() {
        let delta = event.delta;
        // delta.x points to the right
        // delta.y points to the bottom

        for (mut transform, rotate) in &mut query {
            if rotate.rotating {
                let scale = 0.005;
                transform.rotate_local_x(scale * -delta.y);
                transform.rotate_y(scale * -delta.x);
            }
        }
    }
}

fn create_ui(asset_server: &AssetServer, commands: &mut Commands, ui: &mut UI, path: &str) {
    ui::set(commands, ui, |commands| {
        commands
            .spawn_bundle(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(10.0),
                        left: Val::Px(10.0),
                        ..Default::default()
                    },
                    border: UiRect::all(Val::Px(1.0)),
                    ..Default::default()
                },
                color: Color::WHITE.into(),
                ..Default::default()
            })
            .with_children(|parent| {
                parent
                    .spawn_bundle(NodeBundle {
                        color: Color::BLACK.into(),
                        style: Style {
                            padding: UiRect::all(Val::Px(5.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle::from_section(
                            path,
                            TextStyle {
                                font: asset_server.load("fonts/DejaVuSansMono.ttf"),
                                font_size: 30.0,
                                color: Color::WHITE,
                            },
                        ));
                    });
            })
            .id()
    });
}

fn finish_loading(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut ui: ResMut<UI>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<Assets<Level>>,
    current_level: Option<Res<CurrentLevel>>,
) {
    if let Some(current_level) = current_level {
        if let CurrentLevel::Loading { path, handle } = current_level.as_ref() {
            if let Some(level) = assets.get(handle) {
                let world = level::create_world(&mut commands, level, &mut meshes, &mut materials);
                commands.insert_resource(CurrentLevel::Loaded {
                    path: path.clone(),
                    world,
                });

                commands
                    .spawn_bundle(TransformBundle {
                        local: Transform::identity()
                            .looking_at(Vec3::new(-5.0, -5.0, -5.0), Vec3::Y),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(Camera3dBundle {
                                /*
                                [note: implicit camera direction]

                                We assume the camera is always facing in the direction of -Z
                                and allow the parent transform to control orientation.
                                */
                                transform: Transform::from_xyz(0.0, 0.0, 40.0)
                                    .looking_at(Vec3::ZERO, Vec3::Y),
                                projection: PerspectiveProjection {
                                    fov: (60.0 / 360.0) * 2.0 * std::f32::consts::PI,
                                    ..default()
                                }
                                .into(),
                                ..Default::default()
                            })
                            .insert(Zoom);
                    })
                    .insert(Pan { panning: false })
                    .insert(Rotate { rotating: false });

                create_ui(&asset_server, &mut commands, &mut ui, path);
            }
        }
    }
}

pub struct LevelEditorPlugin;

impl Plugin for LevelEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadEvent>()
            .add_system(handle_load_event)
            .add_system(finish_loading)
            .add_system(handle_left_click)
            .add_system(handle_right_click)
            .add_system(handle_drag_panning)
            .add_system(handle_drag_rotating);
    }
}
