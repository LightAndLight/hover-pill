pub mod wall;

use bevy::{
    asset::{AssetLoader, LoadedAsset},
    ecs::system::EntityCommands,
    prelude::*,
    reflect::TypeUuid,
};
use serde::{Deserialize, Serialize};

use crate::{fuel::FuelChanged, fuel_ball::FuelBallBundle, player};

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

impl LevelItem {
    pub fn size(&self) -> Option<Vec2> {
        match self {
            LevelItem::Wall { size, .. } => Some(*size),
            LevelItem::FuelBall { .. } => None,
            LevelItem::Light { .. } => None,
        }
    }

    pub fn size_mut(&mut self) -> Option<&mut Vec2> {
        match self {
            LevelItem::Wall { size, .. } => Some(size),
            LevelItem::FuelBall { .. } => None,
            LevelItem::Light { .. } => None,
        }
    }

    pub fn rotation(&self) -> Option<Quat> {
        match self {
            LevelItem::Wall { rotation, .. } => Some(*rotation),
            LevelItem::FuelBall { .. } => None,
            LevelItem::Light { .. } => None,
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

/**
Postcondition: the order of [`Entities::level_items`] is in the same order as [`Level::structure`].
*/
pub fn create_world(
    commands: &mut Commands,
    level: &Level,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Entities {
    let light = {
        commands
            .spawn(DirectionalLightBundle {
                directional_light: DirectionalLight {
                    illuminance: 10000.0,
                    shadows_enabled: true,
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
                .spawn(FuelBallBundle::new(meshes, materials, *position))
                .id(),
            LevelItem::Light {
                position,
                intensity,
            } => commands
                .spawn(PbrBundle {
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
                    parent.spawn(PointLightBundle {
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
    let mut entity_commands = commands.spawn(wall::WallBundle::new(
        meshes,
        materials,
        Transform::IDENTITY
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
    let mut entity_commands = commands.spawn(wall::WallBundle::new(
        meshes,
        materials,
        Transform::IDENTITY
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
    commands.spawn(wall::WallBundle::new(
        meshes,
        materials,
        Transform::IDENTITY
            .with_translation(position)
            .with_rotation(rotation),
        size,
        Color::WHITE,
    ))
}

pub struct LoadedLevel {
    pub handle: Handle<Level>,
    pub next_level: Option<String>,
    pub player_start: Vec3,
    pub structure: Vec<Entity>,
    pub player: Entity,
}

pub fn create(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    fuel_changed: &mut EventWriter<FuelChanged>,
    handle: Handle<Level>,
    level: &Level,
) -> LoadedLevel {
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

    LoadedLevel {
        handle,
        next_level: level.next_level.clone(),
        player_start: level.player_start,
        structure: entities,
        player,
    }
}

pub fn clear(commands: &mut Commands, level: &LoadedLevel) {
    commands.entity(level.player).despawn_recursive();

    for entity in &level.structure {
        commands.entity(*entity).despawn_recursive();
    }
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<LevelAssetLoader>()
            .add_asset::<Level>();
    }
}
