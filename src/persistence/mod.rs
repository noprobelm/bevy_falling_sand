//! Save and load particles and particle type definitions to disk.
//!
//! This module provides particle type persistence, chunk persistence,  and serialization formats
//! for writing the simulation state to disk.
//!
//! ## Chunk persistence
//!
//! Automatic save/load tied to the core module's chunk loading lifecycle:
//!
//! - **On chunk unload** (`PreUpdate`, after
//!   [`ChunkSystems::Loading`](crate::ChunkSystems::Loading)):
//!   particles still present in the [`ParticleMap`](crate::ParticleMap) are
//!   serialized to [BFS](`bfs`) format and their colors rendered to a PNG. Both
//!   files are written asynchronously.
//!
//! - **On chunk load** (`PostUpdate`): if save files exist for the chunk, an async
//!   task reads and deserializes them. Loaded particles wait in a queue until all
//!   referenced [`ParticleType`](crate::ParticleType) names are registered,
//!   then spawn with restored colors from the PNG.
//!
//! - **On demand**: [`PersistChunksSignal`] saves every currently loaded chunk immediately,
//!   useful before application exit.
//!
//! ## Particle type persistence
//!
//! Save/load [`ParticleType`](crate::ParticleType) entity definitions using
//! Bevy's `DynamicScene` and RON serialization:
//!
//! - [`PersistParticleTypesSignal`] / [`LoadParticleTypesSignal`]
//! - [`ParticleTypesPersistedSignal`] / [`ParticleTypesLoadedSignal`] — confirmation signals
//!
//! ## Binary formats
//!
//! - [`bfs`] — Binary Format without color. Compresses particle positions well.
//! - [`bfc`] — Binary Format with Color. Stores per-particle position and color data at the
//!   expense of worse compression ratios.
///
/// ## Feature flags
///
/// | Feature       | Description                                           | Implies |
/// |---------------|-------------------------------------------------------|---------|
/// | `bfs`         | BFS binary format (run-length compressed, no color).  | —       |
/// | `bfc`         | BFC binary format (per-particle color data).          | `color` |
/// | `persistence` | Full persistence: chunk save/load, scenes, RON types. | `bfs`, `bfc` |
/// BFC format — Binary Format with Color.
#[cfg(feature = "bfc")]
#[cfg_attr(docsrs, doc(cfg(feature = "bfc")))]
pub mod bfc;
/// BFS format — Binary Format without color, with run-length compression.
#[cfg(feature = "bfs")]
#[cfg_attr(docsrs, doc(cfg(feature = "bfs")))]
pub mod bfs;
/// Chunk persistence — automatic save/load as chunks load and unload.
pub mod chunks;

#[cfg(any(feature = "bfs", feature = "bfc"))]
mod io_reader;
pub(crate) mod particle_types;

use bevy::prelude::*;
use std::path::PathBuf;

pub use chunks::{
    ChunkPersistenceError, ParticlePersistenceConfig, ParticlePersistenceState, PendingSaveTasks,
    PersistChunksSignal, chunk_file_path, chunk_png_path,
};
pub use particle_types::{
    LoadParticleTypesSignal, ParticleTypesLoadedSignal, ParticleTypesPersistedSignal,
    PersistParticleTypesSignal,
};

/// Plugin for particle persistence between game sessions.
///
/// Enables automatic chunk save/load, scene persistence, and particle type
/// definition persistence.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::persistence::FallingSandPersistencePlugin;
///
/// App::new()
///     .add_plugins(FallingSandPersistencePlugin::new("saves/world/chunks"))
///     .run();
/// ```
pub struct FallingSandPersistencePlugin {
    /// The path where chunk files will be saved.
    pub save_directory: PathBuf,
}

impl FallingSandPersistencePlugin {
    /// Create a new persistence plugin with the given save directory.
    #[must_use]
    pub fn new(save_directory: impl Into<PathBuf>) -> Self {
        Self {
            save_directory: save_directory.into(),
        }
    }
}

impl Plugin for FallingSandPersistencePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ParticlePersistenceConfig {
            save_path: self.save_directory.clone(),
        })
        .add_plugins((
            chunks::ChunkPersistencePlugin,
            particle_types::ParticleTypePersistencePlugin,
        ));
    }
}
