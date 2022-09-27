pub mod levels;

use bevy::prelude::*;

use crate::{
    controls::Controlled,
    fuel::{add_fuel, Fuel, FuelChanged},
    level::{load_level, CurrentLevel},
    ui::{tutorial::DisplayTutorial1, DisplayCompleteScreenEvent, NextLevelEvent},
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
    mut commands: Commands,
    mut next_level: EventReader<NextLevelEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    current_level: Res<CurrentLevel>,
) {
    for NextLevelEvent in next_level.iter() {
        commands.entity(current_level.player).despawn_recursive();

        for entity in &current_level.structure {
            commands.entity(*entity).despawn_recursive();
        }

        match current_level.next_level {
            Some(make_next_level) => {
                let next_level = make_next_level();

                load_level(&mut commands, &mut meshes, &mut materials, &next_level);
            }
            None => {
                debug!("no next_level selected")
            }
        }
    }
}

fn setup(mut display_tutorial_1: EventWriter<DisplayTutorial1>) {
    display_tutorial_1.send(DisplayTutorial1);
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(reset_when_player_hits_avoid)
            .add_system(show_complete_screen_on_goal)
            .add_system(restart_level)
            .add_system(next_level);
    }
}
