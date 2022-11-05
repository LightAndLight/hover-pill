use std::{f32::consts::PI, fs::File, path::PathBuf};

use bevy::{
    asset::AssetServerSettings,
    input::mouse::{MouseButtonInput, MouseMotion},
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};
use bevy_egui::EguiContext;
use bevy_rapier3d::prelude::{Collider, QueryFilter, RapierContext, RayIntersection, Real};

use crate::{
    arrow,
    camera::Zoom,
    colored_wireframe::ColoredWireframe,
    cone::Cone,
    cylinder::Cylinder,
    level::{self, Level},
    player,
};

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
    Object { moving: Option<Moving> },
}

#[derive(PartialEq, Eq)]
pub enum SpawnMode {
    Avoid,
    Neutral,
    Goal,
}

pub struct Moving {
    intersection_point: Vec3,
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

#[derive(Component)]
enum Highlight {
    Hovered,
    Selected,
}

fn handle_left_click(
    mut commands: Commands,
    mut level_editor: Option<ResMut<LevelEditor>>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    windows: Res<Windows>,
    rapier_context: Res<RapierContext>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    highlight_query: Query<Entity, With<Highlight>>,
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
                                Mode::Object { moving } => {
                                    let cursor_position =
                                        windows.get_primary().unwrap().cursor_position().unwrap();
                                    let (camera, transform) = camera_query.iter().next().unwrap();

                                    let ray =
                                        screen_point_to_world(camera, transform, cursor_position);

                                    *moving = closest_intersection(
                                        rapier_context.as_ref(),
                                        transform.translation(),
                                        ray,
                                    )
                                    .map(
                                        |(entity, intersection)| {
                                            for entity in &highlight_query {
                                                trace!("removing Highlight and ColoredWireframe for {:?}", entity);
                                                commands
                                                    .entity(entity)
                                                    .remove::<Highlight>()
                                                    .remove::<ColoredWireframe>();
                                            }

                                            trace!("inserting Highlight and ColoredWireframe for {:?}", entity);
                                            commands
                                                .entity(entity)
                                                .insert(ColoredWireframe {
                                                    color: Color::GREEN,
                                                })
                                                .insert(Highlight::Selected).add_children(|parent| {
                                                    // +Z
                                                    arrow::spawn_child(parent, &mut meshes, &mut materials, 0.2, 2.0, Transform::from_translation(2.0 * Vec3::Z) * Transform::identity());
                                                    // -Z
                                                    arrow::spawn_child(parent, &mut meshes, &mut materials, 0.2, 2.0, Transform::from_translation(2.0 * -Vec3::Z) * Transform::from_rotation(Quat::from_rotation_x(PI)));
                                                    
                                                    // +Y
                                                    arrow::spawn_child(parent, &mut meshes, &mut materials, 0.2, 2.0, Transform::from_translation(2.0 * Vec3::Y) * Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0)));
                                                    // -Y
                                                    arrow::spawn_child(parent, &mut meshes, &mut materials, 0.2, 2.0, Transform::from_translation(2.0 * -Vec3::Y) * Transform::from_rotation(Quat::from_rotation_x(PI / 2.0)));
                                                   
                                                    // +X
                                                    arrow::spawn_child(parent, &mut meshes, &mut materials, 0.2, 2.0, Transform::from_translation(2.0 * Vec3::X) * Transform::from_rotation(Quat::from_rotation_y(PI / 2.0)));
                                                    // -X
                                                    arrow::spawn_child(parent, &mut meshes, &mut materials, 0.2, 2.0, Transform::from_translation(2.0 * -Vec3::X) * Transform::from_rotation(Quat::from_rotation_y(-PI / 2.0)));
                                                });

                                            Moving {
                                                intersection_point: intersection.point,
                                            }
                                        },
                                    );
                                }
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
                                Mode::Object { moving } => {
                                    *moving = None;
                                }
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

#[derive(Debug)]
struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    fn at(&self, t: f32) -> Vec3 {
        self.origin + t * self.direction
    }

    fn intersect_plane(&self, plane: Plane) -> Option<Vec3> {
        let t_denominator = plane.normal.dot(self.direction);

        let t = if t_denominator != 0.0 {
            let t = plane.normal.dot(plane.point - self.origin) / t_denominator;

            if t >= 0.0 {
                Some(t)
            } else {
                None
            }
        } else {
            None
        }?;

        Some(self.origin + t * self.direction)
    }
}

// TODO: use viewport_to_world when 0.9 is released.
fn screen_point_to_world(
    camera: &Camera,
    transform: &GlobalTransform,
    screen_position: Vec2,
) -> Ray {
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
        x: 2.0 * screen_position.x / logical_width - 1.0,

        /*
        0.0 -> -1.0
        logical_height / 2 -> 0.0
        logical_height -> 1.0
        */
        y: 2.0 * screen_position.y / logical_height - 1.0,
    };

    let ndc_near = screen_position_ndc.extend(1.0);
    let ndc_far = screen_position_ndc.extend(std::f32::EPSILON);

    let ndc_to_world = transform.compute_matrix() * camera.projection_matrix().inverse();

    let world_near = ndc_to_world.project_point3(ndc_near);
    let world_far = ndc_to_world.project_point3(ndc_far);

    Ray {
        origin: world_near,
        direction: (world_far - world_near).normalize(),
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
    level_editor: Option<ResMut<LevelEditor>>,
    highlight_query: Query<(Entity, &Highlight)>,
) {
    if let Some(mut level_editor) = level_editor {
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

                let ray = screen_point_to_world(camera, transform, cursor_position);

                if let Some((entity, _position)) =
                    closest_intersection(rapier_context.as_ref(), transform.translation(), ray)
                {
                    if !matches!(highlight_query.get(entity), Ok((_, Highlight::Selected))) {
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
}

struct Plane {
    point: Vec3,
    normal: Vec3,
}

fn handle_drag(
    mut mouse_move_events: EventReader<MouseMotion>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    level_editor: Option<ResMut<LevelEditor>>,
    mut query: Query<&mut Transform, (With<Pan>, Without<Camera>)>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut transform_query: Query<(&Highlight, &mut Transform), (Without<Pan>, Without<Camera>)>,
) {
    if let Some(mut level_editor) = level_editor {
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
                Mode::Object { moving } => {
                    if let Some(moving) = moving {
                        let (camera, camera_global_transform) = camera_query.iter().next().unwrap();

                        if let Some(cursor_moved) = cursor_moved_events.iter().last() {
                            let cursor_position = cursor_moved.position;

                            let center_ray = {
                                let screen_position_ndc = Vec2::ZERO;

                                let ndc_near = screen_position_ndc.extend(1.0);
                                let ndc_far = screen_position_ndc.extend(std::f32::EPSILON);

                                let ndc_to_world = camera_global_transform.compute_matrix()
                                    * camera.projection_matrix().inverse();

                                let world_near = ndc_to_world.project_point3(ndc_near);
                                let world_far = ndc_to_world.project_point3(ndc_far);

                                Ray {
                                    origin: world_near,
                                    direction: (world_far - world_near).normalize(),
                                }
                            };
                            let cursor_ray = screen_point_to_world(
                                camera,
                                camera_global_transform,
                                cursor_position,
                            );
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
                                        .dot(moving.intersection_point - cursor_ray.origin)
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
                                let translation = cursor_ray.at(end_t) - moving.intersection_point;

                                moving.intersection_point += translation;

                                for (highlight, mut transform) in &mut transform_query {
                                    if let Highlight::Selected = highlight {
                                        transform.translation += translation;
                                    }
                                }
                            }
                        }
                    }
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

fn handle_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keycodes: Res<Input<KeyCode>>,
    level_editor: Option<ResMut<LevelEditor>>,
    windows: Res<Windows>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    pan_query: Query<&Transform, With<Pan>>,
) {
    if let Some(mut level_editor) = level_editor {
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

                let cursor_position = windows.get_primary().unwrap().cursor_position().unwrap();

                let cursor_ray =
                    screen_point_to_world(camera, camera_global_transform, cursor_position);

                let position = cursor_ray
                    .intersect_plane(Plane {
                        normal: pan_transform.rotation * -Vec3::Z,
                        point: pan_transform.translation,
                    })
                    .unwrap();
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
                .id();

                entities.level_items.push(spawned_entity);
            }
        }
    }
}

fn handle_delete(
    mut commands: Commands,
    keycodes: Res<Input<KeyCode>>,
    level_editor: Option<ResMut<LevelEditor>>,
    mut params: ParamSet<(
        Query<(Entity, &Highlight, &LevelItem)>,
        Query<&mut LevelItem>,
    )>,
) {
    if let Some(mut level_editor) = level_editor {
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
                    if let Highlight::Selected = highlight {
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
    asset_server_settings: Res<AssetServerSettings>,
    level_editor: Option<ResMut<LevelEditor>>,
) {
    if let Some(level_editor) = level_editor {
        if let LevelEditor::Loaded { level, .. } = level_editor.as_ref() {
            for SaveEvent { path } in save_event.iter() {
                let file = File::create(
                    PathBuf::from(asset_server_settings.asset_folder.clone()).join(path),
                )
                .unwrap();
                serde_json::to_writer_pretty(file, level).unwrap();
            }
        }
    }
}

fn create_ui(
    level_editor: Option<ResMut<LevelEditor>>,
    mut egui_context: ResMut<EguiContext>,
    mut load_event: EventWriter<LoadEvent>,
    mut save_event: EventWriter<SaveEvent>,
    mut test_event: EventWriter<TestEvent>,
) {
    if let Some(mut level_editor) = level_editor {
        egui::Window::new("Level Editor")
            .fixed_pos((10.0, 10.0))
            .resizable(false)
            .show(egui_context.ctx_mut(), |ui| match level_editor.as_mut() {
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
                            *mode = Mode::Object { moving: None }
                        };
                    });

                    ui.horizontal(|ui| {
                        ui.label("spawn");

                        let _ = ui.radio_value(spawn_mode, SpawnMode::Avoid, "avoid");
                        let _ = ui.radio_value(spawn_mode, SpawnMode::Neutral, "neutral");
                        let _ = ui.radio_value(spawn_mode, SpawnMode::Goal, "goal");
                    });

                    ui.add_space(10.0);

                    ui.vertical_centered(|ui| {
                        if ui.button("test").clicked() {
                            test_event.send(TestEvent::Start);
                        }
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
}

fn handle_test_event(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut test_event: EventReader<TestEvent>,
    level_editor: Option<ResMut<LevelEditor>>,
) {
    if let Some(test_event) = test_event.iter().last() {
        if let Some(mut level_editor) = level_editor {
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
                            level::create_world(&mut commands, &level, &mut meshes, &mut materials);

                        for (index, entity) in entities.level_items.iter().enumerate() {
                            commands.entity(*entity).insert(LevelItem { index });
                        }

                        let camera = spawn_camera(&mut commands);

                        let player =
                            spawn_player(&mut commands, &mut meshes, &mut materials, &level);

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
}

#[derive(Component)]
struct Player;

fn move_player(
    level_editor: Option<ResMut<LevelEditor>>,
    query: Query<&Transform, Changed<Transform>>,
) {
    if let Some(mut level_editor) = level_editor {
        if let LevelEditor::Loaded { player, level, .. } = level_editor.as_mut() {
            if let Ok(transform) = query.get(*player) {
                level.player_start = transform.translation;
            }
        }
    }
}

#[derive(Component)]
struct LevelItem {
    index: usize,
}

fn move_level_item(
    level_editor: Option<ResMut<LevelEditor>>,
    query: Query<(&LevelItem, &Transform), Changed<Transform>>,
) {
    if let Some(mut level_editor) = level_editor {
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
                let entities =
                    level::create_world(&mut commands, level, &mut meshes, &mut materials);

                for (index, entity) in entities.level_items.iter().enumerate() {
                    commands.entity(*entity).insert(LevelItem { index });
                }

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
}

fn spawn_camera(commands: &mut Commands) -> Entity {
    commands
        .spawn_bundle(TransformBundle {
            local: Transform::identity().looking_at(Vec3::new(-5.0, -5.0, -5.0), Vec3::Y),
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
        .spawn_bundle(PbrBundle {
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

pub struct LevelEditorPlugin;

impl Plugin for LevelEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadEvent>()
            .add_event::<TestEvent>()
            .add_event::<SaveEvent>()
            .add_system(handle_load_event)
            .add_system(handle_test_event)
            .add_system(handle_save_event)
            .add_system(finish_loading)
            .add_system(handle_left_click)
            .add_system(handle_right_click)
            .add_system(handle_object_hover)
            .add_system(handle_drag)
            .add_system(handle_drag_rotating)
            .add_system(handle_spawn)
            .add_system(handle_delete.after(handle_object_hover))
            .add_system(move_player)
            .add_system(move_level_item)
            .add_system(create_ui);
    }
}
