use bevy::prelude::*;

use crate::{
    pause::PauseEvent,
    ui::{self, UI},
};

fn handle_continue(
    mut input_events: EventReader<ui::overlay::level_overview::ContinueEvent>,
    mut commands: Commands,
    mut ui: ResMut<UI>,
    overlay: Res<ui::overlay::Overlay>,
    mut pause_event: EventWriter<PauseEvent>,
) {
    use ui::overlay::level_overview::ContinueEvent;

    for ContinueEvent in input_events.iter() {
        pause_event.send(PauseEvent::Unpause);
        ui::overlay::remove(&mut commands, &mut ui, &overlay);
    }
}

pub struct ContinuePlugin;

impl Plugin for ContinuePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_continue);
    }
}
