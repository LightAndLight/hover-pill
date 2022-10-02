use bevy::prelude::*;

#[derive(Component)]
pub struct OnClick {
    pub callback: fn(&mut Commands),
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
        app.add_system(handle_on_click);
    }
}
