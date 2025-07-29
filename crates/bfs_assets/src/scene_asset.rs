use bevy::{
    asset::{AssetLoader, LoadContext},
    prelude::*,
};
#[allow(unused_imports)]
use futures_lite::AsyncReadExt;
use serde::{Deserialize, Serialize};

/// Particle type and position data for scene assets.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ParticleSceneData {
    /// The particle type name.
    pub particle: String,
    /// The particle position as [x, y].
    pub position: [i32; 2],
}

/// Asset representing a particle scene loaded from a RON file.
#[derive(Debug, Clone, Serialize, Deserialize, Asset, Reflect)]
pub struct ParticleSceneAsset {
    /// The particles in this scene with their positions.
    pub particles: Vec<ParticleSceneData>,
}

impl ParticleSceneAsset {
    /// Create a new particle scene asset.
    #[must_use]
    pub const fn new(particles: Vec<ParticleSceneData>) -> Self {
        Self { particles }
    }

    /// Get all particles in the scene.
    #[must_use]
    pub const fn particles(&self) -> &Vec<ParticleSceneData> {
        &self.particles
    }

    /// Get the number of particles in the scene.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.particles.len()
    }

    /// Check if the scene is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }
}

/// Asset loader for particle scenes from RON files.
#[derive(Default)]
pub struct ParticleSceneAssetLoader;

impl AssetLoader for ParticleSceneAssetLoader {
    type Asset = ParticleSceneAsset;
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

            // Parse the RON data as a ParticleScene (from bfs_scenes format)
            #[derive(Deserialize)]
            struct LegacyParticleScene {
                particles: Vec<LegacyParticleSceneData>,
            }

            #[derive(Deserialize)]
            struct LegacyParticleSceneData {
                particle: LegacyParticle,
                position: LegacyParticlePosition,
            }

            #[derive(Deserialize)]
            struct LegacyParticle {
                name: String,
            }

            #[derive(Deserialize)]
            struct LegacyParticlePosition([i32; 2]);

            let legacy_scene: LegacyParticleScene = ron::from_str(ron_str)
                .map_err(|e| anyhow::anyhow!("Failed to parse scene RON: {}", e))?;

            // Convert to new format
            let particles = legacy_scene
                .particles
                .into_iter()
                .map(|legacy_data| ParticleSceneData {
                    particle: legacy_data.particle.name,
                    position: legacy_data.position.0,
                })
                .collect();

            Ok(ParticleSceneAsset::new(particles))
        }
    }

    fn extensions(&self) -> &[&str] {
        &["ron", "scene.ron"]
    }
}
