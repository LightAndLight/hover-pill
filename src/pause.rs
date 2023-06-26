use bevy::prelude::*;

use crate::controls::{Controlled, ControlsConfig};

pub enum PauseEvent {
    Pause,
    Unpause,
}

#[derive(Component)]
struct Paused {
    controls: Controlled,
}

fn pause_player(
    mut commands: Commands,
    mut controls_config: ResMut<ControlsConfig>,
    mut pause_events: EventReader<PauseEvent>,
    mut query: Query<(Entity, &mut Controlled)>,
) {
    if let Some(PauseEvent::Pause) = pause_events.iter().last() {
        controls_config.enabled = false;

        for (entity, mut controlled) in &mut query {
            let controls = std::mem::take(controlled.as_mut());
            commands.entity(entity).insert(Paused { controls });
        }
    }
}

fn unpause_player(
    mut commands: Commands,
    mut controls_config: ResMut<ControlsConfig>,
    mut pause_events: EventReader<PauseEvent>,
    mut query: Query<(Entity, &mut Controlled, &Paused)>,
) {
    if let Some(PauseEvent::Unpause) = pause_events.iter().last() {
        controls_config.enabled = true;

        for (entity, mut controlled, paused) in &mut query {
            *controlled = paused.controls;
            commands.entity(entity).remove::<Paused>();
        }
    }
}

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PauseEvent>()
            .add_system(pause_player)
            .add_system(unpause_player);
    }
}
