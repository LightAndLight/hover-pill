use bevy::prelude::*;

#[derive(Component)]
pub struct OnClick {
    pub callback: fn(&mut Commands),
}

#[derive(Clone, Copy, Component)]
pub enum ButtonName {
    Continue,
    Play,
    LevelEditor,
}

pub struct ButtonPressEvent {
    pub name: ButtonName,
}

fn handle_button_press(
    query: Query<(&Interaction, &ButtonName), Changed<Interaction>>,
    mut button_press: EventWriter<ButtonPressEvent>,
) {
    for (interaction, &name) in &query {
        if let Interaction::Clicked = interaction {
            button_press.send(ButtonPressEvent { name });
        }
    }
}

fn handle_on_click(
    mut commands: Commands,
    query: Query<(&Interaction, &OnClick), Changed<Interaction>>,
) {
    for (interaction, on_click) in &query {
        if let Interaction::Clicked = interaction {
            (on_click.callback)(&mut commands);
        }
    }
}

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ButtonPressEvent>()
            .add_system(handle_button_press)
            .add_system(handle_on_click);
    }
}
