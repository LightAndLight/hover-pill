use bevy::prelude::*;
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_rapier3d::prelude::*;
use hover_pill::{
    fuel::FuelPlugin,
    game::{GamePlugin, GameState},
    hover::HoverPlugin,
    level::LevelPlugin,
    ui::{self, UiPlugin},
    world::WorldPlugin,
};

fn display_collision_events(mut collision_events: EventReader<CollisionEvent>) {
    for collision_event in collision_events.iter() {
        debug!("collision event: {:?}", collision_event);
    }
}

fn setup(asset_server: Res<AssetServer>) {
    if !cfg!(target_family = "wasm") {
        asset_server.watch_for_changes().unwrap();
    }
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(ui::button::ButtonPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(LevelPlugin)
        .add_plugin(WorldPlugin)
        .add_plugin(FuelPlugin)
        .add_plugin(HoverPlugin)
        .add_plugin(GamePlugin)
        .add_startup_system(setup);

    if !cfg!(target_family = "wasm") {
        app.add_plugin(AtmospherePlugin);
    };

    app.add_state(GameState::MainMenu)
        .add_system_set(SystemSet::on_enter(GameState::MainMenu).with_system(ui::main_menu::setup))
        .add_system_set(
            SystemSet::on_update(GameState::MainMenu).with_system(ui::main_menu::handle_buttons),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::MainMenu).with_system(ui::main_menu::teardown),
        );

    if cfg!(debug_assertions) {
        app.add_plugin(RapierDebugRenderPlugin::default())
            .add_system(display_collision_events);
    }

    /*
    app.insert_resource(WindowDescriptor {
        title: String::from("Hover Pill"),
        canvas: Some(String::from("#app")),
        ..Default::default()
    })
    .insert_resource(ReportExecutionOrderAmbiguities::default())
    .init_resource::<ui::Overlay>();

    if cfg!(debug_assertions) {
        app.add_plugin(LogDiagnosticsPlugin::default())
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(RapierDebugRenderPlugin::default());

        /*
        if !cfg!(target_family = "wasm") {
            app.insert_resource(AssetServerSettings {
                watch_for_changes: true,
                ..Default::default()
            });
        };
        */
    }

        .add_plugin(FuelPlugin)
        .add_plugin(FuelBallPlugin)
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup);

    */

    app.run();
}
