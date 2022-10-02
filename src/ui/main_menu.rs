use bevy::prelude::*;

use crate::game::GameState;

use super::button::{ButtonName, ButtonPressEvent};

pub fn handle_buttons(
    mut events: EventReader<ButtonPressEvent>,
    mut state: ResMut<State<GameState>>,
) {
    for event in events.iter() {
        match event.name {
            ButtonName::Play => {
                if let GameState::MainMenu = state.current() {
                    state
                        .set(GameState::Playing)
                        .unwrap_or_else(|err| panic!("{}", err));
                }
            }
            ButtonName::LevelEditor => {
                debug!("level editor clicked")
            }
            _ => {}
        }
    }
}

pub struct MainMenu {
    entities: Vec<Entity>,
}

pub fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
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

    let entities = vec![
        commands.spawn_bundle(Camera2dBundle::default()).id(),
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
                    .insert(ButtonName::Play);

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
                    .insert(ButtonName::LevelEditor);
            })
            .id(),
    ];

    commands.insert_resource(MainMenu { entities });
}

pub fn teardown(main_menu: Res<MainMenu>, mut commands: Commands) {
    for entity in &main_menu.entities {
        commands.entity(*entity).despawn_recursive();
    }

    commands.remove_resource::<MainMenu>();
}
