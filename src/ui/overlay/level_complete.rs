use bevy::prelude::*;

use crate::{
    level,
    ui::{button, UI},
};

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
                    .insert(button::ButtonName::Next);
            });
    });
}

fn handle_next(
    mut commands: Commands,
    mut button_press: EventReader<button::ButtonPressEvent>,
    current_level: Res<level::CurrentLevel>,
    mut ui: ResMut<UI>,
    overlay: Res<super::Overlay>,
    mut output_events: EventWriter<level::LoadEvent>,
) {
    for event in button_press.iter() {
        if let button::ButtonName::Next = event.name {
            debug!("next");

            super::remove(&mut commands, &mut ui, &overlay);

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
}

pub struct LevelCompletePlugin;

impl Plugin for LevelCompletePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_next);
    }
}
