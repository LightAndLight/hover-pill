use bevy::prelude::*;

use crate::{
    level_editor,
    ui::{self, main_menu::MainMenuEvent, UI},
};

use super::state::GameState;

pub fn handle_events(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<NextState<GameState>>,
    mut input_events: EventReader<MainMenuEvent>,
    mut ui: ResMut<UI>,
    mut editor_load_level: EventWriter<level_editor::LoadEvent>,
) {
    if let Some(event) = input_events.iter().last() {
        match event {
            MainMenuEvent::Play => {
                debug!("play");

                ui::clear(&mut commands, &mut ui);
                ui::camera_off(&mut commands, &mut ui);

                ui::set(&mut commands, &mut ui, |commands| {
                    ui::fuel_bar::create(commands, &asset_server)
                });

                super::level::start_loading_level(
                    &mut commands,
                    &asset_server,
                    &mut state,
                    None,
                    "levels/tutorial_1.json",
                );
            }
            MainMenuEvent::LevelEditor => {
                debug!("level editor");

                ui::clear(&mut commands, &mut ui);
                ui::camera_off(&mut commands, &mut ui);

                editor_load_level.send(level_editor::LoadEvent {
                    path: "levels/tutorial_1.json".into(),
                })
            }
        }
    }
}
