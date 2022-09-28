use bevy::prelude::*;

use crate::{
    fuel_ball::FuelBallBundle,
    player::spawn_player,
    ui::{tutorial::OverlayFn, Overlay},
    world::{Avoid, Goal, WallBundle},
};

pub struct Level {
    pub next_level: Option<fn() -> Level>,
    pub player_start: Vec3,
    pub initial_overlay: Option<OverlayFn>,
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
    pub next_level: Option<fn() -> Level>,
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

    let player = spawn_player(
        commands,
        meshes,
        materials,
        Transform::from_translation(level.player_start),
    );

    if let Some(overlay_fn) = level.initial_overlay {
        overlay_fn(asset_server, commands, overlay, visibility_query);
    }

    debug!("finished loading level");

    commands.insert_resource(CurrentLevel {
        next_level: level.next_level,
        player_start: level.player_start,
        structure: entities,
        player,
    });
}
