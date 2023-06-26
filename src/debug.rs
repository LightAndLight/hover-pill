use bevy::prelude::*;
use bevy_rapier3d::{prelude::CollisionEvent, render::RapierDebugRenderPlugin};

fn display_collision_events(mut collision_events: EventReader<CollisionEvent>) {
    for collision_event in collision_events.iter() {
        debug!("collision event: {:?}", collision_event);
    }
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RapierDebugRenderPlugin::default())
            .add_system(display_collision_events);
    }
}
