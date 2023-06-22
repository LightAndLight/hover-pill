use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
};
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_egui::EguiPlugin;
use bevy_rapier3d::prelude::*;
use hover_pill::{
    camera::ZoomPlugin,
    colored_wireframe::{ColoredWireframeConfig, ColoredWireframePlugin},
    config::Config,
    controls::ControlsPlugin,
    fuel::FuelPlugin,
    fuel_ball::FuelBallPlugin,
    game::GamePlugin,
    hover::HoverPlugin,
    level::LevelPlugin,
    level_editor::LevelEditorPlugin,
    player::PlayerPlugin,
    ui::{self, UiPlugin},
};

fn display_collision_events(mut collision_events: EventReader<CollisionEvent>) {
    for collision_event in collision_events.iter() {
        debug!("collision event: {:?}", collision_event);
    }
}

fn main() {
    let mut app = App::new();

    let config = Config {
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
    .insert_resource(config)
    .add_plugin(ColoredWireframePlugin)
    .insert_resource(ColoredWireframeConfig { enabled: true })
    .add_plugin(EguiPlugin)
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
    .add_plugin(ui::button::ButtonPlugin)
    .add_plugin(ControlsPlugin)
    .add_plugin(ZoomPlugin)
    .add_plugin(UiPlugin)
    .add_plugin(LevelPlugin)
    .add_plugin(FuelPlugin)
    .add_plugin(HoverPlugin)
    .add_plugin(FuelBallPlugin)
    .add_plugin(PlayerPlugin)
    .add_plugin(GamePlugin)
    .add_plugin(LevelEditorPlugin);

    if !cfg!(target_family = "wasm") {
        app.add_plugin(AtmospherePlugin);
    };

    if cfg!(debug_assertions) {
        app.add_plugin(RapierDebugRenderPlugin::default())
            .add_system(display_collision_events);
    }

    app.run();
}
