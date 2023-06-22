use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    controls::Controlled,
    fuel::{Fuel, FuelChanged},
    game::state::GameState,
    level::wall,
    ui::{self, UI},
};

use super::{reset_player, CurrentLevel};

enum PlayerHit {
    Avoid,
    Goal,
}

fn check_player_hit(
    player_query: &Query<&Controlled>,
    entity1: &Entity,
    entity2: &Entity,
    wall_type_query: &Query<&wall::WallType>,
) -> Option<PlayerHit> {
    let (player, target) = if player_query.contains(*entity1) {
        Some((*entity1, *entity2))
    } else if player_query.contains(*entity2) {
        Some((*entity2, *entity1))
    } else {
        None
    }?;

    if let Ok(wall_type) = wall_type_query.get(target) {
        match wall_type {
            wall::WallType::Avoid => {
                debug!("player {:?} hit avoid {:?}", player, target);
                Some(PlayerHit::Avoid)
            }
            wall::WallType::Goal => {
                debug!("player {:?} hit goal {:?}", player, target);
                Some(PlayerHit::Goal)
            }
        }
    } else {
        None
    }
}

pub fn handle_player_collisions(
    mut state: ResMut<NextState<GameState>>,
    mut collision_events: EventReader<CollisionEvent>,
    check: (Query<&Controlled>, Query<&wall::WallType>),
    mut reset: (
        Res<CurrentLevel>,
        Query<(&mut Transform, &mut Fuel), With<Controlled>>,
        EventWriter<FuelChanged>,
    ),
    mut goal: (Res<AssetServer>, Commands, ResMut<UI>),
) {
    for event in collision_events.iter() {
        if let CollisionEvent::Started(entity1, entity2, _) = event {
            let event = check_player_hit(&check.0, entity1, entity2, &check.1);

            if let Some(event) = event {
                match event {
                    PlayerHit::Avoid => {
                        reset_player(&reset.0.value, &mut reset.1, &mut reset.2);
                    }
                    PlayerHit::Goal => {
                        state.set(GameState::Paused);

                        ui::overlay::level_complete::display(&goal.0, &mut goal.1, &mut goal.2);
                    }
                }
            }
        }
    }
}
