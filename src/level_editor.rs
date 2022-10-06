use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion},
    prelude::*,
};
use bevy_egui::EguiContext;

use crate::{
    camera::Zoom,
    level::{self, Level},
};

pub enum LevelEditor {
    Loading {
        path: String,
        handle: Handle<Level>,
    },
    Loaded {
        path: String,
        entities: Vec<Entity>,
        mode: Mode,
    },
}

#[derive(PartialEq, Eq)]
pub enum Mode {
    Camera,
    Object,
}

pub struct LoadEvent {
    pub path: String,
}

fn handle_load_event(
    mut commands: Commands,
    mut events: EventReader<LoadEvent>,
    asset_server: Res<AssetServer>,
    level_editor: Option<ResMut<LevelEditor>>,
) {
    if let Some(event) = events.iter().last() {
        let handle = asset_server.load(&event.path);

        if let Some(mut level_editor) = level_editor {
            let old_level_editor = std::mem::replace(
                level_editor.as_mut(),
                LevelEditor::Loading {
                    path: event.path.clone(),
                    handle,
                },
            );

            if let LevelEditor::Loaded { entities, .. } = old_level_editor {
                for entity in entities {
                    commands.entity(entity).despawn_recursive();
                }
            }
        } else {
            commands.insert_resource(LevelEditor::Loading {
                path: event.path.clone(),
                handle,
            });
        }
    }
}

#[derive(Component)]
struct Pan {
    panning: bool,
}

fn handle_left_click(
    level_editor: Option<Res<LevelEditor>>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut pan_query: Query<&mut Pan>,
) {
    for event in mouse_button_events.iter() {
        if let MouseButton::Left = event.button {
            match event.state {
                bevy::input::ButtonState::Pressed => {
                    if let Some(level_editor) = level_editor.as_ref() {
                        if let LevelEditor::Loaded { mode, .. } = level_editor.as_ref() {
                            match mode {
                                Mode::Camera => {
                                    for mut pan in &mut pan_query {
                                        pan.panning = true;
                                    }
                                }
                                Mode::Object => {}
                            }
                        }
                    }
                }
                bevy::input::ButtonState::Released => {
                    if let Some(level_editor) = level_editor.as_ref() {
                        if let LevelEditor::Loaded { mode, .. } = level_editor.as_ref() {
                            match mode {
                                Mode::Camera => {
                                    for mut pan in &mut pan_query {
                                        pan.panning = false;
                                    }
                                }
                                Mode::Object => {}
                            }
                        }
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

fn handle_drag(
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

fn create_ui(
    level_editor: Option<ResMut<LevelEditor>>,
    mut egui_context: ResMut<EguiContext>,
    mut load_event: EventWriter<LoadEvent>,
) {
    if let Some(mut level_editor) = level_editor {
        if let LevelEditor::Loaded { path, mode, .. } = level_editor.as_mut() {
            egui::Window::new("Level Editor")
                .fixed_pos((10.0, 10.0))
                .resizable(false)
                .show(egui_context.ctx_mut(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("level");

                        let _ = ui.text_edit_singleline(path);

                        let _ = ui.button("save");

                        if ui.button("load").clicked() {
                            load_event.send(LoadEvent { path: path.clone() })
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("movement");
                        let _ = ui.radio_value(mode, Mode::Camera, "camera");
                        let _ = ui.radio_value(mode, Mode::Object, "object");
                    });
                });
        }
    }
}

fn finish_loading(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<Assets<Level>>,
    level_editor: Option<Res<LevelEditor>>,
) {
    if let Some(level_editor) = level_editor {
        if let LevelEditor::Loading { path, handle } = level_editor.as_ref() {
            if let Some(level) = assets.get(handle) {
                let mut entities =
                    level::create_world(&mut commands, level, &mut meshes, &mut materials);

                entities.push(
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
                        .insert(Rotate { rotating: false })
                        .id(),
                );

                commands.insert_resource(LevelEditor::Loaded {
                    path: path.clone(),
                    entities,
                    mode: Mode::Camera,
                });
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
            .add_system(handle_drag)
            .add_system(handle_drag_rotating)
            .add_system(create_ui);
    }
}
