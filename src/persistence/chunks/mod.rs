mod load;
mod resources;
mod save;

use bevy::prelude::*;

pub use resources::{
    ChunkPersistenceError, ParticlePersistenceConfig, ParticlePersistenceState, PendingSaveTasks,
    chunk_file_path, chunk_png_path,
};
pub use save::PersistChunksSignal;

use load::LoadPlugin;
use resources::ResourcesPlugin;
use save::SavePlugin;

pub(super) struct ChunkPersistencePlugin;

impl Plugin for ChunkPersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ResourcesPlugin, SavePlugin, LoadPlugin));
    }
}
