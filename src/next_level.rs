use bevy::prelude::*;

use crate::{
    load_level::{CurrentLevel, LoadEvent},
    ui::{self, UI},
};

fn handle_next_level(
    mut commands: Commands,
    mut input_events: EventReader<ui::overlay::level_complete::NextLevelEvent>,
    current_level: Res<CurrentLevel>,
    mut ui: ResMut<UI>,
    overlay: Res<ui::overlay::Overlay>,
    mut load_event: EventWriter<LoadEvent>,
) {
    use ui::overlay::level_complete::NextLevelEvent;

    for NextLevelEvent in input_events.iter() {
        trace!("next level");
        ui::overlay::remove(&mut commands, &mut ui, &overlay);

        if let Some(next_level) = &current_level.value.next_level {
            load_event.send(LoadEvent {
                path: format!("levels/{}.json", next_level),
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
