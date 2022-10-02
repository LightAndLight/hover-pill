use bevy::prelude::*;

use super::button;

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
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                },
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: Color::rgb(0.4, 0.7, 1.0).into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
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
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(10.0)),
                        margin,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle::from_section(
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
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(10.0)),
                        margin,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle::from_section(
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

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MainMenuEvent>();
    }
}
