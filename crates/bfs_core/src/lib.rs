mod chunk_map;
mod common;
mod particle;

use bevy::prelude::*;

pub use chunk_map::*;
pub use common::*;
pub use particle::*;

pub struct FallingSandCorePlugin;

impl Plugin for FallingSandCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ParticlePlugin, ChunkMapPlugin, CommonPlugin));
    }
}
