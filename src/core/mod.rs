//! Provides all of the core constructs required for the falling sand simulation.
//!
//! The [`mod@particle`] module manages
//! - The [`ParticleMap`] resource
//! - [`Particle`] lifecycle (spawning/despawning) and synchronization routines
//!
//! The [`mod@simulation`] module provides constructs used to control the progression of the
//! simulation.
//!
//! The [`mod@chunk`] module provides particle "chunking" controls, allowing simulation and rendering systems
//! to run more efficiently.
//!
//! The [`mod@schedule`] provides chunk and particle system sets, allowing users to order their
//! systems around the falling sand simulation.
pub mod chunk;
pub mod particle;
pub mod schedule;
pub mod simulation;
mod spatial;

use bevy::prelude::*;

use chunk::ChunkPlugin;

pub use chunk::*;
pub use particle::*;
pub use simulation::*;
pub use spatial::*;

/// The core plugin, which manages particle definitions and map setup.
pub(super) struct FallingSandCorePlugin {
    /// Width of the loaded region in world units (must be power of 2).
    pub width: u32,
    /// Height of the loaded region in world units (must be power of 2).
    pub height: u32,
    /// Size of each chunk in world units (must be power of 2).
    pub chunk_size: u32,
}

impl Default for FallingSandCorePlugin {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 1024,
            chunk_size: 32,
        }
    }
}

impl Plugin for FallingSandCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SimulationPlugin,
            ParticlePlugin {
                width: self.width,
                height: self.height,
                origin: IVec2::new(-(self.width as i32 / 2), -(self.height as i32 / 2)),
            },
            ChunkPlugin {
                chunks_wide: self.width / self.chunk_size,
                chunks_tall: self.height / self.chunk_size,
                chunk_size: self.chunk_size,
            },
        ));
    }
}
