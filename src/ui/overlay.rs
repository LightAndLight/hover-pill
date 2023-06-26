pub mod level_complete;
pub mod level_overview;

use bevy::prelude::*;

use super::UI;

#[derive(Default, Resource)]
pub struct Overlay {
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
            .spawn(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    flex_direction: FlexDirection::ColumnReverse,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(),
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

pub fn remove(commands: &mut Commands, ui: &mut UI, overlay: &Overlay) {
    if let Some(overlay_entity) = overlay.entity {
        super::update(commands, ui, |commands, entity| {
            commands.entity(entity).remove_children(&[overlay_entity]);
        });

        commands.entity(overlay_entity).despawn_recursive();
    }
    commands.insert_resource(Overlay { entity: None });
}

pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Overlay>()
            .add_plugin(level_overview::LevelOverviewPlugin)
            .add_plugin(level_complete::LevelCompletePlugin);
    }
}
