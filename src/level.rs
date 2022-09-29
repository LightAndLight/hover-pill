use bevy::{prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};

use crate::{
    fuel_ball::FuelBallBundle,
    player::spawn_player,
    ui::{tutorial::display_level_overlay, Overlay},
    world::{Avoid, Goal, WallBundle},
};

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "a79e94e4-1d11-4581-82f8-fb82cbc67f43"]
pub struct Level {
    pub next_level: Option<String>,
    pub player_start: Vec3,
    pub initial_overlay: Option<Vec<String>>,
    pub structure: Vec<LevelItem>,
}

#[derive(Serialize, Deserialize)]
pub enum WallType {
    Neutral,
    Avoid,
    Goal,
}

#[derive(Serialize, Deserialize)]
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
}

pub struct CurrentLevel {
    pub next_level: Option<String>,
    pub player_start: Vec3,
    pub structure: Vec<Entity>,
    pub player: Entity,
}

pub fn load_level(
    asset_server: &AssetServer,
    overlay: &Overlay,
    visibility_query: &mut Query<&mut Visibility>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    level: &Level,
) {
    debug!("started loading level");

    let entities: Vec<Entity> = level
        .structure
        .iter()
        .map(|item| match item {
            LevelItem::Wall {
                wall_type,
                position,
                rotation,
                size,
            } => match wall_type {
                WallType::Neutral => commands
                    .spawn_bundle(WallBundle::new(
                        meshes,
                        materials,
                        Transform::identity()
                            .with_translation(*position)
                            .with_rotation(*rotation),
                        *size,
                        Color::WHITE,
                    ))
                    .id(),
                WallType::Avoid => commands
                    .spawn_bundle(WallBundle::new(
                        meshes,
                        materials,
                        Transform::identity()
                            .with_translation(*position)
                            .with_rotation(*rotation),
                        *size,
                        Color::RED,
                    ))
                    .insert(Avoid)
                    .id(),
                WallType::Goal => commands
                    .spawn_bundle(WallBundle::new(
                        meshes,
                        materials,
                        Transform::identity()
                            .with_translation(*position)
                            .with_rotation(*rotation),
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

    let player = spawn_player(
        commands,
        meshes,
        materials,
        Transform::from_translation(level.player_start),
    );

    if let Some(overlay_text) = &level.initial_overlay {
        display_level_overlay(
            asset_server,
            commands,
            overlay,
            visibility_query,
            overlay_text,
        );
    }

    debug!("finished loading level");

    commands.insert_resource(CurrentLevel {
        next_level: level.next_level.clone(),
        player_start: level.player_start,
        structure: entities,
        player,
    });
}
