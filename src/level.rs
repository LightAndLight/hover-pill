use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
};
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

pub enum CurrentLevel {
    Loaded {
        handle: Handle<Level>,
        next_level: Option<String>,
        player_start: Vec3,
        structure: Vec<Entity>,
        player: Entity,
    },
    Loading(Handle<Level>),
}

pub fn clear_level(current_level: &CurrentLevel, commands: &mut Commands) {
    if let CurrentLevel::Loaded {
        structure, player, ..
    } = current_level
    {
        commands.entity(*player).despawn_recursive();

        for entity in structure {
            commands.entity(*entity).despawn_recursive();
        }
    }
}

pub fn load_level(
    asset_server: &AssetServer,
    overlay: &Overlay,
    visibility_query: &mut Query<&mut Visibility>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    handle: Handle<Level>,
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

    commands.insert_resource(CurrentLevel::Loaded {
        handle,
        next_level: level.next_level.clone(),
        player_start: level.player_start,
        structure: entities,
        player,
    });
}

fn reload_level(
    mut asset_event: EventReader<AssetEvent<Level>>,
    asset_server: Res<AssetServer>,
    assets: Res<Assets<Level>>,
    overlay: Res<Overlay>,
    mut visibility_query: Query<&mut Visibility>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
                            &overlay,
                            &mut visibility_query,
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            current_level_handle.clone(),
                            level,
                        );
                    }
                }
            }
        }
    }
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<LevelAssetLoader>()
            .add_asset::<Level>()
            .add_system(reload_level);
    }
}
