#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(
    clippy::nursery,
    clippy::pedantic,
    nonstandard_style,
    rustdoc::broken_intra_doc_links,
    missing_docs
)]
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]
//! Asset loading functionality for Bevy Falling Sand particle definitions.
//! Enables serialization and deserialization of particle types from RON files.

mod particle_asset;
mod particle_data;
mod scene_asset;

use bevy::prelude::*;

pub use particle_asset::*;
pub use particle_data::*;
pub use scene_asset::*;

/// Plugin providing asset loading functionality for particle definitions.
pub struct FallingSandAssetsPlugin;

impl Plugin for FallingSandAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ParticleDefinitionsAsset>()
            .init_asset_loader::<ParticleDefinitionsAssetLoader>()
            .init_asset::<ParticleSceneAsset>()
            .init_asset_loader::<ParticleSceneAssetLoader>()
            .register_type::<ParticleData>()
            .register_type::<ParticleSceneData>()
            .add_systems(Update, load_particle_definitions);
    }
}