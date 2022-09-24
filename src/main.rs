use bevy::{prelude::*, winit::WinitSettings};
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_rapier3d::{prelude::*, render::RapierDebugRenderPlugin};
use learn_bevy::{
    fuel::FuelPlugin, fuel_ball::FuelBallPlugin, hover::HoverPlugin, player::PlayerPlugin,
    ui::UiPlugin, world::WorldPlugin,
};

fn display_collision_events(mut collision_events: EventReader<CollisionEvent>) {
    for collision_event in collision_events.iter() {
        debug!("collision event: {:?}", collision_event);
    }
}

fn main() {
    App::new()
        .insert_resource(WinitSettings::game())
        .add_plugins(DefaultPlugins)
        .add_plugin(AtmospherePlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(FuelPlugin)
        .add_plugin(FuelBallPlugin)
        .add_plugin(HoverPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(WorldPlugin)
        .add_system(display_collision_events)
        .run()
}
