use bevy::{prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};

pub mod asset;

#[derive(Serialize, Deserialize, TypeUuid, Clone, Default)]
#[uuid = "330a713c-2dff-422f-a9a4-b59ef1239eab"]
pub struct LevelOrder(Vec<String>);

impl LevelOrder {
    pub fn next_level<'a>(&'a self, current_level_name: &str) -> Option<&'a str> {
        let current_level_index = self.0.iter().enumerate().find_map(|(index, level_name)| {
            if level_name == current_level_name {
                Some(index)
            } else {
                None
            }
        });

        current_level_index
            .and_then(|index| self.0.get(index + 1))
            .map(|level_name| level_name.as_ref())
    }
}

#[derive(Resource)]
pub struct LoadingLevelOrder {
    pub handle: Handle<LevelOrder>,
}

fn start_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    let path = "default.level_order.json";
    trace!("start_loading: {:?}", path);

    let handle = asset_server.load(path);
    commands.insert_resource(LoadingLevelOrder { handle });
}

#[derive(Resource)]
pub struct CurrentLevelOrder {
    pub handle: Handle<LevelOrder>,
    pub level_order: LevelOrder,
}

fn finish_loading(
    mut commands: Commands,
    assets: Res<Assets<LevelOrder>>,
    loading_level_order: Res<LoadingLevelOrder>,
) {
    if let Some(level_order) = assets.get(&loading_level_order.handle) {
        trace!("finish_loading");
        commands.remove_resource::<LoadingLevelOrder>();
        commands.insert_resource(CurrentLevelOrder {
            handle: loading_level_order.handle.clone(),
            level_order: level_order.clone(),
        });
    }
}

fn hotreload(
    mut commands: Commands,
    assets: Res<Assets<LevelOrder>>,
    mut asset_event: EventReader<AssetEvent<LevelOrder>>,
    current_level_order: Res<CurrentLevelOrder>,
) {
    for event in asset_event.iter() {
        if let AssetEvent::Modified {
            handle: modified_handle,
        } = event
        {
            if modified_handle == &current_level_order.handle {
                if let Some(level_order) = assets.get(&current_level_order.handle) {
                    commands.insert_resource(CurrentLevelOrder {
                        handle: current_level_order.handle.clone(),
                        level_order: level_order.clone(),
                    });
                }
            }
        }
    }
}

pub struct LevelOrderPlugin;

impl Plugin for LevelOrderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<asset::LevelOrderAssetLoader>()
            .add_asset::<LevelOrder>()
            .add_startup_system(start_loading)
            .add_system(finish_loading.run_if(resource_exists::<LoadingLevelOrder>()))
            .add_system(hotreload.run_if(resource_exists::<CurrentLevelOrder>()));
    }
}
