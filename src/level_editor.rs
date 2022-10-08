use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion},
    pbr::wireframe::Wireframe,
    prelude::*,
};
use bevy_egui::EguiContext;
use bevy_rapier3d::prelude::{QueryFilter, RapierContext, Real};

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
        selected: Option<Entity>,
        hovered: Option<Entity>,
    },
}

#[derive(PartialEq, Eq)]
pub enum Mode {
    Camera { panning: bool },
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
struct Pan;

fn handle_left_click(
    mut level_editor: Option<ResMut<LevelEditor>>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
) {
    for event in mouse_button_events.iter() {
        if let MouseButton::Left = event.button {
            match event.state {
                bevy::input::ButtonState::Pressed => {
                    if let Some(level_editor) = level_editor.as_mut() {
                        if let LevelEditor::Loaded { mode, .. } = level_editor.as_mut() {
                            match mode {
                                Mode::Camera { panning } => {
                                    *panning = true;
                                }
                                Mode::Object => {}
                            }
                        }
                    }
                }
                bevy::input::ButtonState::Released => {
                    if let Some(level_editor) = level_editor.as_mut() {
                        if let LevelEditor::Loaded { mode, .. } = level_editor.as_mut() {
                            match mode {
                                Mode::Camera { panning } => {
                                    *panning = false;
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

fn handle_object_hover(
    mut commands: Commands,
    mut cursor_move_events: EventReader<CursorMoved>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    rapier_context: Res<RapierContext>,
    level_editor: Option<ResMut<LevelEditor>>,
) {
    if let Some(mut level_editor) = level_editor {
        if let LevelEditor::Loaded {
            mode: Mode::Object,
            hovered,
            ..
        } = level_editor.as_mut()
        {
            if let Some(event) = cursor_move_events.iter().last() {
                let cursor_position = event.position;
                let (camera, transform) = camera_query.iter().next().unwrap();

                let Vec2 {
                    x: logical_width,
                    y: logical_height,
                } = camera.logical_viewport_size().unwrap();

                let screen_position_ndc = Vec2 {
                    /*
                    0.0 -> -1.0
                    logical_width / 2 -> 0.0
                    logical_width -> 1.0
                    */
                    // x: (screen_position.x - logical_width / 2.0) / (logical_width / 2.0),
                    // x: 2.0 * (screen_position.x - logical_width / 2.0) / logical_width,
                    // x: (2.0 * screen_position.x - logical_width) / logical_width,
                    // x: (2.0 * screen_position.x) / logical_width - logical_width / logical_width,
                    x: 2.0 * cursor_position.x / logical_width - 1.0,
                    /*
                    0.0 -> -1.0
                    logical_height / 2 -> 0.0
                    logical_height -> 1.0
                    */
                    y: 2.0 * cursor_position.y / logical_height - 1.0,
                };

                let ndc_near = screen_position_ndc.extend(1.0);
                let ndc_far = screen_position_ndc.extend(std::f32::EPSILON);

                let ndc_to_world =
                    transform.compute_matrix() * camera.projection_matrix().inverse();

                let world_near = ndc_to_world.project_point3(ndc_near);
                let world_far = ndc_to_world.project_point3(ndc_far);

                let ray_origin = world_near;
                let ray_dir = world_far - world_near;

                let mut nearest_intersected_entity: Option<(Entity, Vec3)> = None;
                rapier_context.intersections_with_ray(
                    ray_origin,
                    ray_dir,
                    Real::MAX,
                    false,
                    QueryFilter::new(),
                    |entity, intersection| match nearest_intersected_entity.as_mut() {
                        Some((nearest_entity, nearest_position)) => {
                            if intersection.point.z > nearest_position.z {
                                *nearest_entity = entity;
                                *nearest_position = intersection.point;
                            }

                            true
                        }
                        None => {
                            nearest_intersected_entity = Some((entity, intersection.point));

                            true
                        }
                    },
                );

                match nearest_intersected_entity {
                    Some((entity, _position)) => {
                        debug!("hovered {:?}", entity);
                        if *hovered != Some(entity) {
                            if let Some(entity) = hovered.take() {
                                commands.entity(entity).remove::<Wireframe>();
                            }

                            *hovered = Some(entity);
                            commands.entity(entity).insert(Wireframe);
                        }
                    }
                    None => {
                        if let Some(entity) = hovered.take() {
                            commands.entity(entity).remove::<Wireframe>();
                        }
                    }
                }
            }
        }
    }
}

fn handle_drag(
    mut mouse_move_events: EventReader<MouseMotion>,
    level_editor: Option<Res<LevelEditor>>,
    mut query: Query<&mut Transform, (With<Pan>, Without<Camera>)>,
) {
    if let Some(level_editor) = level_editor {
        if let LevelEditor::Loaded {
            mode: Mode::Camera { panning: true },
            ..
        } = level_editor.as_ref()
        {
            for event in mouse_move_events.iter() {
                let delta = event.delta;
                // delta.x points to the right
                // delta.y points to the bottom

                for mut transform in &mut query {
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

                        if ui
                            .radio(matches!(mode, Mode::Camera { .. }), "camera")
                            .clicked()
                        {
                            *mode = Mode::Camera { panning: false };
                        };
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
                        .insert(Pan)
                        .insert(Rotate { rotating: false })
                        .id(),
                );

                commands.insert_resource(LevelEditor::Loaded {
                    path: path.clone(),
                    entities,
                    mode: Mode::Camera { panning: false },
                    selected: None,
                    hovered: None,
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
            .add_system(handle_object_hover)
            .add_system(handle_drag)
            .add_system(handle_drag_rotating)
            .add_system(create_ui);
    }
}
