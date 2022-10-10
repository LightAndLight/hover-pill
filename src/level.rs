pub mod wall;

use bevy::{
    asset::{AssetLoader, LoadedAsset},
    ecs::system::EntityCommands,
    prelude::*,
    reflect::TypeUuid,
};
use serde::{Deserialize, Serialize};

use crate::{
    fuel::FuelChanged,
    fuel_ball::FuelBallBundle,
    player,
    ui::{overlay, UI},
};

#[derive(Serialize, Deserialize, TypeUuid, Clone, Default)]
#[uuid = "a79e94e4-1d11-4581-82f8-fb82cbc67f43"]
pub struct Level {
    pub next_level: Option<String>,
    pub player_start: Vec3,
    pub initial_overlay: Option<Vec<String>>,
    pub structure: Vec<LevelItem>,
}

#[derive(Default)]
pub struct LevelAssetLoader;

impl AssetLoader for LevelAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let level = serde_json::from_slice::<Level>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(level));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum WallType {
    Neutral,
    Avoid,
    Goal,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum LevelItem {
    Wall {
        wall_type: WallType,
        position: Vec3,
        rotation: Quat,
        size: Vec2,
    },
    FuelBall {
        position: Vec3,
    },
    Light {
        position: Vec3,
        intensity: f32,
    },
}

pub enum CurrentLevel {
    None,
    Loading(Handle<Level>),
    Loaded {
        handle: Handle<Level>,
        next_level: Option<String>,
        player_start: Vec3,
        structure: Vec<Entity>,
        player: Entity,
    },
}

impl Default for CurrentLevel {
    fn default() -> Self {
        CurrentLevel::None
    }
}

pub fn clear_level(current_level: &CurrentLevel, commands: &mut Commands) {
    if let CurrentLevel::Loaded {
        structure, player, ..
    } = current_level
    {
        debug!("clearing level");

        commands.entity(*player).despawn_recursive();

        for entity in structure {
            commands.entity(*entity).despawn_recursive();
        }
    }
}

pub struct Entities {
    pub light: Entity,
    pub level_items: Vec<Entity>,
}

impl Entities {
    pub fn despawn(self, commands: &mut Commands) {
        commands.entity(self.light).despawn_recursive();

        for level_item in self.level_items {
            commands.entity(level_item).despawn_recursive();
        }
    }
}

pub fn create_world(
    commands: &mut Commands,
    level: &Level,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Entities {
    let light = {
        commands
            .spawn_bundle(DirectionalLightBundle {
                directional_light: DirectionalLight {
                    illuminance: 10000.0,
                    shadows_enabled: true,
                    shadow_projection: OrthographicProjection {
                        left: -50.0,
                        right: 50.0,
                        bottom: -50.0,
                        top: 50.0,
                        near: -50.0,
                        far: 50.0,
                        ..default()
                    },
                    ..default()
                },
                transform: Transform::from_rotation(Quat::from_rotation_x(
                    -std::f32::consts::PI / 3.5,
                )),
                ..default()
            })
            .id()
    };

    let level_items = level
        .structure
        .iter()
        .map(|item| match item {
            LevelItem::Wall {
                wall_type,
                position,
                rotation,
                size,
            } => match wall_type {
                WallType::Neutral => {
                    spawn_wall_neutral(commands, meshes, materials, *position, *rotation, *size)
                        .id()
                }
                WallType::Avoid => {
                    spawn_wall_avoid(commands, meshes, materials, *position, *rotation, *size).id()
                }
                WallType::Goal => {
                    spawn_wall_goal(commands, meshes, materials, *position, *rotation, *size).id()
                }
            },
            LevelItem::FuelBall { position } => commands
                .spawn_bundle(FuelBallBundle::new(meshes, materials, *position))
                .id(),
            LevelItem::Light {
                position,
                intensity,
            } => commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::UVSphere {
                        radius: 0.1,
                        sectors: 20,
                        stacks: 20,
                    })),
                    material: materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        emissive: Color::rgba_linear(100.0, 100.0, 100.0, 0.0),
                        ..default()
                    }),
                    transform: Transform::from_translation(*position),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(PointLightBundle {
                        point_light: PointLight {
                            intensity: *intensity,
                            radius: 0.1,
                            shadows_enabled: true,
                            ..default()
                        },
                        ..default()
                    });
                })
                .id(),
        })
        .collect();

    Entities { light, level_items }
}

pub fn spawn_wall_goal<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
    rotation: Quat,
    size: Vec2,
) -> EntityCommands<'w, 's, 'a> {
    let mut entity_commands = commands.spawn_bundle(wall::WallBundle::new(
        meshes,
        materials,
        Transform::identity()
            .with_translation(position)
            .with_rotation(rotation),
        size,
        Color::GREEN,
    ));

    entity_commands.insert(wall::WallType::Goal);

    entity_commands
}

pub fn spawn_wall_avoid<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
    rotation: Quat,
    size: Vec2,
) -> EntityCommands<'w, 's, 'a> {
    let mut entity_commands = commands.spawn_bundle(wall::WallBundle::new(
        meshes,
        materials,
        Transform::identity()
            .with_translation(position)
            .with_rotation(rotation),
        size,
        Color::RED,
    ));

    entity_commands.insert(wall::WallType::Avoid);

    entity_commands
}

pub fn spawn_wall_neutral<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
    rotation: Quat,
    size: Vec2,
) -> EntityCommands<'w, 's, 'a> {
    commands.spawn_bundle(wall::WallBundle::new(
        meshes,
        materials,
        Transform::identity()
            .with_translation(position)
            .with_rotation(rotation),
        size,
        Color::WHITE,
    ))
}

pub fn load_level(
    asset_server: &AssetServer,
    ui: &mut UI,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    fuel_changed: &mut EventWriter<FuelChanged>,
    handle: Handle<Level>,
    level: &Level,
) {
    debug!("started loading level");

    let entities: Vec<Entity> = {
        let Entities {
            light,
            mut level_items,
        } = create_world(commands, level, meshes, materials);
        level_items.push(light);
        level_items
    };

    let player = player::spawn_player(
        commands,
        meshes,
        materials,
        Transform::from_translation(level.player_start),
        Some(fuel_changed),
    );

    if let Some(overlay_text) = &level.initial_overlay {
        overlay::level_overview::display(asset_server, commands, ui, overlay_text);
    }

    debug!("finished loading level");

    commands.insert_resource(CurrentLevel::Loaded {
        handle,
        next_level: level.next_level.clone(),
        player_start: level.player_start,
        structure: entities,
        player,
    });
}

fn hotreload_level(
    mut asset_event: EventReader<AssetEvent<Level>>,
    asset_server: Res<AssetServer>,
    assets: Res<Assets<Level>>,
    mut ui: ResMut<UI>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut fuel_changed: EventWriter<FuelChanged>,
    current_level: Res<CurrentLevel>,
) {
    for event in asset_event.iter() {
        debug!("level asset event: {:?}", event);

        if let AssetEvent::Modified {
            handle: modified_handle,
        } = event
        {
            debug!("asset modified: {:?}", modified_handle);

            if let CurrentLevel::Loaded {
                handle: current_level_handle,
                ..
            } = current_level.as_ref()
            {
                if modified_handle == current_level_handle {
                    if let Some(level) = assets.get(current_level_handle) {
                        clear_level(&current_level, &mut commands);

                        load_level(
                            &asset_server,
                            &mut ui,
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            &mut fuel_changed,
                            current_level_handle.clone(),
                            level,
                        );
                    }
                }
            }
        }
    }
}

pub struct LoadEvent {
    pub path: String,
}

pub struct LevelPlugin;

fn handle_load_events(
    mut commands: Commands,
    mut input_events: EventReader<LoadEvent>,
    current_level: Res<CurrentLevel>,
    asset_server: Res<AssetServer>,
) {
    for event in input_events.iter() {
        clear_level(&current_level, &mut commands);
        let handle = asset_server.load(&event.path);
        commands.insert_resource(CurrentLevel::Loading(handle));
    }
}

fn finish_loading(
    asset_server: Res<AssetServer>,
    assets: Res<Assets<Level>>,
    mut ui: ResMut<UI>,
    current_level: Res<CurrentLevel>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut fuel_changed: EventWriter<FuelChanged>,
) {
    if let CurrentLevel::Loading(next_level_handle) = current_level.as_ref() {
        debug!(
            "loading {:?}",
            asset_server.get_handle_path(next_level_handle)
        );

        if let Some(level) = assets.get(next_level_handle) {
            load_level(
                &asset_server,
                &mut ui,
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut fuel_changed,
                next_level_handle.clone(),
                level,
            );
        }
    }
}

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<LevelAssetLoader>()
            .add_asset::<Level>()
            .add_event::<LoadEvent>()
            .init_resource::<CurrentLevel>()
            .add_system(handle_load_events)
            .add_system(finish_loading)
            .add_system(hotreload_level);
    }
}
