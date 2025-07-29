use bevy::{
    asset::{AssetLoader, LoadContext},
    prelude::*,
};
#[allow(unused_imports)]
use futures_lite::AsyncReadExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{ParticleData, ParticleDefinitions};

/// Asset representing a collection of particle definitions loaded from a RON file.
#[derive(Debug, Clone, Serialize, Deserialize, Asset, Reflect)]
pub struct ParticleDefinitionsAsset {
    /// The particle definitions loaded from the asset file.
    pub definitions: ParticleDefinitions,
}

impl ParticleDefinitionsAsset {
    /// Create a new particle definitions asset.
    #[must_use]
    pub const fn new(definitions: ParticleDefinitions) -> Self {
        Self { definitions }
    }

    /// Get a particle definition by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&ParticleData> {
        self.definitions.get(name)
    }

    /// Get all particle definitions.
    #[must_use]
    pub const fn definitions(&self) -> &ParticleDefinitions {
        &self.definitions
    }

    /// Spawn all particle types from this asset into the world.
    pub fn spawn_all_particle_types(&self, commands: &mut Commands) -> Vec<Entity> {
        self.definitions
            .values()
            .map(|particle_data| particle_data.spawn_particle_type(commands))
            .collect()
    }
}

/// Asset loader for particle definitions from RON files.
#[derive(Default)]
pub struct ParticleDefinitionsAssetLoader;

impl AssetLoader for ParticleDefinitionsAssetLoader {
    type Asset = ParticleDefinitionsAsset;
    type Settings = ();
    type Error = anyhow::Error;

    #[allow(clippy::manual_async_fn)]
    fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl std::future::Future<Output = Result<Self::Asset, Self::Error>> + Send {
        async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let ron_str = std::str::from_utf8(&bytes)?;

            // Parse the RON data as a HashMap<String, ParticleData>
            let definitions: HashMap<String, ParticleData> = ron::from_str(ron_str)
                .map_err(|e| anyhow::anyhow!("Failed to parse RON: {}", e))?;

            Ok(ParticleDefinitionsAsset::new(definitions))
        }
    }

    fn extensions(&self) -> &[&str] {
        &["ron", "particle.ron"]
    }
}

/// Component that tracks a loaded particle definitions asset handle.
/// Can be used to trigger particle type spawning when the asset is loaded.
#[derive(Component, Debug, Clone)]
pub struct ParticleDefinitionsHandle {
    /// The handle to the particle definitions asset.
    pub handle: Handle<ParticleDefinitionsAsset>,
    /// Whether the particle types have been spawned from this asset.
    pub spawned: bool,
}

impl ParticleDefinitionsHandle {
    /// Create a new particle definitions handle.
    #[must_use]
    pub const fn new(handle: Handle<ParticleDefinitionsAsset>) -> Self {
        Self {
            handle,
            spawned: false,
        }
    }
}

/// System that automatically loads particle definitions when assets are ready.
pub fn load_particle_definitions(
    mut commands: Commands,
    mut handles: Query<&mut ParticleDefinitionsHandle>,
    assets: Res<Assets<ParticleDefinitionsAsset>>,
) {
    for mut handle_component in handles.iter_mut() {
        if !handle_component.spawned {
            if let Some(asset) = assets.get(&handle_component.handle) {
                info!("Loading {} particle definitions", asset.definitions.len());
                asset.spawn_all_particle_types(&mut commands);
                handle_component.spawned = true;
            }
        }
    }
}
