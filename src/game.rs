use bevy::prelude::*;

use crate::{controls::Controlled, world::PlayerHit};

fn reset_when_player_hits_avoid(
    mut player_hit: EventReader<PlayerHit>,
    mut query: Query<&mut Transform, With<Controlled>>,
) {
    for event in player_hit.iter() {
        if let PlayerHit::Avoid = event {
            for mut transform in &mut query {
                transform.translation = 2.0 * Vec3::Y;
            }
        }
    }
}

fn debug_player_hits_goal(mut player_hit: EventReader<PlayerHit>) {
    for _event in player_hit.iter() {
        debug!("hit goal")
    }
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(reset_when_player_hits_avoid)
            .add_system(debug_player_hits_goal);
    }
}
