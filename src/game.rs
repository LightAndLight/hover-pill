use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    controls::Controlled,
    fuel::{add_fuel, Fuel, FuelChanged},
    level::{self, wall},
    ui::{self, main_menu::MainMenuEvent, UI},
};

fn reset_player_position(
    current_level: &level::CurrentLevel,
    query: &mut Query<&mut Transform, With<Controlled>>,
) {
    if let level::CurrentLevel::Loaded { player_start, .. } = current_level {
        for mut transform in query {
            transform.translation = *player_start;
        }
    }
}
enum PlayerHit {
    Avoid,
    Goal,
}

fn handle_player_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    player_query: Query<&Controlled>,
    wall_type_query: Query<&wall::WallType>,
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
                let hit = if let Ok(wall_type) = wall_type_query.get(target) {
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
                };

                if let Some(event) = hit {
                    player_hit.send(event);
                }
            }
        }
    }
}

fn reset_when_player_hits_avoid(
    mut player_hit: EventReader<PlayerHit>,
    current_level: Res<level::CurrentLevel>,
    mut query: Query<&mut Transform, With<Controlled>>,
) {
    for event in player_hit.iter() {
        if let PlayerHit::Avoid = event {
            reset_player_position(&current_level, &mut query);
        }
    }
}

fn handle_goal(
    mut commands: Commands,
    mut player_hit: EventReader<PlayerHit>,
    asset_server: Res<AssetServer>,
    mut ui: ResMut<UI>,
) {
    for event in player_hit.iter() {
        if let PlayerHit::Goal = event {
            debug!("player hit goal");

            ui::overlay::level_complete::display(&asset_server, &mut commands, &mut ui);
        }
    }
}

fn restart_level(
    keys: Res<Input<KeyCode>>,
    current_level: Res<level::CurrentLevel>,
    mut transform_query: Query<&mut Transform, With<Controlled>>,
    mut fuel_query: Query<&mut Fuel, With<Controlled>>,
    mut fuel_changed: EventWriter<FuelChanged>,
) {
    if keys.just_pressed(KeyCode::R) {
        reset_player_position(&current_level, &mut transform_query);

        for mut fuel in &mut fuel_query {
            let amount = 1.0 - fuel.value;
            add_fuel(&mut fuel, amount, &mut fuel_changed);
        }
    }
}

pub fn handle_main_menu(
    mut commands: Commands,
    mut input_events: EventReader<MainMenuEvent>,
    asset_server: Res<AssetServer>,
    mut ui: ResMut<UI>,
    mut output_events: EventWriter<level::LoadEvent>,
) {
    for event in input_events.iter() {
        match event {
            MainMenuEvent::Play => {
                debug!("play");

                ui::clear(&mut commands, &mut ui);
                ui::remove_camera(&mut commands, &mut ui);

                ui::set(&mut commands, &mut ui, |commands| {
                    ui::fuel_bar::create(commands, &asset_server)
                });

                output_events.send(level::LoadEvent {
                    path: "levels/tutorial_1.json".into(),
                })
            }
            MainMenuEvent::LevelEditor => {
                debug!("level editor")
            }
        }
    }
}

fn handle_next_level(
    mut commands: Commands,
    mut input_events: EventReader<ui::overlay::level_complete::NextLevelEvent>,
    current_level: Res<level::CurrentLevel>,
    mut ui: ResMut<UI>,
    overlay: Res<ui::overlay::Overlay>,
    mut output_events: EventWriter<level::LoadEvent>,
) {
    use ui::overlay::level_complete::NextLevelEvent;

    for NextLevelEvent in input_events.iter() {
        debug!("next");

        ui::overlay::remove(&mut commands, &mut ui, &overlay);

        if let level::CurrentLevel::Loaded {
            next_level: Some(next_level),
            ..
        } = current_level.as_ref()
        {
            output_events.send(level::LoadEvent {
                path: format!("levels/{}.json", next_level),
            });
        }
    }
}

fn handle_continue(
    mut input_events: EventReader<ui::overlay::level_overview::ContinueEvent>,
    mut commands: Commands,
    mut ui: ResMut<UI>,
    overlay: Res<ui::overlay::Overlay>,
) {
    use ui::overlay::level_overview::ContinueEvent;

    for ContinueEvent in input_events.iter() {
        debug!("continue");

        ui::overlay::remove(&mut commands, &mut ui, &overlay);
    }
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerHit>()
            .add_system(reset_when_player_hits_avoid)
            .add_system(handle_goal)
            .add_system(restart_level)
            .add_system(handle_main_menu)
            .add_system(handle_next_level)
            .add_system(handle_continue)
            .add_system(handle_player_collisions);
    }
}
