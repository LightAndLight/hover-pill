use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{controls::Controlled, level::wall};

pub enum PlayerHit {
    Avoid,
    Goal,
}

fn handle_player_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    player_query: Query<&Controlled>,
    avoid_query: Query<&wall::Avoid>,
    goal_query: Query<&wall::Goal>,
    mut player_hit: EventWriter<PlayerHit>,
) {
    for event in collision_events.iter() {
        if let CollisionEvent::Started(entity1, entity2, _) = event {
            let entities = if player_query.contains(*entity1) {
                Some((*entity1, *entity2))
            } else if player_query.contains(*entity2) {
                Some((*entity2, *entity1))
            } else {
                None
            };

            if let Some((player, target)) = entities {
                let hit = if avoid_query.contains(target) {
                    debug!("player {:?} hit avoid {:?}", player, target);
                    Some(PlayerHit::Avoid)
                } else if goal_query.contains(target) {
                    debug!("player {:?} hit goal {:?}", player, target);
                    Some(PlayerHit::Goal)
                } else {
                    None
                };

                if let Some(event) = hit {
                    player_hit.send(event);
                }
            }
        }
    }
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerHit>()
            .add_system(handle_player_collisions);
    }
}
