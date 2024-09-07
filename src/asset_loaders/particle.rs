use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypePath,
};
use serde::Deserialize;
use thiserror::Error;

use crate::components::*;

/// Particle type asset
#[derive(Debug, Deserialize)]
pub struct ParticleTypeTemplate {
    particle_colors: ParticleColor,
    material_type: MaterialType,
    density: Option<Density>,
    velocity: Option<Velocity>,
    momentum: Option<Momentum>,
    randomizes_colors: Option<RandomizesColor>,
    flows_colors: Option<RandomizesColor>
}

/// Collection of particle types loaded from an asset.
#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct ParticleTypesAsset {
    /// The particle types.
    pub particle_types: Vec<ParticleTypeTemplate>
}

/// Asset loader for particle types.
#[derive(Default)]
pub struct ParticleTypesAssetLoader;

/// Possible errors that can be produced by [`ParticleTypesAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ParticleTypesAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for ParticleTypesAssetLoader {
    type Asset = ParticleTypesAsset;
    type Settings = ();
    type Error = ParticleTypesAssetLoaderError;
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
	println!("{:?}", bytes);
        Ok(ParticleTypesAsset{particle_types: vec![]})
    }

    fn extensions(&self) -> &[&str] {
        &["custom"]
    }
}

impl ParticleTypesAssetLoader {
/// Handles deserialization of a components for a given entity.
fn handle_liquid(
) {
    
}

}
