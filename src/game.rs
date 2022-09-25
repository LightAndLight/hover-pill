use bevy::prelude::*;

use crate::{controls::Controlled, world::PlayerHitAvoid};

fn reset_when_player_hits_avoid(
    mut player_hit_avoid: EventReader<PlayerHitAvoid>,
    mut query: Query<&mut Transform, With<Controlled>>,
) {
    for PlayerHitAvoid in player_hit_avoid.iter() {
        for mut transform in &mut query {
            transform.translation = 2.0 * Vec3::Y;
        }
    }
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(reset_when_player_hits_avoid);
    }
}
