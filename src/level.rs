pub mod asset;

use bevy::{prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};

use crate::{
    fuel::FuelChanged,
    fuel_ball::FuelBallBundle,
    player,
    wall::{WallBundle, WallType},
};

#[derive(Serialize, Deserialize, TypeUuid, Clone, Default)]
#[uuid = "a79e94e4-1d11-4581-82f8-fb82cbc67f43"]
pub struct Level {
    pub next_level: Option<String>,
    pub player_start: Vec3,
    pub initial_overlay: Option<Vec<String>>,
    pub structure: Vec<LevelItem>,
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

    pub fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Entity {
        match self {
            LevelItem::Wall {
                wall_type,
                position,
                rotation,
                size,
            } => match wall_type {
                WallType::Neutral => commands
                    .spawn(WallBundle::neutral(
                        meshes, materials, *position, *rotation, *size,
                    ))
                    .id(),
                WallType::Avoid => commands
                    .spawn(WallBundle::avoid(
                        meshes, materials, *position, *rotation, *size,
                    ))
                    .id(),
                WallType::Goal => commands
                    .spawn(WallBundle::goal(
                        meshes, materials, *position, *rotation, *size,
                    ))
                    .id(),
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
        .map(|item| item.spawn(commands, meshes, materials))
        .collect();

    Entities { light, level_items }
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
        app.init_asset_loader::<asset::LevelAssetLoader>()
            .add_asset::<Level>();
    }
}
