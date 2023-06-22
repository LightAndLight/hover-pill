pub mod collision;

use bevy::prelude::*;

use crate::{
    controls::Controlled,
    fuel::{add_fuel, Fuel, FuelChanged},
    level::{self, Level},
    ui::{self, UI},
};

use super::state::GameState;

#[derive(Resource)]
struct LoadingLevel {
    handle: Handle<Level>,
}

pub fn start_loading_level<'a, P: Into<bevy::asset::AssetPath<'a>>>(
    commands: &mut Commands,
    asset_server: &AssetServer,
    state: &mut NextState<GameState>,
    current_level: Option<&level::LoadedLevel>,
    path: P,
) {
    if let Some(current_level) = current_level {
        level::clear(commands, current_level);
    }

    let handle = asset_server.load(path);

    commands.insert_resource(LoadingLevel { handle });
    state.set(GameState::Loading);
}

pub fn finish_loading_level(
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

        play_level(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut fuel_changed,
            &mut state,
            &asset_server,
            &mut ui,
            &loading_level.handle,
            level,
        );
    }
}

#[derive(Resource)]
struct CurrentLevel {
    value: crate::level::LoadedLevel,
}

fn play_level(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    fuel_changed: &mut EventWriter<FuelChanged>,
    state: &mut NextState<GameState>,
    asset_server: &AssetServer,
    ui: &mut UI,
    handle: &Handle<Level>,
    level: &Level,
) {
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

        ui::overlay::level_overview::display(asset_server, commands, ui, overlay_text);
    } else {
        state.set(GameState::Playing);
    }
}

pub fn handle_next_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<NextState<GameState>>,
    mut input_events: EventReader<ui::overlay::level_complete::NextLevelEvent>,
    current_level: Res<CurrentLevel>,
    mut ui: ResMut<UI>,
    overlay: Res<ui::overlay::Overlay>,
) {
    use ui::overlay::level_complete::NextLevelEvent;

    for NextLevelEvent in input_events.iter() {
        trace!("next level");
        ui::overlay::remove(&mut commands, &mut ui, &overlay);

        if let Some(next_level) = &current_level.value.next_level {
            start_loading_level(
                &mut commands,
                &asset_server,
                &mut state,
                Some(&current_level.value),
                format!("levels/{}.json", next_level),
            );
        }
    }
}

pub fn hotreload_level(
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

                        play_level(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            &mut fuel_changed,
                            &mut state,
                            &asset_server,
                            &mut ui,
                            &current_level.value.handle,
                            level,
                        );
                    }
                }
            }
        }
    }
}

pub fn restart_level(
    keys: Res<Input<KeyCode>>,
    mut reset: (
        Res<CurrentLevel>,
        Query<(&mut Transform, &mut Fuel), With<Controlled>>,
        EventWriter<FuelChanged>,
    ),
) {
    if keys.just_pressed(KeyCode::R) {
        reset_player(&reset.0.value, &mut reset.1, &mut reset.2);
    }
}

fn reset_player(
    current_level: &crate::level::LoadedLevel,
    query: &mut Query<(&mut Transform, &mut Fuel), With<Controlled>>,
    fuel_changed: &mut EventWriter<FuelChanged>,
) {
    for (mut transform, mut fuel) in query {
        transform.translation = current_level.player_start;

        let amount = 1.0 - fuel.value;
        add_fuel(&mut fuel, amount, fuel_changed);
    }
}

pub fn handle_continue(
    mut state: ResMut<NextState<GameState>>,
    mut input_events: EventReader<ui::overlay::level_overview::ContinueEvent>,
    mut commands: Commands,
    mut ui: ResMut<UI>,
    overlay: Res<ui::overlay::Overlay>,
) {
    use ui::overlay::level_overview::ContinueEvent;

    for ContinueEvent in input_events.iter() {
        state.set(GameState::Playing);
        ui::overlay::remove(&mut commands, &mut ui, &overlay);
    }
}
