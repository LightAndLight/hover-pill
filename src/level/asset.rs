use bevy::asset::{AssetLoader, LoadedAsset};

use super::Level;

#[derive(Default)]
pub struct LevelAssetLoader;

impl AssetLoader for LevelAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let level = serde_json::from_slice::<Level>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(level));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}
