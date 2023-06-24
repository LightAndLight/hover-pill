use bevy::asset::{AssetLoader, LoadedAsset};

use super::LevelOrder;

#[derive(Default)]
pub struct LevelOrderAssetLoader;

impl AssetLoader for LevelOrderAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let level = serde_json::from_slice::<LevelOrder>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(level));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["level_order.json"]
    }
}
