pub mod button;
pub mod fuel_bar;
pub mod main_menu;
pub mod overlay;

use bevy::prelude::*;

pub struct UI {
    root: Entity,
    camera: Option<Entity>,
}

impl FromWorld for UI {
    fn from_world(world: &mut World) -> Self {
        let root = world
            .spawn()
            .insert_bundle(NodeBundle {
                style: Style {
                    size: Size {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                    },
                    ..Default::default()
                },
                color: Color::NONE.into(),
                ..Default::default()
            })
            .id();

        let entity = world.spawn().insert_bundle(Camera2dBundle::default()).id();

        Self {
            root,
            camera: Some(entity),
        }
    }
}

pub fn clear(commands: &mut Commands, ui: &mut UI) {
    commands.entity(ui.root).despawn_descendants();
}

pub fn set(
    commands: &mut Commands,
    ui: &mut UI,
    create_new_ui: impl FnOnce(&mut Commands) -> Entity,
) {
    clear(commands, ui);

    let new_ui = create_new_ui(commands);
    commands.entity(ui.root).add_child(new_ui);
}

pub fn update(commands: &mut Commands, ui: &mut UI, update_ui: impl FnOnce(&mut Commands, Entity)) {
    update_ui(commands, ui.root);
}

pub fn remove_camera(commands: &mut Commands, ui: &mut UI) {
    if let Some(entity) = ui.camera {
        commands.entity(entity).despawn_recursive();
        ui.camera = None;
    }
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UI>()
            .add_plugin(main_menu::MainMenuPlugin)
            .add_plugin(overlay::OverlayPlugin)
            .add_plugin(fuel_bar::FuelBarPlugin);
    }
}
