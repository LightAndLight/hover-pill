use bevy::prelude::*;

use crate::ui::{button, UI};

pub struct NextLevelEvent;

fn next_level_callback(commands: &mut Commands) {
    commands.add(|world: &mut World| world.send_event(NextLevelEvent))
}

pub fn display(asset_server: &AssetServer, commands: &mut Commands, ui: &mut UI) {
    super::display(commands, ui, |parent| {
        let style = TextStyle {
            font: asset_server.load("fonts/DejaVuSansMono.ttf"),
            font_size: 40.0,
            color: Color::WHITE,
        };

        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(200.0),
                        ..Default::default()
                    },
                    flex_direction: FlexDirection::ColumnReverse,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                color: Color::NONE.into(),
                ..Default::default()
            })
            .with_children(|parent| {
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: UiRect {
                                top: Val::Px(200.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        color: Color::NONE.into(),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle::from_section("complete!", style));
                    });

                parent
                    .spawn_bundle(ButtonBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: UiRect {
                                top: Val::Px(400.0),
                                ..Default::default()
                            },
                            padding: UiRect::all(Val::Px(10.0)),
                            ..Default::default()
                        },
                        color: Color::WHITE.into(),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle::from_section(
                            "next level",
                            TextStyle {
                                font: asset_server.load("fonts/DejaVuSansMono.ttf"),
                                font_size: 30.0,
                                color: Color::BLACK,
                            },
                        ));
                    })
                    .insert(button::OnClick {
                        callback: next_level_callback,
                    });
            });
    });
}

pub struct LevelCompletePlugin;

impl Plugin for LevelCompletePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NextLevelEvent>();
    }
}
