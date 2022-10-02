use bevy::prelude::*;

use crate::{
    controls::Controlled,
    fuel::{add_fuel, Fuel, FuelChanged},
    level::{self, CurrentLevel},
    ui::{
        self,
        overlay::{self, Overlay},
        DisplayCompleteScreenEvent, NextLevelEvent,
    },
    world::PlayerHit,
};

fn reset_player_position(mut transform: &mut Transform) {
    transform.translation = 2.0 * Vec3::Y;
}

fn reset_when_player_hits_avoid(
    mut player_hit: EventReader<PlayerHit>,
    mut query: Query<&mut Transform, With<Controlled>>,
) {
    for event in player_hit.iter() {
        if let PlayerHit::Avoid = event {
            for mut transform in &mut query {
                reset_player_position(&mut transform);
            }
        }
    }
}

fn show_complete_screen_on_goal(
    mut player_hit: EventReader<PlayerHit>,
    mut display_complete_screen: EventWriter<DisplayCompleteScreenEvent>,
) {
    for event in player_hit.iter() {
        if let PlayerHit::Goal = event {
            debug!("player hit goal");
            display_complete_screen.send(DisplayCompleteScreenEvent);
        }
    }
}

fn restart_level(
    keys: Res<Input<KeyCode>>,
    mut query: Query<(&mut Fuel, &mut Transform), With<Controlled>>,
    mut fuel_changed: EventWriter<FuelChanged>,
) {
    if keys.just_pressed(KeyCode::R) {
        for (mut fuel, mut transform) in &mut query {
            reset_player_position(&mut transform);

            let amount = 1.0 - fuel.value;
            add_fuel(&mut fuel, amount, &mut fuel_changed);
        }
    }
}

fn next_level(
    mut next_level: EventReader<NextLevelEvent>,
    asset_server: Res<AssetServer>,
    mut current_level: ResMut<CurrentLevel>,
    mut commands: Commands,
) {
    for NextLevelEvent in next_level.iter() {
        level::clear_level(&current_level, &mut commands);

        if let CurrentLevel::Loaded { next_level, .. } = current_level.as_ref() {
            match next_level {
                Some(next_level_name) => {
                    let next_level_path = format!("levels/{}.json", next_level_name);

                    debug!("next_level_path: {:?}", next_level_path);
                    let next_level_handle = asset_server.load::<level::Level, _>(&next_level_path);

                    *current_level = CurrentLevel::Loading(next_level_handle);
                }
                None => {
                    debug!("no next_level selected")
                }
            }
        }
    }
}

fn load_next_level(
    asset_server: Res<AssetServer>,
    assets: Res<Assets<level::Level>>,
    overlay: Res<Overlay>,
    current_level: Res<CurrentLevel>,
    mut visibility_query: Query<&mut Visibility>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut fuel_changed: EventWriter<FuelChanged>,
) {
    if let CurrentLevel::Loading(next_level_handle) = current_level.as_ref() {
        if let Some(level) = assets.get(next_level_handle) {
            level::load_level(
                &asset_server,
                &overlay,
                &mut visibility_query,
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut fuel_changed,
                next_level_handle.clone(),
                level,
            );
        }
    }
}

fn load_level(asset_server: &AssetServer, path: &str, commands: &mut Commands) {
    let next_level_handle = asset_server.load(path);
    commands.insert_resource(level::CurrentLevel::Loading(next_level_handle));
}

pub fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.init_resource::<Overlay>();
    ui::display_fuel_bar(&mut commands, &asset_server);
    load_level(&asset_server, "levels/tutorial_1.json", &mut commands);
}

pub fn teardown(mut commands: Commands) {
    commands.remove_resource::<Overlay>();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    MainMenu,
    Playing,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup))
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(reset_when_player_hits_avoid)
                    .with_system(show_complete_screen_on_goal)
                    .with_system(restart_level)
                    .with_system(next_level)
                    .with_system(load_next_level)
                    .with_system(overlay::handle_continue)
                    .with_system(ui::display_complete_screen)
                    .with_system(ui::handle_next_level)
                    .with_system(ui::update_fuel_bar)
                    .with_system(level::reload_level),
            )
            .add_system_set(SystemSet::on_exit(GameState::Playing).with_system(teardown));
    }
}
