pub mod levels;

use bevy::prelude::*;

use crate::{
    controls::Controlled,
    fuel::{add_fuel, Fuel, FuelChanged},
    level::{load_level, CurrentLevel},
    ui::{complete_screen, NextLevelEvent},
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
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_hit: EventReader<PlayerHit>,
) {
    for event in player_hit.iter() {
        if let PlayerHit::Goal = event {
            complete_screen(&mut commands, &asset_server)
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
    mut commands: Commands,
    mut next_level: EventReader<NextLevelEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    current_level: ResMut<CurrentLevel>,
    mut player_query: Query<&mut Transform, With<Controlled>>,
) {
    for NextLevelEvent in next_level.iter() {
        for entity in &current_level.structure {
            commands.entity(*entity).despawn();
            match current_level.next_level {
                Some(make_next_level) => {
                    let next_level = make_next_level();
                    load_level(&mut commands, &mut meshes, &mut materials, &next_level);
                    for mut transform in &mut player_query {
                        transform.translation = next_level.player_start;
                    }
                }
                None => {
                    debug!("no next_level selected")
                }
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
            .add_system(next_level);
    }
}
