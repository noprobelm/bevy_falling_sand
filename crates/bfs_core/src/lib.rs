pub mod blueprint;
mod particle;
mod chunk_map;
mod particle_type;
mod common;

use bevy::prelude::*;

pub use particle::*;
pub use chunk_map::*;
pub use common::*;
pub use particle_type::*;
pub use blueprint::*;

pub struct FallingSandCorePlugin;

impl Plugin for FallingSandCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ParticlePlugin,
            ParticleTypePlugin,
            ChunkMapPlugin,
            CommonUtilitiesPlugin,
        ));
    }
}
