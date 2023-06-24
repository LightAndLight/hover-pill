use bevy::prelude::*;

use crate::{
    level_editor, load_level, ui,
    ui::{button, UI},
    GameState,
};

pub fn create(asset_server: &AssetServer, commands: &mut Commands) -> Entity {
    let style = TextStyle {
        font: asset_server.load("fonts/DejaVuSansMono.ttf"),
        font_size: 40.0,
        color: Color::WHITE,
    };

    let margin = UiRect {
        top: Val::Px(10.0),
        bottom: Val::Px(10.0),
        ..Default::default()
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                },
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color: Color::rgb(0.4, 0.7, 1.0).into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                style: Style {
                    margin: UiRect {
                        top: Val::Px(10.0),
                        bottom: Val::Px(10.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..TextBundle::from_section("Hover Pill", style.clone())
            });

            parent
                .spawn(ButtonBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(10.0)),
                        margin,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Play",
                        TextStyle {
                            color: Color::BLACK,
                            font_size: 30.0,
                            ..style.clone()
                        },
                    ));
                })
                .insert(button::OnClick {
                    callback: play_callback,
                });

            parent
                .spawn(ButtonBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(10.0)),
                        margin,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Level Editor",
                        TextStyle {
                            color: Color::BLACK,
                            font_size: 30.0,
                            ..style
                        },
                    ));
                })
                .insert(button::OnClick {
                    callback: level_editor_callback,
                });
        })
        .id()
}

pub enum MainMenuEvent {
    Play,
    LevelEditor,
}

fn play_callback(commands: &mut Commands) {
    commands.add(|world: &mut World| world.send_event(MainMenuEvent::Play));
}

fn level_editor_callback(commands: &mut Commands) {
    commands.add(|world: &mut World| world.send_event(MainMenuEvent::LevelEditor));
}

fn handle_events(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut input_events: EventReader<MainMenuEvent>,
    mut ui: ResMut<UI>,
    mut next_state: ResMut<NextState<GameState>>,
    mut load_event: EventWriter<load_level::LoadEvent>,
    mut start_editor_event: EventWriter<level_editor::StartEvent>,
) {
    if let Some(event) = input_events.iter().last() {
        match event {
            MainMenuEvent::Play => {
                trace!("play clicked");

                ui::clear(&mut commands, &mut ui);
                ui::camera_off(&mut commands, &mut ui);

                ui::set(&mut commands, &mut ui, |commands| {
                    ui::fuel_bar::create(commands, &asset_server)
                });

                next_state.set(GameState::Playing);

                load_event.send(load_level::LoadEvent {
                    path: "levels/tutorial_1.level.json".into(),
                });
            }
            MainMenuEvent::LevelEditor => {
                trace!("level editor clicked");

                ui::clear(&mut commands, &mut ui);
                ui::camera_off(&mut commands, &mut ui);

                start_editor_event.send(level_editor::StartEvent {
                    path: "levels/tutorial_1.level.json".into(),
                })
            }
        }
    }
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MainMenuEvent>().add_system(handle_events);
    }
}
