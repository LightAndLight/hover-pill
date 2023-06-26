use bevy::prelude::*;

use crate::ui::{button, UI};

pub struct ContinueEvent;

fn continue_callback(commands: &mut Commands) {
    commands.add(|world: &mut World| world.send_event(ContinueEvent))
}

pub fn display(asset_server: &AssetServer, commands: &mut Commands, ui: &mut UI, lines: &[String]) {
    super::display(commands, ui, |parent| {
        let style = TextStyle {
            font: asset_server.load("fonts/DejaVuSansMono.ttf"),
            font_size: 40.0,
            color: Color::WHITE,
        };

        parent
            .spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(200.0),
                        ..Default::default()
                    },
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: Color::NONE.into(),
                ..Default::default()
            })
            .with_children(|parent| {
                for line in lines {
                    parent.spawn(TextBundle::from_section(line, style.clone()));
                }
            });

        parent
            .spawn(ButtonBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    margin: UiRect {
                        top: Val::Px(30.0),
                        ..Default::default()
                    },
                    padding: UiRect::all(Val::Px(10.0)),
                    ..Default::default()
                },
                background_color: Color::WHITE.into(),
                ..Default::default()
            })
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "continue",
                    TextStyle {
                        font: asset_server.load("fonts/DejaVuSansMono.ttf"),
                        font_size: 30.0,
                        color: Color::BLACK,
                    },
                ));
            })
            .insert(button::OnClick {
                callback: continue_callback,
            });
    });
}

pub struct LevelOverviewPlugin;

impl Plugin for LevelOverviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ContinueEvent>();
    }
}
