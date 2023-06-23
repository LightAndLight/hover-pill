use bevy::prelude::*;

use crate::{
    controls::Controlled,
    fuel::{add_fuel, Fuel, FuelChanged},
    game::{level::CurrentLevel, state::GameState},
};

pub struct ResetEvent;

fn reset_player(
    current_level: Res<CurrentLevel>,
    mut query: Query<(&mut Transform, &mut Fuel), With<Controlled>>,
    mut fuel_changed_event: EventWriter<FuelChanged>,
) {
    for (mut transform, mut fuel) in &mut query {
        transform.translation = current_level.value.player_start;

        let amount = 1.0 - fuel.value;
        add_fuel(&mut fuel, amount, &mut fuel_changed_event);
    }
}

pub struct ResetPlugin;

impl Plugin for ResetPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ResetEvent>()
            .add_system(reset_player.in_set(OnUpdate(GameState::Playing)));
    }
}
