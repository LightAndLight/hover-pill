use bevy::prelude::*;

#[derive(Clone, Copy, Component)]
pub enum ButtonName {
    Next,
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

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ButtonPressEvent>()
            .add_system(handle_button_press);
    }
}
