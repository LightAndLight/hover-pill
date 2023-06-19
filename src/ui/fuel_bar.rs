use bevy::prelude::*;

use crate::fuel::FuelChanged;

pub fn create(commands: &mut Commands, asset_server: &AssetServer) -> Entity {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(300.0), Val::Px(30.0)),
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        position_type: PositionType::Absolute,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    background_color: Color::BLACK.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                ..default()
                            },
                            background_color: Color::rgb(0.4, 0.4, 1.0).into(),
                            ..default()
                        })
                        .insert(FuelBar);
                });

            parent.spawn(TextBundle::from_section(
                "fuel",
                TextStyle {
                    font: asset_server.load("fonts/DejaVuSansMono.ttf"),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            ));
        })
        .id()
}

#[derive(Component)]
struct FuelBar;

fn update_fuel_bar(
    mut fuel_changed: EventReader<FuelChanged>,
    mut query: Query<&mut Style, With<FuelBar>>,
) {
    for fuel_changed in fuel_changed.iter() {
        for mut style in &mut query {
            style.size.width = Val::Percent(fuel_changed.new_value * 100.0);
        }
    }
}

pub struct FuelBarPlugin;

impl Plugin for FuelBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_fuel_bar);
    }
}
