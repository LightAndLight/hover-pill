use bevy::prelude::*;

use crate::level;

use super::{button, UI};

#[derive(Default)]
struct Overlay {
    entity: Option<Entity>,
}

pub fn display(
    commands: &mut Commands,
    ui: &mut UI,
    create_children: impl FnOnce(&mut ChildBuilder),
) {
    super::update(commands, ui, |commands, entity| {
        debug!("adding overlay to ui");

        let overlay = commands
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(),
                ..Default::default()
            })
            .with_children(create_children)
            .id();

        commands.entity(entity).add_child(overlay);

        commands.insert_resource(Overlay {
            entity: Some(overlay),
        });
    });
}

fn remove(commands: &mut Commands, ui: &mut UI, overlay: &Overlay) {
    if let Some(overlay_entity) = overlay.entity {
        super::update(commands, ui, |commands, entity| {
            commands.entity(entity).remove_children(&[overlay_entity]);
        });

        commands.entity(overlay_entity).despawn_recursive();
    }
    commands.insert_resource(Overlay { entity: None });
}

pub fn display_level(
    asset_server: &AssetServer,
    commands: &mut Commands,
    ui: &mut UI,
    lines: &[String],
) {
    display(commands, ui, |parent| {
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
                for line in lines {
                    parent.spawn_bundle(TextBundle::from_section(line, style.clone()));
                }
            });

        parent
            .spawn_bundle(ButtonBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    margin: UiRect {
                        top: Val::Px(30.0),
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
                    "continue",
                    TextStyle {
                        font: asset_server.load("fonts/DejaVuSansMono.ttf"),
                        font_size: 30.0,
                        color: Color::BLACK,
                    },
                ));
            })
            .insert(button::ButtonName::Continue);
    });
}

pub fn display_complete(asset_server: &AssetServer, commands: &mut Commands, ui: &mut UI) {
    display(commands, ui, |parent| {
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

fn handle_continue(
    mut button_press: EventReader<button::ButtonPressEvent>,
    mut commands: Commands,
    mut ui: ResMut<UI>,
    overlay: Res<Overlay>,
) {
    for event in button_press.iter() {
        if let button::ButtonName::Continue = event.name {
            debug!("continue");

            remove(&mut commands, &mut ui, &overlay);
        }
    }
}

fn handle_next(
    mut commands: Commands,
    mut button_press: EventReader<button::ButtonPressEvent>,
    current_level: Res<level::CurrentLevel>,
    mut ui: ResMut<UI>,
    overlay: Res<Overlay>,
    mut output_events: EventWriter<level::LoadEvent>,
) {
    for event in button_press.iter() {
        if let button::ButtonName::Next = event.name {
            debug!("next");

            remove(&mut commands, &mut ui, &overlay);

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

pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Overlay>()
            .add_system(handle_continue)
            .add_system(handle_next);
    }
}
