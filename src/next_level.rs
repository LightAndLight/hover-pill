use bevy::prelude::*;

use crate::{
    level_order::CurrentLevelOrder,
    load_level::{CurrentLevel, LoadEvent},
    pause::PauseEvent,
    ui::{self, UI},
};

fn handle_next_level(
    mut commands: Commands,
    mut input_events: EventReader<ui::overlay::level_complete::NextLevelEvent>,
    current_level_order: Res<CurrentLevelOrder>,
    current_level: Res<CurrentLevel>,
    mut ui: ResMut<UI>,
    overlay: Res<ui::overlay::Overlay>,
    mut pause_event: EventWriter<PauseEvent>,
    mut load_event: EventWriter<LoadEvent>,
) {
    use ui::overlay::level_complete::NextLevelEvent;

    if let Some(NextLevelEvent) = input_events.iter().last() {
        trace!("next level");

        if let Some(next_level) = current_level_order
            .level_order
            .next_level(&current_level.path)
        {
            ui::overlay::remove(&mut commands, &mut ui, &overlay);
            pause_event.send(PauseEvent::Unpause);

            load_event.send(LoadEvent {
                path: String::from(next_level),
            })
        }
    }
}

pub struct NextLevelPlugin;

impl Plugin for NextLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_next_level.run_if(resource_exists::<CurrentLevel>()));
    }
}
