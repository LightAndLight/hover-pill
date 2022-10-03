use bevy::prelude::*;

pub struct LoadEvent {
    pub path: String,
}

pub struct LevelEditorPlugin;

impl Plugin for LevelEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadEvent>();
    }
}
