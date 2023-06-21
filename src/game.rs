use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    controls::Controlled,
    fuel::{add_fuel, Fuel, FuelChanged},
    level::{self, wall, Level},
    level_editor,
    ui::{self, main_menu::MainMenuEvent, UI},
};

fn reset_player(
    current_level: &level::LoadedLevel,
    query: &mut Query<(&mut Transform, &mut Fuel), With<Controlled>>,
    fuel_changed: &mut EventWriter<FuelChanged>,
) {
    for (mut transform, mut fuel) in query {
        transform.translation = current_level.player_start;

        let amount = 1.0 - fuel.value;
        add_fuel(&mut fuel, amount, fuel_changed);
    }
}
enum PlayerHit {
    Avoid,
    Goal,
}

fn check_player_hit(
    player_query: &Query<&Controlled>,
    entity1: &Entity,
    entity2: &Entity,
    wall_type_query: &Query<&wall::WallType>,
) -> Option<PlayerHit> {
    let (player, target) = if player_query.contains(*entity1) {
        Some((*entity1, *entity2))
    } else if player_query.contains(*entity2) {
        Some((*entity2, *entity1))
    } else {
        None
    }?;

    if let Ok(wall_type) = wall_type_query.get(target) {
        match wall_type {
            wall::WallType::Avoid => {
                debug!("player {:?} hit avoid {:?}", player, target);
                Some(PlayerHit::Avoid)
            }
            wall::WallType::Goal => {
                debug!("player {:?} hit goal {:?}", player, target);
                Some(PlayerHit::Goal)
            }
        }
    } else {
        None
    }
}

#[derive(Resource)]
struct CurrentLevel {
    value: level::LoadedLevel,
}

fn handle_player_collisions(
    mut state: ResMut<NextState<State>>,
    mut collision_events: EventReader<CollisionEvent>,
    check: (Query<&Controlled>, Query<&wall::WallType>),
    mut reset: (
        Res<CurrentLevel>,
        Query<(&mut Transform, &mut Fuel), With<Controlled>>,
        EventWriter<FuelChanged>,
    ),
    mut goal: (Res<AssetServer>, Commands, ResMut<UI>),
) {
    for event in collision_events.iter() {
        if let CollisionEvent::Started(entity1, entity2, _) = event {
            let event = check_player_hit(&check.0, entity1, entity2, &check.1);

            if let Some(event) = event {
                match event {
                    PlayerHit::Avoid => {
                        reset_player(&reset.0.value, &mut reset.1, &mut reset.2);
                    }
                    PlayerHit::Goal => {
                        state.set(State::Paused);

                        ui::overlay::level_complete::display(&goal.0, &mut goal.1, &mut goal.2);
                    }
                }
            }
        }
    }
}

fn restart_level(
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

#[derive(Resource)]
struct LoadingLevel {
    handle: Handle<Level>,
}

fn start_loading_level<'a, P: Into<bevy::asset::AssetPath<'a>>>(
    commands: &mut Commands,
    asset_server: &AssetServer,
    state: &mut NextState<State>,
    current_level: Option<&CurrentLevel>,
    path: P,
) {
    if let Some(current_level) = current_level {
        level::clear(commands, &current_level.value);
        commands.remove_resource::<CurrentLevel>();
    }

    let handle = asset_server.load(path);

    commands.insert_resource(LoadingLevel { handle });
    state.set(State::Loading);
}

fn handle_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<NextState<State>>,
    current_level: Option<Res<CurrentLevel>>,
    mut input_events: EventReader<MainMenuEvent>,
    mut ui: ResMut<UI>,
    mut editor_load_level: EventWriter<level_editor::LoadEvent>,
) {
    for event in input_events.iter() {
        match event {
            MainMenuEvent::Play => {
                debug!("play");

                ui::clear(&mut commands, &mut ui);
                ui::camera_off(&mut commands, &mut ui);

                ui::set(&mut commands, &mut ui, |commands| {
                    ui::fuel_bar::create(commands, &asset_server)
                });

                start_loading_level(
                    &mut commands,
                    &asset_server,
                    &mut state,
                    current_level.as_ref().map(|x| x.as_ref()),
                    "levels/tutorial_1.json",
                );
            }
            MainMenuEvent::LevelEditor => {
                debug!("level editor");

                ui::clear(&mut commands, &mut ui);
                ui::camera_off(&mut commands, &mut ui);

                editor_load_level.send(level_editor::LoadEvent {
                    path: "levels/tutorial_1.json".into(),
                })
            }
        }
    }
}

fn play_level(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    fuel_changed: &mut EventWriter<FuelChanged>,
    state: &mut NextState<State>,
    asset_server: &AssetServer,
    ui: &mut UI,
    handle: &Handle<Level>,
    level: &Level,
) {
    commands.remove_resource::<LoadingLevel>();

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
        state.set(State::Paused);

        ui::overlay::level_overview::display(asset_server, commands, ui, overlay_text);
    } else {
        state.set(State::Playing);
    }
}

fn finish_loading_level(
    mut state: ResMut<NextState<State>>,
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

fn handle_continue(
    mut state: ResMut<NextState<State>>,
    mut input_events: EventReader<ui::overlay::level_overview::ContinueEvent>,
    mut commands: Commands,
    mut ui: ResMut<UI>,
    overlay: Res<ui::overlay::Overlay>,
) {
    use ui::overlay::level_overview::ContinueEvent;

    for ContinueEvent in input_events.iter() {
        state.set(State::Playing);
        ui::overlay::remove(&mut commands, &mut ui, &overlay);
    }
}

fn handle_next_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<NextState<State>>,
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
                Some(&current_level),
                format!("levels/{}.json", next_level),
            );
        }
    }
}

fn hotreload_level(
    mut commands: Commands,
    mut state: ResMut<NextState<State>>,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, States)]
pub enum State {
    #[default]
    MainMenu,
    Loading,
    Paused,
    Playing,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<State>()
            .add_event::<PlayerHit>()
            .add_system(hotreload_level)
            .add_system(finish_loading_level.in_set(OnUpdate(State::Loading)))
            .add_system(handle_main_menu.in_set(OnUpdate(State::MainMenu)))
            .add_systems((handle_next_level, handle_continue).in_set(OnUpdate(State::Paused)))
            .add_systems(
                (handle_player_collisions, restart_level).in_set(OnUpdate(State::Playing)),
            );
    }
}
