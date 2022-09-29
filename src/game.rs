pub mod levels;

use bevy::prelude::*;

use crate::{
    controls::Controlled,
    fuel::{add_fuel, Fuel, FuelChanged},
    level::{self, load_level, CurrentLevel},
    ui::{DisplayCompleteScreenEvent, NextLevelEvent, Overlay},
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
    assets: Res<Assets<level::Level>>,
    overlay: Res<Overlay>,
    current_level: Res<CurrentLevel>,
    mut visibility_query: Query<&mut Visibility>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for NextLevelEvent in next_level.iter() {
        commands.entity(current_level.player).despawn_recursive();

        for entity in &current_level.structure {
            commands.entity(*entity).despawn_recursive();
        }

        match &current_level.next_level {
            Some(next_level_name) => {
                let next_level_path = format!("levels/{}.json", next_level_name);

                let next_level = asset_server.load::<level::Level, _>(&next_level_path);

                load_level(
                    &asset_server,
                    &overlay,
                    &mut visibility_query,
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    assets.get(&next_level).unwrap(),
                );
            }
            None => {
                debug!("no next_level selected")
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
