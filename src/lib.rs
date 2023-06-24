use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
};
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_egui::EguiPlugin;
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};

pub mod camera;
pub mod collision;
pub mod colored_wireframe;
pub mod config;
pub mod r#continue;
pub mod controls;
pub mod cylinder;
pub mod debug;
pub mod fuel;
pub mod fuel_ball;
pub mod hover;
pub mod jump;
pub mod level;
pub mod level_editor;
pub mod load_level;
pub mod main_menu;
pub mod next_level;
pub mod pause;
pub mod player;
pub mod reset;
pub mod ui;
pub mod wall;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, States)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    Editing,
    Testing,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut ui: ResMut<ui::UI>) {
    ui::set(&mut commands, &mut ui, |commands| {
        main_menu::create(&asset_server, commands)
    })
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let config = config::Config {
            asset_dir: "assets".into(),
        };

        app.edit_schedule(CoreSchedule::Main, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        })
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            asset_folder: config.asset_dir.clone(),
        }))
        .add_state::<GameState>()
        .insert_resource(config)
        .insert_resource(colored_wireframe::ColoredWireframeConfig { enabled: true })
        .add_plugin(colored_wireframe::ColoredWireframePlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(ui::button::ButtonPlugin)
        .add_plugin(controls::ControlsPlugin)
        .add_plugin(camera::ZoomPlugin)
        .add_plugin(ui::UiPlugin)
        .add_plugin(level::LevelPlugin)
        .add_plugin(fuel::FuelPlugin)
        .add_plugin(hover::HoverPlugin)
        .add_plugin(fuel_ball::FuelBallPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(reset::ResetPlugin)
        .add_plugin(load_level::LoadLevelPlugin)
        .add_plugin(next_level::NextLevelPlugin)
        .add_plugin(r#continue::ContinuePlugin)
        .add_plugin(pause::PausePlugin)
        .add_plugin(main_menu::MainMenuPlugin)
        .add_plugin(level_editor::LevelEditorPlugin)
        .add_startup_system(setup)
        .add_system(collision::handle_player_collisions.in_set(OnUpdate(GameState::Playing)));

        if !cfg!(target_family = "wasm") {
            app.add_plugin(AtmospherePlugin);
        };

        if cfg!(debug_assertions) {
            app.add_plugin(debug::DebugPlugin);
        }
    }
}
