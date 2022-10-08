use bevy::{pbr::wireframe::WireframePlugin, prelude::*};
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_egui::EguiPlugin;
use bevy_rapier3d::prelude::*;
use hover_pill::{
    camera::ZoomPlugin,
    controls::ControlsPlugin,
    fuel::FuelPlugin,
    fuel_ball::FuelBallPlugin,
    game::GamePlugin,
    hover::HoverPlugin,
    level::LevelPlugin,
    level_editor::LevelEditorPlugin,
    player::PlayerPlugin,
    ui::{self, UiPlugin, UI},
};

fn display_collision_events(mut collision_events: EventReader<CollisionEvent>) {
    for collision_event in collision_events.iter() {
        debug!("collision event: {:?}", collision_event);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut ui: ResMut<UI>) {
    if !cfg!(target_family = "wasm") {
        asset_server.watch_for_changes().unwrap();
    }

    ui::set(&mut commands, &mut ui, |commands| {
        ui::main_menu::create(&asset_server, commands)
    })
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
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
        .add_plugin(LevelEditorPlugin)
        .add_startup_system(setup);

    if !cfg!(target_family = "wasm") {
        app.add_plugin(AtmospherePlugin);
    };

    if cfg!(debug_assertions) {
        app
            // .add_plugin(RapierDebugRenderPlugin::default())
            .add_system(display_collision_events);
    }

    app.run();
}
