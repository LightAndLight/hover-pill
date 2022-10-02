use bevy::prelude::*;

use crate::ui::{button, UI};

pub fn display(asset_server: &AssetServer, commands: &mut Commands, ui: &mut UI, lines: &[String]) {
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

fn handle_continue(
    mut button_press: EventReader<button::ButtonPressEvent>,
    mut commands: Commands,
    mut ui: ResMut<UI>,
    overlay: Res<super::Overlay>,
) {
    for event in button_press.iter() {
        if let button::ButtonName::Continue = event.name {
            debug!("continue");

            super::remove(&mut commands, &mut ui, &overlay);
        }
    }
}

pub struct LevelOverviewPlugin;

impl Plugin for LevelOverviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_continue);
    }
}
