use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    controls::Controlled,
    fuel::{add_fuel, Fuel, FuelChanged},
    level::{self, wall},
    level_editor,
    ui::{self, main_menu::MainMenuEvent, UI},
};

fn reset_player(
    current_level: &level::CurrentLevel,
    query: &mut Query<(&mut Transform, &mut Fuel), With<Controlled>>,
    fuel_changed: &mut EventWriter<FuelChanged>,
) {
    if let level::CurrentLevel::Loaded { player_start, .. } = current_level {
        for (mut transform, mut fuel) in query {
            transform.translation = *player_start;

            let amount = 1.0 - fuel.value;
            add_fuel(&mut fuel, amount, fuel_changed);
        }
    }
}
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

fn handle_player_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    check: (Query<&Controlled>, Query<&wall::WallType>),
    mut reset: (
        Res<level::CurrentLevel>,
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
                        reset_player(&reset.0, &mut reset.1, &mut reset.2);
                    }
                    PlayerHit::Goal => {
                        ui::overlay::level_complete::display(&goal.0, &mut goal.1, &mut goal.2);
                    }
                }
            }
        }
    }
}

fn restart_level(
    keys: Res<Input<KeyCode>>,
    mut reset: (
        Res<level::CurrentLevel>,
        Query<(&mut Transform, &mut Fuel), With<Controlled>>,
        EventWriter<FuelChanged>,
    ),
) {
    if keys.just_pressed(KeyCode::R) {
        reset_player(&reset.0, &mut reset.1, &mut reset.2);
    }
}

pub fn handle_main_menu(
    mut commands: Commands,
    mut input_events: EventReader<MainMenuEvent>,
    asset_server: Res<AssetServer>,
    mut ui: ResMut<UI>,
    mut load_level: EventWriter<level::LoadEvent>,
    mut editor_load_level: EventWriter<level_editor::LoadEvent>,
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

                load_level.send(level::LoadEvent {
                    path: "levels/tutorial_1.json".into(),
                })
            }
            MainMenuEvent::LevelEditor => {
                debug!("level editor");

                ui::clear(&mut commands, &mut ui);
                ui::remove_camera(&mut commands, &mut ui);

                editor_load_level.send(level_editor::LoadEvent {
                    path: "levels/tutorial_1.json".into(),
                })
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
            .add_system(restart_level)
            .add_system(handle_main_menu)
            .add_system(handle_next_level)
            .add_system(handle_continue)
            .add_system(handle_player_collisions);
    }
}
