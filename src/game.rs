use bevy::prelude::*;

use crate::{
    controls::Controlled,
    fuel::{add_fuel, Fuel, FuelChanged},
    level,
    ui::{self, main_menu::MainMenuEvent, UI},
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

fn handle_goal(
    mut commands: Commands,
    mut player_hit: EventReader<PlayerHit>,
    asset_server: Res<AssetServer>,
    mut ui: ResMut<UI>,
) {
    for event in player_hit.iter() {
        if let PlayerHit::Goal = event {
            debug!("player hit goal");

            ui::overlay::level_complete::display(&asset_server, &mut commands, &mut ui);
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

pub fn handle_main_menu(
    mut commands: Commands,
    mut input_events: EventReader<MainMenuEvent>,
    asset_server: Res<AssetServer>,
    mut ui: ResMut<UI>,
    mut output_events: EventWriter<level::LoadEvent>,
) {
    for event in input_events.iter() {
        match event {
            MainMenuEvent::Play => {
                debug!("play");

                ui::clear(&mut commands, &mut ui);
                ui::remove_camera(&mut commands, &mut ui);

                ui::set(&mut commands, &mut ui, |commands| {
                    ui::fuel_bar::create(commands, &asset_server)
                });

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
            .add_system(handle_goal)
            .add_system(restart_level)
            // .add_system(next_level)
            // .add_system(load_next_level)
            .add_system(handle_main_menu)
            // .add_system(ui::display_complete_screen)
            // .add_system(ui::handle_next_level)
            // .add_system(level::reload_level);
            ;
    }
}
