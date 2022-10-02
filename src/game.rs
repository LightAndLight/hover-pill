use bevy::prelude::*;

use crate::{
    controls::Controlled,
    fuel::{add_fuel, Fuel, FuelChanged},
    level::{self, CurrentLevel},
    ui::{
        self,
        main_menu::MainMenuEvent,
        overlay::{self, Overlay},
        DisplayCompleteScreenEvent, NextLevelEvent, UI,
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

pub fn handle_main_menu(
    mut commands: Commands,
    mut input_events: EventReader<MainMenuEvent>,
    mut ui: ResMut<UI>,
    mut output_events: EventWriter<level::LoadEvent>,
) {
    for event in input_events.iter() {
        match event {
            MainMenuEvent::Play => {
                debug!("play");

                ui::clear(&mut commands, &mut ui);
                ui::remove_camera(&mut commands, &mut ui);

                output_events.send(level::LoadEvent {
                    path: "levels/tutorial_1.json".into(),
                })
            }
            MainMenuEvent::LevelEditor => {
                debug!("level editor")
            }
        }
    }
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(reset_when_player_hits_avoid)
            .add_system(show_complete_screen_on_goal)
            .add_system(restart_level)
            // .add_system(next_level)
            // .add_system(load_next_level)
            .add_system(handle_main_menu)
            // .add_system(ui::display_complete_screen)
            // .add_system(ui::handle_next_level)
            .add_system(ui::update_fuel_bar)
            // .add_system(level::reload_level);
            ;
    }
}
