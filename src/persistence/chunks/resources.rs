use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::tasks::Task;
use image::RgbaImage;
use std::path::PathBuf;

use crate::core::ChunkCoord;
use crate::persistence::bfs::ParticleData as BfsParticleData;

pub(super) struct ResourcesPlugin;

impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticlePersistenceState>()
            .init_resource::<PendingSaveTasks>()
            .init_resource::<PendingLoadTasks>();
    }
}

/// Configuration for particle persistence to disk.
///
/// # Examples
///
/// ```no_run
/// use bevy_falling_sand::persistence::ParticlePersistenceConfig;
///
/// let config = ParticlePersistenceConfig::new("saves/world/chunks");
/// ```
#[derive(Resource, Clone, Debug)]
pub struct ParticlePersistenceConfig {
    /// Base path for particle files.
    pub save_path: PathBuf,
}

impl ParticlePersistenceConfig {
    /// Create a new persistence config with the given save directory.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            save_path: path.into(),
        }
    }
}

/// Tracks particle persistence operations and errors.
///
/// Errors are drained each frame by the internal error handler after logging.
/// To inspect errors programmatically, query this resource before `PostUpdate`.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::persistence::ParticlePersistenceState;
///
/// fn check_errors(state: Res<ParticlePersistenceState>) {
///     for error in &state.errors {
///         println!("Persistence error: {error:?}");
///     }
/// }
/// ```
#[derive(Resource, Default, Debug)]
pub struct ParticlePersistenceState {
    /// Errors encountered during persistence operations.
    pub errors: Vec<ChunkPersistenceError>,
}

/// Error types for chunk persistence operations.
///
/// # Examples
///
/// ```
/// use bevy_falling_sand::core::ChunkCoord;
/// use bevy_falling_sand::persistence::ChunkPersistenceError;
///
/// let err = ChunkPersistenceError::IoError {
///     coord: ChunkCoord::new(0, 0),
///     message: "disk full".to_string(),
/// };
/// ```
#[derive(Debug, Clone)]
pub enum ChunkPersistenceError {
    /// I/O error during file operations.
    IoError {
        /// The chunk coordinate that failed.
        coord: ChunkCoord,
        /// Error message.
        message: String,
    },
    /// Error during deserialization.
    DeserializationError {
        /// The chunk coordinate that failed.
        coord: ChunkCoord,
        /// Error message.
        message: String,
    },
}

/// Tracks pending async save tasks for chunks.
///
/// Query this resource to check if save operations are still in progress,
/// which is useful for waiting on saves to complete before exiting.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::persistence::PendingSaveTasks;
///
/// fn check_saves(tasks: Res<PendingSaveTasks>) {
///     if tasks.is_empty() {
///         println!("All saves complete");
///     }
/// }
/// ```
#[derive(Resource, Default)]
pub struct PendingSaveTasks {
    pub(super) tasks: HashMap<ChunkCoord, Task<ChunkSaveResult>>,
}

impl PendingSaveTasks {
    /// Returns true if there are no pending save tasks.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Returns true if a save is pending for the given chunk coordinate.
    #[must_use]
    pub fn has_pending_save(&self, coord: ChunkCoord) -> bool {
        self.tasks.contains_key(&coord)
    }
}

/// Get the BFS file path for a chunk.
#[must_use]
pub fn chunk_file_path(config: &ParticlePersistenceConfig, coord: ChunkCoord) -> PathBuf {
    config
        .save_path
        .join(format!("chunk_{}_{}.bfs", coord.x(), coord.y()))
}

/// Get the PNG image file path for a chunk.
#[must_use]
pub fn chunk_png_path(config: &ParticlePersistenceConfig, coord: ChunkCoord) -> PathBuf {
    config
        .save_path
        .join(format!("chunk_{}_{}.png", coord.x(), coord.y()))
}

pub(super) struct ChunkSaveResult {
    pub(super) coord: ChunkCoord,
    pub(super) particle_count: usize,
    pub(super) error: Option<String>,
}

pub(super) struct ChunkImageData {
    pub(super) data: Vec<u8>,
    pub(super) width: u32,
    pub(super) height: u32,
}

pub(super) struct ChunkLoadResult {
    pub(super) coord: ChunkCoord,
    pub(super) particles: Result<Vec<BfsParticleData>, String>,
    pub(super) image: Option<RgbaImage>,
}

pub(super) struct PendingChunkData {
    pub(super) coord: ChunkCoord,
    pub(super) particles: Vec<BfsParticleData>,
    pub(super) image: Option<RgbaImage>,
}

#[derive(Resource, Default)]
pub(super) struct PendingLoadTasks {
    pub(super) tasks: HashMap<ChunkCoord, Task<ChunkLoadResult>>,
    pub(super) pending_spawn: Vec<PendingChunkData>,
    pub(super) blocked_by_save: Vec<ChunkCoord>,
}
