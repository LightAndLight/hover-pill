use bevy::prelude::Resource;

#[derive(Resource)]
pub struct Config {
    pub asset_dir: String,
}
