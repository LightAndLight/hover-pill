use bevy::prelude::*;

use crate::{
    fuel_ball::FuelBallBundle,
    world::{Avoid, Goal, WallBundle},
};

pub struct Level {
    pub player_start: Vec3,
    pub structure: Vec<LevelItem>,
}

pub enum WallType {
    Neutral,
    Avoid,
    Goal,
}

pub enum LevelItem {
    Wall {
        wall_type: WallType,
        transform: Transform,
        size: Vec2,
    },
    FuelBall {
        position: Vec3,
    },
}

pub struct CurrentLevel {
    pub player_start: Vec3,
    pub structure: Vec<Entity>,
}

pub fn load_level(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    level: &Level,
) {
    let entities: Vec<Entity> = level
        .structure
        .iter()
        .map(|item| match item {
            LevelItem::Wall {
                wall_type,
                transform,
                size,
            } => match wall_type {
                WallType::Neutral => commands
                    .spawn_bundle(WallBundle::new(
                        meshes,
                        materials,
                        *transform,
                        *size,
                        Color::WHITE,
                    ))
                    .id(),
                WallType::Avoid => commands
                    .spawn_bundle(WallBundle::new(
                        meshes,
                        materials,
                        *transform,
                        *size,
                        Color::RED,
                    ))
                    .insert(Avoid)
                    .id(),
                WallType::Goal => commands
                    .spawn_bundle(WallBundle::new(
                        meshes,
                        materials,
                        *transform,
                        *size,
                        Color::GREEN,
                    ))
                    .insert(Goal)
                    .id(),
            },
            LevelItem::FuelBall { position } => commands
                .spawn_bundle(FuelBallBundle::new(meshes, materials, *position))
                .id(),
        })
        .collect();

    commands.insert_resource(CurrentLevel {
        player_start: level.player_start,
        structure: entities,
    });
}
