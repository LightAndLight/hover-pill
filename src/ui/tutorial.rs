use bevy::prelude::*;

use super::Overlay;

#[derive(Component)]
struct Continue;

#[derive(Component)]
struct TutorialScreen;

pub struct DisplayTutorial1;

fn display_tutorial_1(
    mut display_tutorial_1: EventReader<DisplayTutorial1>,
    overlay: Res<Overlay>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut visibility_query: Query<&mut Visibility>,
) {
    for DisplayTutorial1 in display_tutorial_1.iter() {
        let style = TextStyle {
            font: asset_server.load("fonts/DejaVuSansMono.ttf"),
            font_size: 40.0,
            color: Color::WHITE,
        };

        let mut overlay = commands.entity(overlay.entity);

        overlay
            .with_children(|parent| {
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
                        parent.spawn_bundle(TextBundle::from_section(
                            "w - move forward",
                            style.clone(),
                        ));
                        parent.spawn_bundle(TextBundle::from_section(
                            "s - move backward",
                            style.clone(),
                        ));
                        parent
                            .spawn_bundle(TextBundle::from_section("a - move left", style.clone()));
                        parent.spawn_bundle(TextBundle::from_section(
                            "d - move right",
                            style.clone(),
                        ));
                        parent.spawn_bundle(TextBundle::from_section(
                            "right click and drag - look around",
                            style.clone(),
                        ));
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
                    .insert(Continue);
            })
            .insert(TutorialScreen);

        let mut visibility = visibility_query.get_mut(overlay.id()).unwrap();
        visibility.is_visible = true;
    }
}

fn handle_continue(
    query: Query<&Interaction, (Changed<Interaction>, With<Continue>)>,
    mut commands: Commands,
    overlay: Res<Overlay>,
    mut visibility_query: Query<&mut Visibility>,
) {
    for interaction in &query {
        if let Interaction::Clicked = interaction {
            let mut visibility = visibility_query.get_mut(overlay.entity).unwrap();
            visibility.is_visible = false;

            let mut overlay = commands.entity(overlay.entity);
            overlay.remove::<TutorialScreen>();
            overlay.despawn_descendants();
        }
    }
}

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DisplayTutorial1>()
            .add_system(handle_continue)
            .add_system(display_tutorial_1);
    }
}
