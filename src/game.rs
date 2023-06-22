pub mod level;
pub mod main_menu;
pub mod state;

use bevy::prelude::*;

use crate::ui::{self, UI};

use state::GameState;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut ui: ResMut<UI>) {
    ui::set(&mut commands, &mut ui, |commands| {
        ui::main_menu::create(&asset_server, commands)
    })
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_startup_system(setup)
            .add_system(level::hotreload_level)
            .add_system(level::finish_loading_level.in_set(OnUpdate(GameState::Loading)))
            .add_system(main_menu::handle_events.in_set(OnUpdate(GameState::MainMenu)))
            .add_systems(
                (level::handle_next_level, level::handle_continue)
                    .in_set(OnUpdate(GameState::Paused)),
            )
            .add_systems(
                (
                    level::collision::handle_player_collisions,
                    level::restart_level,
                )
                    .in_set(OnUpdate(GameState::Playing)),
            );
    }
}
