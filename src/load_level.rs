use bevy::prelude::*;

use crate::{
    fuel::FuelChanged,
    level::{self, Level},
    ui::{self, UI},
    GameState,
};

pub struct LoadEvent {
    pub path: String,
}

#[derive(Resource)]
pub struct CurrentLevel {
    pub value: crate::level::LoadedLevel,
}

#[derive(Resource)]
pub struct LoadingLevel {
    pub handle: Handle<Level>,
}

fn start_loading(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<NextState<GameState>>,
    current_level: Option<Res<CurrentLevel>>,
    mut load_events: EventReader<LoadEvent>,
) {
    if let Some(LoadEvent { path }) = load_events.iter().last() {
        if let Some(current_level) = current_level {
            level::clear(&mut commands, &current_level.value);
        }

        let handle = asset_server.load(path);

        commands.insert_resource(LoadingLevel { handle });
        state.set(GameState::Loading);
    }
}

fn finish_loading(
    mut state: ResMut<NextState<GameState>>,
    loading_level: Res<LoadingLevel>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    assets: Res<Assets<Level>>,
    mut ui: ResMut<UI>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut fuel_changed: EventWriter<FuelChanged>,
) {
    if let Some(level) = assets.get(&loading_level.handle) {
        commands.remove_resource::<LoadingLevel>();

        let loaded_level = level::create(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut fuel_changed,
            loading_level.handle.clone(),
            level,
        );

        commands.insert_resource(CurrentLevel {
            value: loaded_level,
        });

        if let Some(overlay_text) = &level.initial_overlay {
            state.set(GameState::Paused);

            ui::overlay::level_overview::display(
                &asset_server,
                &mut commands,
                &mut ui,
                overlay_text,
            );
        } else {
            state.set(GameState::Playing);
        }
    }
}

fn hotreload(
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>,
    assets: Res<Assets<Level>>,
    mut asset_event: EventReader<AssetEvent<Level>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ui: ResMut<UI>,
    mut fuel_changed: EventWriter<FuelChanged>,
    current_level: Option<Res<CurrentLevel>>,
) {
    if let Some(current_level) = current_level {
        for event in asset_event.iter() {
            debug!("level asset event: {:?}", event);

            if let AssetEvent::Modified {
                handle: modified_handle,
            } = event
            {
                debug!("asset modified: {:?}", modified_handle);

                if modified_handle == &current_level.value.handle {
                    if let Some(level) = assets.get(&current_level.value.handle) {
                        level::clear(&mut commands, &current_level.value);

                        {
                            let commands: &mut Commands = &mut commands;
                            let meshes: &mut Assets<Mesh> = &mut meshes;
                            let materials: &mut Assets<StandardMaterial> = &mut materials;
                            let fuel_changed: &mut EventWriter<FuelChanged> = &mut fuel_changed;
                            let state: &mut NextState<GameState> = &mut state;
                            let asset_server: &AssetServer = &asset_server;
                            let ui: &mut UI = &mut ui;
                            let handle = &current_level.value.handle;
                            let loaded_level = level::create(
                                commands,
                                meshes,
                                materials,
                                fuel_changed,
                                handle.clone(),
                                level,
                            );

                            commands.insert_resource(CurrentLevel {
                                value: loaded_level,
                            });

                            if let Some(overlay_text) = &level.initial_overlay {
                                state.set(GameState::Paused);

                                ui::overlay::level_overview::display(
                                    asset_server,
                                    commands,
                                    ui,
                                    overlay_text,
                                );
                            } else {
                                state.set(GameState::Playing);
                            }
                        };
                    }
                }
            }
        }
    }
}

pub struct LoadLevelPlugin;

impl Plugin for LoadLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadEvent>()
            .add_system(start_loading)
            .add_system(finish_loading.run_if(resource_exists::<LoadingLevel>()))
            .add_system(hotreload);
    }
}
