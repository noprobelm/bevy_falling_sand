use bevy::prelude::*;

mod particle;

pub use particle::*;

pub(super) struct AssetLoaderPlugin;

impl bevy::prelude::Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
	app.init_asset::<ParticleTypesAsset>()
        .init_asset_loader::<ParticleTypesAssetLoader>() ;
    }
}


