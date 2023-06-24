use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    controls::Controlled,
    pause::PauseEvent,
    reset::ResetEvent,
    ui::{self, UI},
    wall::{Wall, WallType},
};

enum PlayerHit {
    Avoid,
    Goal,
}

fn check_player_hit(
    player_query: &Query<&Controlled>,
    entity1: &Entity,
    entity2: &Entity,
    wall_query: &Query<&Wall>,
) -> Option<PlayerHit> {
    let (player, target) = if player_query.contains(*entity1) {
        Some((*entity1, *entity2))
    } else if player_query.contains(*entity2) {
        Some((*entity2, *entity1))
    } else {
        None
    }?;

    if let Ok(wall) = wall_query.get(target) {
        match wall.wall_type {
            WallType::Avoid => {
                debug!("player {:?} hit avoid {:?}", player, target);
                Some(PlayerHit::Avoid)
            }
            WallType::Goal => {
                debug!("player {:?} hit goal {:?}", player, target);
                Some(PlayerHit::Goal)
            }
            WallType::Neutral => None,
        }
    } else {
        None
    }
}

pub fn handle_player_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    check: (Query<&Controlled>, Query<&Wall>),
    mut goal: (Res<AssetServer>, Commands, ResMut<UI>),
    mut pause_event: EventWriter<PauseEvent>,
    mut reset_event: EventWriter<ResetEvent>,
) {
    for event in collision_events.iter() {
        if let CollisionEvent::Started(entity1, entity2, _) = event {
            let event = check_player_hit(&check.0, entity1, entity2, &check.1);

            if let Some(event) = event {
                match event {
                    PlayerHit::Avoid => {
                        reset_event.send(ResetEvent);
                    }
                    PlayerHit::Goal => {
                        pause_event.send(PauseEvent::Pause);
                        ui::overlay::level_complete::display(&goal.0, &mut goal.1, &mut goal.2);
                    }
                }
            }
        }
    }
}
