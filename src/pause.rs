use bevy::prelude::*;

use crate::{controls::Controlled, GameState};

#[derive(Component)]
struct Paused {
    controls: Controlled,
}

fn pause_player(mut commands: Commands, mut query: Query<(Entity, &mut Controlled)>) {
    for (entity, mut controlled) in &mut query {
        let controls = std::mem::take(controlled.as_mut());
        commands.entity(entity).insert(Paused { controls });
    }
}

fn unpause_player(mut commands: Commands, mut query: Query<(Entity, &mut Controlled, &Paused)>) {
    for (entity, mut controlled, paused) in &mut query {
        *controlled = paused.controls;
        commands.entity(entity).remove::<Paused>();
    }
}

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(pause_player.in_schedule(OnEnter(GameState::Paused)))
            .add_system(unpause_player.in_schedule(OnExit(GameState::Paused)));
    }
}
