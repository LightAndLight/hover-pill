use bevy::prelude::*;

use crate::{
    level_editor, load_level,
    ui::{self, main_menu::MainMenuEvent, UI},
};

pub fn handle_events(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut input_events: EventReader<MainMenuEvent>,
    mut ui: ResMut<UI>,
    mut load_event: EventWriter<load_level::LoadEvent>,
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

                load_event.send(load_level::LoadEvent {
                    path: "levels/tutorial_1.json".into(),
                });
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
