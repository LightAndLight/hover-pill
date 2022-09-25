use bevy::prelude::*;

use crate::fuel::FuelChanged;

#[derive(Component)]
pub struct FuelBar;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
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
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        position_type: PositionType::Absolute,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    color: Color::BLACK.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                ..default()
                            },
                            color: Color::rgb(0.4, 0.4, 1.0).into(),
                            ..default()
                        })
                        .insert(FuelBar);
                });

            parent.spawn_bundle(TextBundle::from_section(
                "fuel",
                TextStyle {
                    font: asset_server.load("fonts/DejaVuSansMono.ttf"),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            ));
        });

    make_complete_screen(&mut commands, &asset_server);
}

pub fn update_fuel_bar(
    mut fuel_changed: EventReader<FuelChanged>,
    mut query: Query<&mut Style, With<FuelBar>>,
) {
    for fuel_changed in fuel_changed.iter() {
        for mut style in &mut query {
            style.size.width = Val::Percent(fuel_changed.new_value * 100.0);
        }
    }
}

#[derive(Component)]
struct NextLevel;

pub struct NextLevelEvent;

#[derive(Component)]
pub struct CompleteScreen;

pub fn make_complete_screen(commands: &mut Commands, asset_server: &AssetServer) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(),
            visibility: Visibility { is_visible: false },
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
                    parent.spawn_bundle(TextBundle::from_section(
                        "complete!",
                        TextStyle {
                            font: asset_server.load("fonts/DejaVuSansMono.ttf"),
                            font_size: 40.0,
                            color: Color::WHITE,
                        },
                    ));
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
                .insert(NextLevel);
        })
        .insert(CompleteScreen);
}

fn handle_next_level(
    query: Query<&Interaction, (Changed<Interaction>, With<NextLevel>)>,
    mut next_level: EventWriter<NextLevelEvent>,
    mut complete_screen_query: Query<&mut Visibility, With<CompleteScreen>>,
) {
    for interaction in &query {
        if let Interaction::Clicked = interaction {
            next_level.send(NextLevelEvent);
            for mut visibility in &mut complete_screen_query {
                visibility.is_visible = false;
            }
        }
    }
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_event::<NextLevelEvent>()
            .add_system(handle_next_level)
            .add_system(update_fuel_bar);
    }
}
