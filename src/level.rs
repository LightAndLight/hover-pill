pub mod asset;

use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};

use crate::{
    fuel_ball::FuelBallBundle,
    wall::{WallBundle, WallType},
};

#[derive(Debug, Serialize, Deserialize, TypeUuid, Clone, Default)]
#[uuid = "a79e94e4-1d11-4581-82f8-fb82cbc67f43"]
pub struct Level {
    pub player_start: Vec3,
    pub initial_overlay: Option<Vec<String>>,
    pub structure: Vec<LevelItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub fn position_mut(&mut self) -> &mut Vec3 {
        match self {
            LevelItem::Wall { position, .. } => position,
            LevelItem::FuelBall { position } => position,
            LevelItem::Light { position, .. } => position,
        }
    }

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

    pub fn spawn<'w, 's, 'a>(
        &self,
        commands: &'a mut Commands<'w, 's>,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) -> EntityCommands<'w, 's, 'a> {
        match self {
            LevelItem::Wall {
                wall_type,
                position,
                rotation,
                size,
            } => match wall_type {
                WallType::Neutral => commands.spawn(WallBundle::neutral(
                    meshes, materials, *position, *rotation, *size,
                )),
                WallType::Avoid => commands.spawn(WallBundle::avoid(
                    meshes, materials, *position, *rotation, *size,
                )),
                WallType::Goal => commands.spawn(WallBundle::goal(
                    meshes, materials, *position, *rotation, *size,
                )),
            },
            LevelItem::FuelBall { position } => {
                commands.spawn(FuelBallBundle::new(meshes, materials, *position))
            }
            LevelItem::Light {
                position,
                intensity,
            } => {
                let mut entity_commands = commands.spawn(PbrBundle {
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
                });

                entity_commands.with_children(|parent| {
                    parent.spawn(PointLightBundle {
                        point_light: PointLight {
                            intensity: *intensity,
                            radius: 0.1,
                            shadows_enabled: true,
                            ..default()
                        },
                        ..default()
                    });
                });

                entity_commands
            }
        }
    }
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<asset::LevelAssetLoader>()
            .add_asset::<Level>();
    }
}
