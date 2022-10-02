use bevy::prelude::*;

use super::button;

pub struct Overlay {
    pub entity: Entity,
}

impl FromWorld for Overlay {
    fn from_world(world: &mut World) -> Self {
        let entity = world
            .spawn()
            .insert_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(),
                visibility: Visibility { is_visible: false },
                ..Default::default()
            })
            .id();

        Self { entity }
    }
}

fn spawn_continue_button(parent: &mut ChildBuilder, asset_server: &AssetServer) {
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
}

pub fn display_level_overlay(
    asset_server: &AssetServer,
    commands: &mut Commands,
    overlay: &Overlay,
    visibility_query: &mut Query<&mut Visibility>,
    lines: &[String],
) {
    let style = TextStyle {
        font: asset_server.load("fonts/DejaVuSansMono.ttf"),
        font_size: 40.0,
        color: Color::WHITE,
    };

    let mut overlay = commands.entity(overlay.entity);
    overlay.with_children(|parent| {
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

        spawn_continue_button(parent, asset_server);
    });

    let mut visibility = visibility_query.get_mut(overlay.id()).unwrap();
    visibility.is_visible = true;
}

pub fn handle_continue(
    mut button_press: EventReader<button::ButtonPressEvent>,
    mut commands: Commands,
    overlay: Res<Overlay>,
    mut visibility_query: Query<&mut Visibility>,
) {
    for event in button_press.iter() {
        if let button::ButtonName::Continue = event.name {
            let mut visibility = visibility_query.get_mut(overlay.entity).unwrap();
            visibility.is_visible = false;

            let mut overlay = commands.entity(overlay.entity);
            overlay.despawn_descendants();
        }
    }
}
