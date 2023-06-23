use bevy::prelude::*;

use crate::{
    ui::{self, UI},
    GameState,
};

fn handle_continue(
    mut state: ResMut<NextState<GameState>>,
    mut input_events: EventReader<ui::overlay::level_overview::ContinueEvent>,
    mut commands: Commands,
    mut ui: ResMut<UI>,
    overlay: Res<ui::overlay::Overlay>,
) {
    use ui::overlay::level_overview::ContinueEvent;

    for ContinueEvent in input_events.iter() {
        state.set(GameState::Playing);
        ui::overlay::remove(&mut commands, &mut ui, &overlay);
    }
}

pub struct ContinuePlugin;

impl Plugin for ContinuePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_continue.in_set(OnUpdate(GameState::Paused)));
    }
}
