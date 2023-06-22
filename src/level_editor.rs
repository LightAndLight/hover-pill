use std::{fs::File, path::PathBuf};

use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::{egui, EguiContexts};
use bevy_rapier3d::prelude::{Collider, QueryFilter, RapierContext, RayIntersection, Real};

use crate::{
    camera::Zoom,
    colored_wireframe::ColoredWireframe,
    config::Config,
    level::{self, Level},
    player,
    ui::{self, UI},
};

#[derive(Resource)]
pub enum LevelEditor {
    Empty,
    Loading {
        path: String,
        handle: Handle<Level>,
    },
    Loaded {
        path: String,
        level: Level,
        camera: Entity,
        entities: level::Entities,
        player: Entity,
        mode: Mode,
        spawn_mode: SpawnMode,
    },
    Testing {
        path: String,
        level: Level,
        player: Entity,
        entities: level::Entities,
    },
}

pub enum Mode {
    Camera { panning: bool },
    Object { action: ObjectAction },
}

#[derive(Default)]
pub enum ObjectAction {
    #[default]
    None,
    Moving {
        intersection_point: Vec3,
    },
}

#[derive(PartialEq, Eq)]
pub enum SpawnMode {
    Avoid,
    Neutral,
    Goal,
}

pub struct LoadEvent {
    pub path: String,
}

fn handle_load_event(
    mut commands: Commands,
    mut events: EventReader<LoadEvent>,
    asset_server: Res<AssetServer>,
    level_editor: Option<ResMut<LevelEditor>>,
    mut state: ResMut<NextState<State>>,
) {
    if let Some(event) = events.iter().last() {
        let handle = asset_server.load(&event.path);
        let new_level_editor = LevelEditor::Loading {
            path: event.path.clone(),
            handle,
        };

        match level_editor {
            Some(mut level_editor) => {
                let old_level_editor = std::mem::replace(level_editor.as_mut(), new_level_editor);
                if let LevelEditor::Loaded {
                    player,
                    camera,
                    entities,
                    ..
                } = old_level_editor
                {
                    commands.entity(player).despawn_recursive();
                    commands.entity(camera).despawn_recursive();
                    entities.despawn(&mut commands);
                }
            }
            None => commands.insert_resource(new_level_editor),
        }

        state.set(State::Enabled);
    }
}

#[derive(Component)]
struct Pan;

#[derive(Component)]
enum Highlight {
    Hovered,
    Selected,
}

fn handle_left_click(
    mut commands: Commands,
    mut level_editor: ResMut<LevelEditor>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    windows: Query<&Window, With<PrimaryWindow>>,
    rapier_context: Res<RapierContext>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    highlight_query: Query<(Entity, &Highlight)>,
) {
    for event in mouse_button_events.iter() {
        if let MouseButton::Left = event.button {
            match event.state {
                bevy::input::ButtonState::Pressed => {
                    if let LevelEditor::Loaded { mode, .. } = level_editor.as_mut() {
                        match mode {
                            Mode::Camera { panning } => {
                                *panning = true;
                            }
                            Mode::Object { action } => {
                                handle_left_click_object(
                                    &windows,
                                    &camera_query,
                                    action,
                                    &rapier_context,
                                    &highlight_query,
                                    &mut commands,
                                );
                            }
                        }
                    }
                }
                bevy::input::ButtonState::Released => {
                    if let LevelEditor::Loaded { mode, .. } = level_editor.as_mut() {
                        match mode {
                            Mode::Camera { panning } => {
                                *panning = false;
                            }
                            Mode::Object { action } => {
                                *action = ObjectAction::None;
                            }
                        }
                    }
                }
            }
        }
    }
}

// This is kind whacky: I had to factor out this function because `rustfmt` refused to format things properly.
fn handle_left_click_object(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera_query: &Query<(&Camera, &GlobalTransform)>,
    action: &mut ObjectAction,
    rapier_context: &Res<RapierContext>,
    highlight_query: &Query<(Entity, &Highlight)>,
    commands: &mut Commands,
) {
    let cursor_position = windows.get_single().unwrap().cursor_position().unwrap();
    let (camera, transform) = camera_query.iter().next().unwrap();

    let ray = camera
        .viewport_to_world(transform, cursor_position)
        .unwrap();

    let action_old = std::mem::take(action);
    *action = closest_intersection(rapier_context.as_ref(), transform.translation(), ray)
        .map(|(entity, intersection)| {
            for (entity, highlight) in highlight_query {
                if let Highlight::Selected = highlight {
                    commands
                        .entity(entity)
                        .remove::<Highlight>()
                        .remove::<ColoredWireframe>();
                }
            }

            trace!("inserting Highlight and ColoredWireframe for {:?}", entity);
            commands
                .entity(entity)
                .insert(ColoredWireframe {
                    color: Color::GREEN,
                })
                .insert(Highlight::Selected);

            ObjectAction::Moving {
                intersection_point: intersection.point,
            }
        })
        .unwrap_or(action_old);
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

fn closest_intersection(
    rapier_context: &RapierContext,
    source: Vec3,
    ray: Ray,
) -> Option<(Entity, RayIntersection)> {
    let mut closest: Option<(Entity, RayIntersection)> = None;

    rapier_context.intersections_with_ray(
        ray.origin,
        ray.direction,
        Real::MAX,
        false,
        QueryFilter::new(),
        |entity, intersection| match closest.as_mut() {
            Some((closest_entity, closest_intersection)) => {
                if (intersection.point - source).length()
                    < (closest_intersection.point - source).length()
                {
                    *closest_entity = entity;
                    *closest_intersection = intersection;
                }

                true
            }
            None => {
                closest = Some((entity, intersection));

                true
            }
        },
    );

    closest
}

fn handle_object_hover(
    mut commands: Commands,
    mut cursor_move_events: EventReader<CursorMoved>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    rapier_context: Res<RapierContext>,
    mut level_editor: ResMut<LevelEditor>,
    highlight_query: Query<(Entity, &Highlight)>,
) {
    if let LevelEditor::Loaded {
        mode: Mode::Object { .. },
        ..
    } = level_editor.as_mut()
    {
        if let Some(event) = cursor_move_events.iter().last() {
            for (entity, highlight) in &highlight_query {
                if let Highlight::Hovered = highlight {
                    trace!("removing Highlight and ColoredWireframe for {:?}", entity);
                    commands
                        .entity(entity)
                        .remove::<Highlight>()
                        .remove::<ColoredWireframe>();
                }
            }

            let cursor_position = event.position;
            let (camera, transform) = camera_query.iter().next().unwrap();

            let ray = camera
                .viewport_to_world(transform, cursor_position)
                .unwrap();

            if let Some((entity, _position)) =
                closest_intersection(rapier_context.as_ref(), transform.translation(), ray)
            {
                if !matches!(
                    highlight_query.get(entity),
                    Ok((_, Highlight::Selected { .. }))
                ) {
                    debug!("hovered {:?}", entity);

                    commands
                        .entity(entity)
                        .insert(ColoredWireframe {
                            color: Color::WHITE,
                        })
                        .insert(Highlight::Hovered);
                }
            }
        }
    }
}

fn handle_drag(
    mut mouse_move_events: EventReader<MouseMotion>,
    cursor_moved_events: EventReader<CursorMoved>,
    mut level_editor: ResMut<LevelEditor>,
    mut query: Query<&mut Transform, (With<Pan>, Without<Camera>)>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    highlighted_transform_query: Query<
        (&Highlight, &mut Transform),
        (Without<Pan>, Without<Camera>),
    >,
) {
    if let LevelEditor::Loaded { mode, .. } = level_editor.as_mut() {
        match mode {
            Mode::Camera { panning } => {
                if *panning {
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
            Mode::Object { action } => {
                handle_drag_object_action(
                    camera_query,
                    cursor_moved_events,
                    highlighted_transform_query,
                    action,
                );
            }
        }
    }
}

/// Compute a ray extending from the center of the camera, perpendicular to the viewing plane.
fn get_camera_ray(camera: &Camera, camera_global_transform: &GlobalTransform) -> Ray {
    let screen_position_ndc = Vec2::ZERO;

    let ndc_near = screen_position_ndc.extend(1.0);
    let ndc_far = screen_position_ndc.extend(std::f32::EPSILON);

    let ndc_to_world =
        camera_global_transform.compute_matrix() * camera.projection_matrix().inverse();

    let world_near = ndc_to_world.project_point3(ndc_near);
    let world_far = ndc_to_world.project_point3(ndc_far);

    Ray {
        origin: world_near,
        direction: (world_far - world_near).normalize(),
    }
}

fn handle_drag_object_action(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut highlighted_transform_query: Query<
        (&Highlight, &mut Transform),
        (Without<Pan>, Without<Camera>),
    >,
    action: &mut ObjectAction,
) {
    match action {
        ObjectAction::Moving { intersection_point } => {
            let (camera, camera_global_transform) = camera_query.iter().next().unwrap();

            if let Some(cursor_moved) = cursor_moved_events.iter().last() {
                let cursor_position = cursor_moved.position;

                let center_ray = get_camera_ray(camera, camera_global_transform);
                let cursor_ray = camera
                    .viewport_to_world(camera_global_transform, cursor_position)
                    .unwrap();
                debug!("cursor ray: {:?}", cursor_ray);

                /*
                A plane is defined of the set of points `(x, y, z)` that satisfy the equation
                `n_x(x - x_0) + n_y(y - y_0) + n_z(z - z_0) = 0` where `n` is a vector perpendicular to the
                plane and `(x_0, y_0, z_0)` is a predetermined point that lies on the plane.

                `center_ray`'s direction is perpendicular to the near/far planes, and the predetermined point
                is the point of intersection when the user started moving. This gives us a "movement plane".

                The target object's destination is the point where `cursor_ray` intersects the movement plane.

                We need to find some `t` and `(x, y, z)` such that `n_x(x - x_0) + n_y(y - y_0) + n_z(z - z_0) = 0`
                and `cursor_ray.origin + t * cursor_ray.direction = (x, y, z)`.

                The ray equation expands to:

                ```
                cursor_ray.origin.x + t * cursor_ray.direction.x = x
                cursor_ray.origin.y + t * cursor_ray.direction.y = y
                cursor_ray.origin.z + t * cursor_ray.direction.z = z
                ```

                and when substituted in to the plane equation:

                ```
                n_x((cursor_ray.origin.x + t * cursor_ray.direction.x) - x_0)
                + n_y((cursor_ray.origin.y + t * cursor_ray.direction.y) - y_0)
                + n_z((cursor_ray.origin.z + t * cursor_ray.direction.z) - z_0)
                = 0

                n_x * cursor_ray.origin.x + n_x * t * cursor_ray.direction.x - n_x * x_0
                + n_y * cursor_ray.origin.y + n_y * t * cursor_ray.direction.y - n_y * y_0
                + n_z * cursor_ray.origin.z + n_z * t * cursor_ray.direction.z - n_z * z_0
                = 0

                n_x * t * cursor_ray.direction.x
                + n_y * t * cursor_ray.direction.y
                + n_z * t * cursor_ray.direction.z
                =
                  -n_x * cursor_ray.origin.x
                  + n_x * x_0
                  - n_y * cursor_ray.origin.y
                  + n_y * y_0
                  - n_z * cursor_ray.origin.z
                  + n_z * z_0

                t * (n_x * cursor_ray.direction.x + n_y * cursor_ray.direction.y + n_z * cursor_ray.direction.z)
                =
                  -n_x * cursor_ray.origin.x
                  + n_x * x_0
                  - n_y * cursor_ray.origin.y
                  + n_y * y_0
                  - n_z * cursor_ray.origin.z
                  + n_z * z_0

                t = (
                  -n_x * cursor_ray.origin.x
                  + n_x * x_0
                  - n_y * cursor_ray.origin.y
                  + n_y * y_0
                  - n_z * cursor_ray.origin.z
                  + n_z * z_0
                ) /
                  (n_x * cursor_ray.direction.x + n_y * cursor_ray.direction.y + n_z * cursor_ray.direction.z)

                t = (
                  n_x * (-cursor_ray.origin.x + x_0)
                  + n_y * (-cursor_ray.origin.y + y_0)
                  + n_z * (-cursor_ray.origin.z + z_0)
                ) /
                  (n_x * cursor_ray.direction.x + n_y * cursor_ray.direction.y + n_z * cursor_ray.direction.z)

                t = (
                  n_x * (-cursor_ray.origin.x + x_0)
                  + n_y * (-cursor_ray.origin.y + y_0)
                  + n_z * (-cursor_ray.origin.z + z_0)
                ) /
                  n.dot(cursor_ray.direction)

                t =
                  n.dot((-cursor_ray.origin.x + x_0, -cursor_ray.origin.y + y_0, -cursor_ray.origin.z + z_0))
                  / n.dot(cursor_ray.direction)

                t = n.dot(-cursor_ray.origin + (x_0, y_0, z_0)) / n.dot(cursor_ray.direction)

                t = n.dot((x_0, y_0, z_0) - cursor_ray.origin) / n.dot(cursor_ray.direction)

                `t` is undefined when `n.dot(cursor_ray.direction) = 0` because in that case
                `cursor_ray.direction` lies parallel to the plane (perpendicular to `n`). If `cursor_ray.origin`
                lies on the plan then `t` has infinite solutions (the ray lies in the plain), otherwise
                `t` has no solutions.
                ```
                */

                let end_t = {
                    let t_denominator = center_ray.direction.dot(cursor_ray.direction);
                    if t_denominator != 0.0 {
                        let t = center_ray
                            .direction
                            .dot(*intersection_point - cursor_ray.origin)
                            / t_denominator;

                        if t >= 0.0 {
                            Some(t)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };

                if let Some(end_t) = end_t {
                    let translation = cursor_ray.get_point(end_t) - *intersection_point;

                    *intersection_point += translation;

                    for (highlight, mut transform) in &mut highlighted_transform_query {
                        if let Highlight::Selected = highlight {
                            transform.translation += translation;
                        }
                    }
                }
            }
        }
        ObjectAction::None => {}
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

fn handle_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keycodes: Res<Input<KeyCode>>,
    mut level_editor: ResMut<LevelEditor>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    pan_query: Query<&Transform, With<Pan>>,
) {
    if let LevelEditor::Loaded {
        spawn_mode,
        level,
        entities,
        ..
    } = level_editor.as_mut()
    {
        if keycodes.just_pressed(KeyCode::Space) {
            let (camera, camera_global_transform) = camera_query.iter().next().unwrap();
            let pan_transform = pan_query.iter().next().unwrap();

            let cursor_position = windows.get_single().unwrap().cursor_position().unwrap();

            let cursor_ray = camera
                .viewport_to_world(camera_global_transform, cursor_position)
                .unwrap();

            let position = cursor_ray.get_point(
                cursor_ray
                    .intersect_plane(pan_transform.translation, pan_transform.rotation * -Vec3::Z)
                    .unwrap(),
            );
            let rotation = Quat::default();
            let size = Vec2::new(5.0, 5.0);

            level.structure.push(level::LevelItem::Wall {
                wall_type: match spawn_mode {
                    SpawnMode::Avoid => level::WallType::Avoid,
                    SpawnMode::Neutral => level::WallType::Neutral,
                    SpawnMode::Goal => level::WallType::Goal,
                },
                position,
                rotation,
                size,
            });

            let index = entities.level_items.len();

            let spawned_entity = (match spawn_mode {
                SpawnMode::Avoid => level::spawn_wall_avoid(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    position,
                    rotation,
                    size,
                ),
                SpawnMode::Neutral => level::spawn_wall_neutral(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    position,
                    rotation,
                    size,
                ),
                SpawnMode::Goal => level::spawn_wall_goal(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    position,
                    rotation,
                    size,
                ),
            })
            .insert(LevelItem { index })
            .insert(Size::new(size.x, size.y))
            .id();

            entities.level_items.push(spawned_entity);
        }
    }
}

fn handle_delete(
    mut commands: Commands,
    keycodes: Res<Input<KeyCode>>,
    mut level_editor: ResMut<LevelEditor>,
    mut params: ParamSet<(
        Query<(Entity, &Highlight, &LevelItem)>,
        Query<&mut LevelItem>,
    )>,
) {
    if let LevelEditor::Loaded {
        mode: Mode::Object { .. },
        level,
        entities,
        ..
    } = level_editor.as_mut()
    {
        if keycodes.just_pressed(KeyCode::Delete) {
            debug!("delete pressed");

            let mut deleted_level_item_indices = Vec::new();

            let highlight_query = params.p0();
            for (entity, highlight, deleted_level_item) in &highlight_query {
                if let Highlight::Selected { .. } = highlight {
                    let deleted_level_item_index = deleted_level_item.index;
                    deleted_level_item_indices.push(deleted_level_item_index);

                    debug_assert!(
                        entities.level_items[deleted_level_item_index] == entity,
                        "wrong entity at index {:?}. expected {:?}, got {:?}",
                        deleted_level_item_index,
                        entity,
                        entities.level_items[deleted_level_item_index]
                    );

                    level.structure.remove(deleted_level_item.index);
                    entities.level_items.remove(deleted_level_item.index);

                    commands.entity(entity).despawn_recursive();
                }
            }

            /*
            This could be slow when there are many level objects - iterating
            through all level object for each deleted object.
            */
            let mut level_item_query = params.p1();
            for deleted_level_item_index in deleted_level_item_indices {
                for mut level_item in &mut level_item_query {
                    if level_item.index > deleted_level_item_index {
                        level_item.index -= 1;
                    }
                }
            }
        }
    }
}

enum TestEvent {
    Start,
    Stop,
}

struct SaveEvent {
    path: String,
}

fn handle_save_event(
    mut save_event: EventReader<SaveEvent>,
    config: Res<Config>,
    level_editor: ResMut<LevelEditor>,
) {
    if let LevelEditor::Loaded { level, .. } = level_editor.as_ref() {
        for SaveEvent { path } in save_event.iter() {
            let file = File::create(PathBuf::from(config.asset_dir.clone()).join(path)).unwrap();
            serde_json::to_writer_pretty(file, level).unwrap();
        }
    }
}

fn create_ui(
    mut state: ResMut<NextState<State>>,
    mut level_editor: ResMut<LevelEditor>,
    mut egui_contexts: EguiContexts,
    mut load_event: EventWriter<LoadEvent>,
    mut save_event: EventWriter<SaveEvent>,
    mut test_event: EventWriter<TestEvent>,
    mut item_parameters_query: Query<(&Highlight, &mut Transform, &mut Size)>,
) {
    egui::Window::new("Level Editor")
        .fixed_pos((10.0, 10.0))
        .resizable(false)
        .show(egui_contexts.ctx_mut(), |ui| match level_editor.as_mut() {
            LevelEditor::Loaded {
                path,
                mode,
                spawn_mode,
                ..
            } => {
                ui.horizontal(|ui| {
                    ui.label("level");

                    let _ = ui.text_edit_singleline(path);

                    if ui.button("save").clicked() {
                        save_event.send(SaveEvent { path: path.clone() });
                    };

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

                    if ui
                        .radio(matches!(mode, Mode::Object { .. }), "object")
                        .clicked()
                    {
                        *mode = Mode::Object {
                            action: ObjectAction::None,
                        }
                    };
                });

                ui.horizontal(|ui| {
                    ui.label("spawn");

                    let _ = ui.radio_value(spawn_mode, SpawnMode::Avoid, "avoid");
                    let _ = ui.radio_value(spawn_mode, SpawnMode::Neutral, "neutral");
                    let _ = ui.radio_value(spawn_mode, SpawnMode::Goal, "goal");
                });

                for (highlight, mut transform, mut size) in &mut item_parameters_query {
                    if let Highlight::Selected = highlight {
                        ui.add_space(10.0);

                        ui.heading("Selected Object");

                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label("x");
                                    let _ = ui.add(
                                        egui::DragValue::new(&mut transform.translation.x)
                                            .speed(1.0),
                                    );
                                });

                                ui.horizontal(|ui| {
                                    ui.label("y");
                                    let _ = ui.add(
                                        egui::DragValue::new(&mut transform.translation.y)
                                            .speed(1.0),
                                    );
                                });

                                ui.horizontal(|ui| {
                                    ui.label("z");
                                    let _ = ui.add(
                                        egui::DragValue::new(&mut transform.translation.z)
                                            .speed(1.0),
                                    );
                                });
                            });

                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label("length");
                                    let _ = ui.add(
                                        egui::DragValue::new(&mut size.y)
                                            .speed(1.0)
                                            .clamp_range(1.0..=f32::INFINITY),
                                    );
                                });

                                ui.horizontal(|ui| {
                                    ui.label("width");
                                    let _ = ui.add(
                                        egui::DragValue::new(&mut size.x)
                                            .speed(1.0)
                                            .clamp_range(1.0..=f32::INFINITY),
                                    );
                                });
                            });
                        });
                    }
                }

                ui.add_space(10.0);

                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("test").clicked() {
                            test_event.send(TestEvent::Start);
                        }

                        if ui.button("exit").clicked() {
                            state.set(State::Disabled);
                        }
                    })
                });
            }
            LevelEditor::Testing { .. } => {
                ui.vertical_centered(|ui| {
                    if ui.button("stop testing").clicked() {
                        test_event.send(TestEvent::Stop);
                    }
                });
            }
            _ => {}
        });
}

fn handle_test_event(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut test_event: EventReader<TestEvent>,
    mut level_editor: ResMut<LevelEditor>,
) {
    if let Some(test_event) = test_event.iter().last() {
        match test_event {
            TestEvent::Start => {
                let level_editor_content =
                    std::mem::replace(level_editor.as_mut(), LevelEditor::Empty);

                if let LevelEditor::Loaded {
                    path,
                    level,
                    player,
                    camera,
                    entities,
                    ..
                } = level_editor_content
                {
                    commands.entity(player).despawn_recursive();
                    commands.entity(camera).despawn_recursive();
                    entities.despawn(&mut commands);

                    let entities =
                        level::create_world(&mut commands, &level, &mut meshes, &mut materials);

                    let player = player::spawn_player(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        Transform::from_translation(level.player_start),
                        None,
                    );

                    *level_editor = LevelEditor::Testing {
                        path,
                        level,
                        player,
                        entities,
                    }
                }
            }
            TestEvent::Stop => {
                let level_editor_content =
                    std::mem::replace(level_editor.as_mut(), LevelEditor::Empty);

                if let LevelEditor::Testing {
                    path,
                    level,
                    player,
                    entities,
                } = level_editor_content
                {
                    commands.entity(player).despawn_recursive();
                    entities.despawn(&mut commands);

                    let entities =
                        create_editable_level(&mut commands, &mut meshes, &mut materials, &level);

                    let camera = spawn_camera(&mut commands);

                    let player = spawn_player(&mut commands, &mut meshes, &mut materials, &level);

                    *level_editor = LevelEditor::Loaded {
                        path,
                        level,
                        camera,
                        entities,
                        player,
                        mode: Mode::Camera { panning: false },
                        spawn_mode: SpawnMode::Neutral,
                    };
                }
            }
        }
    }
}

#[derive(Component)]
struct Player;

fn move_player(
    mut level_editor: ResMut<LevelEditor>,
    query: Query<&Transform, Changed<Transform>>,
) {
    if let LevelEditor::Loaded { player, level, .. } = level_editor.as_mut() {
        if let Ok(transform) = query.get(*player) {
            level.player_start = transform.translation;
        }
    }
}

#[derive(Component)]
struct LevelItem {
    index: usize,
}

fn move_level_item(
    mut level_editor: ResMut<LevelEditor>,
    query: Query<(&LevelItem, &Transform), Changed<Transform>>,
) {
    if let LevelEditor::Loaded { level, .. } = level_editor.as_mut() {
        for (level_item, transform) in &query {
            match &mut level.structure.get_mut(level_item.index) {
                Some(level::LevelItem::Wall { position, .. }) => {
                    *position = transform.translation;
                }
                Some(level::LevelItem::FuelBall { position }) => {
                    *position = transform.translation;
                }
                Some(level::LevelItem::Light { position, .. }) => {
                    *position = transform.translation;
                }
                None => {
                    debug!("no level item at index {}", level_item.index);
                }
            }
        }
    }
}

#[derive(Component)]
struct Size {
    x: f32,
    y: f32,
    x_unscaled: f32,
    y_unscaled: f32,
}

impl Size {
    fn new(x: f32, y: f32) -> Self {
        Size {
            x,
            y,
            x_unscaled: x,
            y_unscaled: y,
        }
    }
}

fn scale_level_item(
    mut level_editor: ResMut<LevelEditor>,
    mut query: Query<(&LevelItem, &mut Transform, &Size), Changed<Size>>,
) {
    if let LevelEditor::Loaded { level, .. } = level_editor.as_mut() {
        for (level_item, mut transform, size) in &mut query {
            if let Some(level_item_size) = level.structure[level_item.index].size_mut() {
                level_item_size.x = size.x;
                level_item_size.y = size.y;
            }

            transform.scale = Vec3 {
                x: size.x / size.x_unscaled,
                y: transform.scale.y,
                z: size.y / size.y_unscaled,
            };
        }
    }
}

fn finish_loading(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<Assets<Level>>,
    level_editor: Res<LevelEditor>,
) {
    if let LevelEditor::Loading { path, handle } = level_editor.as_ref() {
        if let Some(level) = assets.get(handle) {
            let entities = create_editable_level(&mut commands, &mut meshes, &mut materials, level);
            let camera = spawn_camera(&mut commands);
            let player = spawn_player(&mut commands, &mut meshes, &mut materials, level);

            commands.insert_resource(LevelEditor::Loaded {
                path: path.clone(),
                level: level.clone(),
                entities,
                player,
                camera,
                mode: Mode::Camera { panning: false },
                spawn_mode: SpawnMode::Neutral,
            });
        }
    }
}

fn create_editable_level(
    commands: &mut Commands<'_, '_>,
    meshes: &mut ResMut<'_, Assets<Mesh>>,
    materials: &mut ResMut<'_, Assets<StandardMaterial>>,
    level: &Level,
) -> level::Entities {
    let entities = level::create_world(commands, level, meshes, materials);

    for (index, (entity, level_item)) in entities
        .level_items
        .iter()
        .zip(level.structure.iter())
        .enumerate()
    {
        let mut entity_commands = commands.entity(*entity);
        entity_commands.insert(LevelItem { index });
        if let Some(size) = level_item.size() {
            entity_commands.insert(Size::new(size.x, size.y));
        }
    }
    entities
}

fn spawn_camera(commands: &mut Commands) -> Entity {
    commands
        .spawn(TransformBundle {
            local: Transform::IDENTITY.looking_at(Vec3::new(-5.0, -5.0, -5.0), Vec3::Y),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(Camera3dBundle {
                    /*
                    [note: implicit camera direction]

                    We assume the camera is always facing in the direction of -Z
                    and allow the parent transform to control orientation.
                    */
                    transform: Transform::from_xyz(0.0, 0.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
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
        .id()
}

fn spawn_player(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    level: &Level,
) -> Entity {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
                radius: player::CAPSULE_RADIUS,
                depth: player::CAPSULE_DEPTH,
                ..default()
            })),
            material: materials.add(player::CAPSULE_COLOR.into()),
            transform: Transform::from_translation(level.player_start),
            ..default()
        })
        .insert(Collider::capsule_y(
            player::CAPSULE_DEPTH / 2.0,
            player::CAPSULE_RADIUS,
        ))
        .id()
}

fn teardown(
    mut commands: Commands,
    mut level_editor: ResMut<LevelEditor>,
    asset_server: Res<AssetServer>,
    mut ui: ResMut<UI>,
) {
    let old_level_editor = std::mem::replace(level_editor.as_mut(), LevelEditor::Empty);

    if let LevelEditor::Loaded {
        player,
        camera,
        entities,
        ..
    } = old_level_editor
    {
        commands.entity(player).despawn_recursive();
        commands.entity(camera).despawn_recursive();
        entities.despawn(&mut commands);
    }

    ui::set(&mut commands, &mut ui, |commands| {
        ui::main_menu::create(&asset_server, commands)
    });
    ui::camera_on(&mut commands, &mut ui);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, States)]
pub enum State {
    #[default]
    Disabled,
    Enabled,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum LevelEditorSet {
    Ui,
    Transform,
    Interaction,
}

pub struct LevelEditorPlugin;

impl Plugin for LevelEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadEvent>()
            .add_event::<TestEvent>()
            .add_event::<SaveEvent>()
            .add_state::<State>()
            .add_system(handle_load_event)
            .add_system(teardown.in_schedule(OnExit(State::Enabled)))
            .configure_sets(
                (
                    LevelEditorSet::Ui,
                    LevelEditorSet::Transform,
                    LevelEditorSet::Interaction,
                )
                    .in_set(OnUpdate(State::Enabled)),
            )
            .configure_set(LevelEditorSet::Ui.before(LevelEditorSet::Transform))
            .configure_set(LevelEditorSet::Transform.before(LevelEditorSet::Interaction))
            .add_system(create_ui.in_set(LevelEditorSet::Ui))
            .add_systems(
                (move_player, move_level_item, scale_level_item).in_set(LevelEditorSet::Transform),
            )
            .add_systems(
                (
                    handle_test_event,
                    handle_save_event,
                    finish_loading,
                    handle_left_click,
                    handle_right_click,
                    handle_object_hover,
                    handle_drag,
                    handle_drag_rotating,
                    handle_spawn,
                    handle_delete.after(handle_object_hover),
                )
                    .in_set(LevelEditorSet::Interaction),
            );
    }
}
