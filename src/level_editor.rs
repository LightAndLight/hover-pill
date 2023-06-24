use std::{fs::File, path::PathBuf};

use bevy::{
    ecs::system::EntityCommands,
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
    level,
    load_level::{self, CurrentLevel, InCurrentLevel},
    main_menu, player,
    ui::{self, UI},
    wall::{WallBundle, WallType},
    GameState,
};

#[derive(Resource)]
pub struct LevelEditor {
    path: String,
    mode: Mode,
    spawn_mode: SpawnMode,
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

pub struct StartEvent {
    pub path: String,
}

fn handle_start_event(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut level_editor_next_state: ResMut<NextState<LevelEditorState>>,
    mut start_events: EventReader<StartEvent>,
    mut load_event: EventWriter<load_level::LoadEvent>,
) {
    if let Some(StartEvent { path }) = start_events.iter().last() {
        load_event.send(load_level::LoadEvent {
            path: "levels/tutorial_1.level.json".into(),
        });

        commands.insert_resource(LevelEditor {
            path: path.clone(),
            mode: Mode::Camera { panning: false },
            spawn_mode: SpawnMode::Neutral,
        });

        next_state.set(GameState::Editing);
        level_editor_next_state.set(LevelEditorState::Editing);
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
                bevy::input::ButtonState::Pressed => match &mut level_editor.mode {
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
                },
                bevy::input::ButtonState::Released => match &mut level_editor.mode {
                    Mode::Camera { panning } => {
                        *panning = false;
                    }
                    Mode::Object { action } => {
                        *action = ObjectAction::None;
                    }
                },
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
    highlight_query: Query<(Entity, &Highlight), With<InCurrentLevel>>,
) {
    if let LevelEditor {
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
    match &mut level_editor.mode {
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
    level_editor: ResMut<LevelEditor>,
    mut current_level: ResMut<CurrentLevel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    pan_query: Query<&Transform, With<Pan>>,
) {
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

        let index = current_level.level.structure.len();

        current_level.level.structure.push(level::LevelItem::Wall {
            wall_type: match level_editor.spawn_mode {
                SpawnMode::Avoid => WallType::Avoid,
                SpawnMode::Neutral => WallType::Neutral,
                SpawnMode::Goal => WallType::Goal,
            },
            position,
            rotation,
            size,
        });

        commands.spawn((
            InCurrentLevel::LevelItem(index),
            Size::new(size.x, size.y),
            Rotation::ZERO,
            match level_editor.spawn_mode {
                SpawnMode::Avoid => {
                    WallBundle::avoid(&mut meshes, &mut materials, position, rotation, size)
                }
                SpawnMode::Neutral => {
                    WallBundle::neutral(&mut meshes, &mut materials, position, rotation, size)
                }
                SpawnMode::Goal => {
                    WallBundle::goal(&mut meshes, &mut materials, position, rotation, size)
                }
            },
        ));
    }
}

fn handle_delete(
    mut commands: Commands,
    keycodes: Res<Input<KeyCode>>,
    mut current_level: ResMut<CurrentLevel>,
    mut params: ParamSet<(
        Query<(Entity, &Highlight, &InCurrentLevel)>,
        Query<&mut InCurrentLevel>,
    )>,
) {
    if keycodes.just_pressed(KeyCode::Delete) {
        debug!("delete pressed");

        let mut deleted_level_item_indices: Vec<usize> = Vec::new();

        let highlight_query = params.p0();
        for (entity, highlight, location) in &highlight_query {
            if let Highlight::Selected { .. } = highlight {
                match location {
                    InCurrentLevel::NoLocation => {}
                    InCurrentLevel::LevelItem(deleted_level_item_index) => {
                        deleted_level_item_indices.push(*deleted_level_item_index);

                        current_level
                            .level
                            .structure
                            .remove(*deleted_level_item_index);

                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
        }

        /*
        This could be slow when there are many level objects - iterating
        through all level objects for each deleted object.
        */
        let mut level_item_query = params.p1();
        for deleted_level_item_index in deleted_level_item_indices {
            for mut location in &mut level_item_query {
                match location.as_mut() {
                    InCurrentLevel::NoLocation => {}
                    InCurrentLevel::LevelItem(level_item_index) => {
                        if *level_item_index > deleted_level_item_index {
                            *level_item_index -= 1;
                        }
                    }
                }
            }
        }
    }
}

struct SaveEvent {
    path: String,
}

fn handle_save_event(
    mut save_event: EventReader<SaveEvent>,
    config: Res<Config>,
    current_level: Res<CurrentLevel>,
) {
    for SaveEvent { path } in save_event.iter() {
        let file = File::create(PathBuf::from(config.asset_dir.clone()).join(path)).unwrap();
        serde_json::to_writer_pretty(file, &current_level.level).unwrap();
    }
}

fn create_ui(
    level_editor_state: Res<State<LevelEditorState>>,
    mut level_editor: ResMut<LevelEditor>,
    mut egui_contexts: EguiContexts,
    mut load_level_event: EventWriter<load_level::LoadEvent>,
    mut save_event: EventWriter<SaveEvent>,
    mut test_event: EventWriter<TestEvent>,
    mut exit_event: EventWriter<ExitEvent>,
    mut item_parameters_query: Query<(&Highlight, &mut Transform, &mut Size, &mut Rotation)>,
) {
    egui::Window::new("Level Editor")
        .fixed_pos((10.0, 10.0))
        .resizable(false)
        .show(egui_contexts.ctx_mut(), |ui| match level_editor_state.0 {
            LevelEditorState::Editing => {
                ui.horizontal(|ui| {
                    ui.label("level");

                    let _ = ui.text_edit_singleline(&mut level_editor.path);

                    if ui.button("save").clicked() {
                        save_event.send(SaveEvent {
                            path: level_editor.path.clone(),
                        });
                    };

                    if ui.button("load").clicked() {
                        load_level_event.send(load_level::LoadEvent {
                            path: level_editor.path.clone(),
                        })
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("movement");

                    if ui
                        .radio(matches!(level_editor.mode, Mode::Camera { .. }), "camera")
                        .clicked()
                    {
                        level_editor.mode = Mode::Camera { panning: false };
                    };

                    if ui
                        .radio(matches!(level_editor.mode, Mode::Object { .. }), "object")
                        .clicked()
                    {
                        level_editor.mode = Mode::Object {
                            action: ObjectAction::None,
                        }
                    };
                });

                ui.horizontal(|ui| {
                    ui.label("spawn");

                    let _ = ui.radio_value(&mut level_editor.spawn_mode, SpawnMode::Avoid, "avoid");
                    let _ =
                        ui.radio_value(&mut level_editor.spawn_mode, SpawnMode::Neutral, "neutral");
                    let _ = ui.radio_value(&mut level_editor.spawn_mode, SpawnMode::Goal, "goal");
                });

                for (highlight, mut transform, mut size, mut rotation) in &mut item_parameters_query
                {
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

                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label("x");
                                    let _ = ui.add(egui::Slider::new(&mut rotation.x, 0.0..=360.0));
                                });

                                ui.horizontal(|ui| {
                                    ui.label("y");
                                    let _ = ui.add(egui::Slider::new(&mut rotation.y, 0.0..=360.0));
                                });

                                ui.horizontal(|ui| {
                                    ui.label("z");
                                    let _ = ui.add(egui::Slider::new(&mut rotation.z, 0.0..=360.0));
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
                            exit_event.send(ExitEvent);
                        }
                    })
                });
            }
            LevelEditorState::Testing { .. } => {
                ui.vertical_centered(|ui| {
                    if ui.button("stop testing").clicked() {
                        test_event.send(TestEvent::Stop);
                    }
                });
            }
            LevelEditorState::Disabled => {
                panic!("level editor is disabled");
            }
        });
}

enum TestEvent {
    Start,
    Stop,
}

fn handle_test_event(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut test_event: EventReader<TestEvent>,
    mut level_editor_state: ResMut<NextState<LevelEditorState>>,
    current_level: Res<CurrentLevel>,
    player_token_query: Query<Entity, (With<PlayerToken>, With<InCurrentLevel>)>,
    test_player_query: Query<Entity, (With<player::Player>, With<InCurrentLevel>)>,
) {
    if let Some(test_event) = test_event.iter().last() {
        trace!("handle_test_event");

        match test_event {
            TestEvent::Start => {
                for player_token in player_token_query.iter() {
                    commands.entity(player_token).despawn_recursive();
                }

                player::spawn_player(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    Transform::from_translation(current_level.level.player_start),
                    None,
                )
                .insert(InCurrentLevel::NoLocation);

                level_editor_state.set(LevelEditorState::Testing);
            }
            TestEvent::Stop => {
                for test_player in test_player_query.iter() {
                    commands.entity(test_player).despawn_recursive();
                }

                spawn_player_token(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    current_level.level.player_start,
                );

                level_editor_state.set(LevelEditorState::Editing);
            }
        }
    }
}

#[derive(Component)]
struct PlayerToken;

pub fn spawn_player_token<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
) -> EntityCommands<'w, 's, 'a> {
    commands.spawn((
        PlayerToken,
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
                radius: player::CAPSULE_RADIUS,
                depth: player::CAPSULE_DEPTH,
                ..default()
            })),
            material: materials.add(player::CAPSULE_COLOR.into()),
            transform: Transform::from_translation(position),
            ..default()
        },
        Collider::capsule_y(player::CAPSULE_DEPTH / 2.0, player::CAPSULE_RADIUS),
        InCurrentLevel::NoLocation,
    ))
}

fn move_player_start(
    mut current_level: ResMut<CurrentLevel>,
    query: Query<&Transform, (Changed<Transform>, With<PlayerToken>, With<InCurrentLevel>)>,
) {
    for transform in query.iter() {
        current_level.level.player_start = transform.translation;
    }
}

fn move_level_item(
    mut current_level: ResMut<CurrentLevel>,
    query: Query<(&InCurrentLevel, &Transform), Changed<Transform>>,
) {
    for (location, transform) in &query {
        match location {
            InCurrentLevel::NoLocation => {}
            InCurrentLevel::LevelItem(level_item_index) => {
                match current_level.level.structure.get_mut(*level_item_index) {
                    Some(level_item) => {
                        *level_item.position_mut() = transform.translation;
                    }
                    None => {
                        debug!("no level item at index {}", level_item_index);
                    }
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
    mut current_level: ResMut<CurrentLevel>,
    mut query: Query<(&InCurrentLevel, &mut Transform, &Size), Changed<Size>>,
) {
    for (location, mut transform, size) in &mut query {
        match location {
            InCurrentLevel::NoLocation => {}
            InCurrentLevel::LevelItem(level_item_index) => {
                if let Some(level_item_size) =
                    current_level.level.structure[*level_item_index].size_mut()
                {
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
}

#[derive(Component)]
struct Rotation {
    x: f32,
    y: f32,
    z: f32,
}

impl Rotation {
    const ZERO: Self = Rotation {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    fn from_quat(quat: &Quat) -> Self {
        use std::f32::consts::TAU;

        let (x, y, z) = quat.to_euler(EulerRot::XYZ);
        Self {
            x: 360.0 * x / TAU,
            y: 360.0 * y / TAU,
            z: 360.0 * z / TAU,
        }
    }

    fn to_quat(&self) -> Quat {
        use std::f32::consts::TAU;

        Quat::from_euler(
            EulerRot::XYZ,
            TAU * self.x / 360.0,
            TAU * self.y / 360.0,
            TAU * self.z / 360.0,
        )
    }
}

fn rotate_level_item(mut query: Query<(&mut Transform, &Rotation), Changed<Rotation>>) {
    for (mut transform, rotation) in &mut query {
        transform.rotation = rotation.to_quat();
    }
}

fn annotate_level_items(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    level_item_query: Query<(Entity, &InCurrentLevel)>,
) {
    for (entity, location) in level_item_query.iter() {
        match location {
            InCurrentLevel::NoLocation => {}
            InCurrentLevel::LevelItem(level_item_index) => {
                let level_item = &current_level.level.structure[*level_item_index];
                let mut entity_commands = commands.entity(entity);

                if let Some(size) = level_item.size() {
                    entity_commands.insert(Size::new(size.x, size.y));
                }

                if let Some(rotation) = level_item.rotation() {
                    entity_commands.insert(Rotation::from_quat(&rotation));
                }
            }
        }
    }
}

#[derive(Resource)]
struct LevelEditorCamera {
    entity: Entity,
    camera_entity: Entity,
}

fn setup_level_editor_camera(mut commands: Commands) {
    let entity = commands
        .spawn(TransformBundle {
            local: Transform::IDENTITY.looking_at(Vec3::new(-5.0, -5.0, -5.0), Vec3::Y),
            ..Default::default()
        })
        .insert(Pan)
        .insert(Rotate { rotating: false })
        .id();

    let camera_entity = commands
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
        .insert(Zoom)
        .set_parent(entity)
        .id();

    commands.insert_resource(LevelEditorCamera {
        entity,
        camera_entity,
    });
}

fn enable_level_editor_camera(
    level_editor_camera: Res<LevelEditorCamera>,
    mut query: Query<&mut Camera>,
) {
    trace!("enable_level_editor_camera");

    query
        .get_mut(level_editor_camera.camera_entity)
        .unwrap()
        .is_active = true;
}

fn disable_level_editor_camera(
    level_editor_camera: Res<LevelEditorCamera>,
    mut query: Query<&mut Camera>,
) {
    trace!("disable_level_editor_camera");

    query
        .get_mut(level_editor_camera.camera_entity)
        .unwrap()
        .is_active = false;
}

fn remove_level_editor_camera(mut commands: Commands, level_editor_camera: Res<LevelEditorCamera>) {
    commands
        .entity(level_editor_camera.entity)
        .despawn_recursive();
}

struct ExitEvent;

fn handle_exit_event(
    mut commands: Commands,
    mut level_editor_next_state: ResMut<NextState<LevelEditorState>>,
    asset_server: Res<AssetServer>,
    mut ui: ResMut<UI>,
    mut exit_events: EventReader<ExitEvent>,
    in_current_level_query: Query<Entity, With<InCurrentLevel>>,
) {
    if let Some(ExitEvent) = exit_events.iter().last() {
        {
            let commands: &mut Commands = &mut commands;
            let query = &in_current_level_query;
            for entity in query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        };
        commands.remove_resource::<CurrentLevel>();

        commands.remove_resource::<LevelEditor>();
        level_editor_next_state.set(LevelEditorState::Disabled);

        ui::set(&mut commands, &mut ui, |commands| {
            main_menu::create(&asset_server, commands)
        });
        ui::camera_on(&mut commands, &mut ui);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, States)]
pub enum LevelEditorState {
    #[default]
    Disabled,
    Editing,
    Testing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, SystemSet)]
pub enum LevelEditorSet {
    Base,
    Transform,
    Interact,
}

pub struct LevelEditorPlugin;

impl Plugin for LevelEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartEvent>()
            .add_event::<ExitEvent>()
            .add_event::<TestEvent>()
            .add_event::<SaveEvent>()
            .add_state::<LevelEditorState>()
            .add_system(handle_start_event.in_set(OnUpdate(LevelEditorState::Disabled)));

        app.add_system(setup_level_editor_camera.in_schedule(OnExit(LevelEditorState::Disabled)))
            .add_system(
                remove_level_editor_camera
                    .in_schedule(OnEnter(LevelEditorState::Disabled))
                    .run_if(resource_exists::<LevelEditorCamera>()),
            )
            .add_system(
                enable_level_editor_camera
                    .after(setup_level_editor_camera)
                    .in_schedule(OnEnter(LevelEditorState::Editing))
                    .run_if(resource_exists::<LevelEditorCamera>()),
            )
            .add_system(
                disable_level_editor_camera
                    .before(remove_level_editor_camera)
                    .in_schedule(OnExit(LevelEditorState::Editing))
                    .run_if(resource_exists::<LevelEditorCamera>()),
            );

        app.add_system(create_ui.run_if(resource_exists::<LevelEditor>()))
            .add_systems((
                handle_test_event
                    .run_if(not(in_state(LevelEditorState::Disabled)))
                    .run_if(resource_exists::<CurrentLevel>()),
                annotate_level_items.run_if(
                    in_state(GameState::Editing)
                        .or_else(in_state(GameState::Testing))
                        .and_then(resource_added::<CurrentLevel>()),
                ),
            ))
            .configure_set(
                LevelEditorSet::Transform
                    .run_if(in_state(LevelEditorState::Editing))
                    .run_if(resource_exists::<CurrentLevel>()),
            )
            .add_systems(
                (
                    move_player_start,
                    move_level_item,
                    scale_level_item,
                    rotate_level_item,
                )
                    .in_set(LevelEditorSet::Transform),
            );

        app.configure_set(
            LevelEditorSet::Interact
                .run_if(in_state(LevelEditorState::Editing))
                .run_if(resource_exists::<CurrentLevel>()),
        )
        .add_systems(
            (
                handle_save_event,
                handle_exit_event,
                handle_spawn,
                handle_delete.after(handle_object_hover),
            )
                .in_set(LevelEditorSet::Interact),
        );

        app.add_systems(
            (
                // interactions
                handle_left_click,
                handle_right_click,
                handle_object_hover,
                handle_drag,
                handle_drag_rotating,
            )
                .in_set(OnUpdate(LevelEditorState::Editing)),
        );
    }
}
