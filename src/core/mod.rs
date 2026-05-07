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

use bevy::{ecs::system::SystemParam, prelude::*};

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

/// Read-only access to dirty particles across every chunk.
///
/// Use this when a system only needs to inspect dirty particles. Use
/// [`ParticleChunksMut`] when the system also needs mutable access to the next-frame
/// [`ChunkDirtyState`].
#[derive(SystemParam)]
pub struct ParticleChunks<'w, 's> {
    particle_map: Res<'w, ParticleMap>,
    chunk_index: Res<'w, ChunkIndex>,
    dirty_chunks: Query<'w, 's, &'static ChunkDirtyState>,
}

impl ParticleChunks<'_, '_> {
    /// Returns the [`ParticleMap`] for lookups outside the dirty-rect walk.
    #[must_use]
    pub fn map(&self) -> &ParticleMap {
        &self.particle_map
    }

    /// Returns an iterator over every dirty particle in the world as `(position, entity)`.
    pub fn dirty_particles(&self) -> impl Iterator<Item = (IVec2, Entity)> + '_ {
        let map = &*self.particle_map;
        let dirty_chunks = &self.dirty_chunks;
        self.chunk_index
            .iter()
            .filter_map(move |(_, chunk_entity)| {
                dirty_chunks.get(chunk_entity).ok().and_then(|s| s.current)
            })
            .flat_map(move |rect| {
                (rect.min.y..=rect.max.y).flat_map(move |y| {
                    (rect.min.x..=rect.max.x).filter_map(move |x| {
                        let pos = IVec2::new(x, y);
                        map.get_copied(pos).ok().flatten().map(|e| (pos, e))
                    })
                })
            })
    }

    /// Invokes `f` for every dirty particle in the world.
    ///
    /// The closure receives the [`ParticleMap`] (for neighborhood lookups), the world-space
    /// position, and the particle [`Entity`]. Closure-form counterpart to
    /// [`dirty_particles`](Self::dirty_particles); use this when callers want to capture mutable
    /// state across iterations without the borrow gymnastics an iterator chain would require.
    pub fn for_each_dirty_particle(&self, mut f: impl FnMut(&ParticleMap, IVec2, Entity)) {
        for (_, chunk_entity) in self.chunk_index.iter() {
            let Ok(dirty_state) = self.dirty_chunks.get(chunk_entity) else {
                continue;
            };

            let Some(dirty_rect) = dirty_state.current else {
                continue;
            };

            for y in dirty_rect.min.y..=dirty_rect.max.y {
                for x in dirty_rect.min.x..=dirty_rect.max.x {
                    let pos = IVec2::new(x, y);

                    let Ok(Some(entity)) = self.particle_map.get_copied(pos) else {
                        continue;
                    };

                    f(&self.particle_map, pos, entity);
                }
            }
        }
    }
}

/// Bundles the resources needed to walk every particle in every chunk's current dirty rect.
///
/// This `SystemParam` helps reduce boilerplate by providing access to every dirty particle's
/// `Entity` id, as well as its associated `Mut<'_, ChunkDirtyState>` in the event the user wants to
/// mark a position as dirty.
#[derive(SystemParam)]
pub struct ParticleChunksMut<'w, 's> {
    particle_map: Res<'w, ParticleMap>,
    chunk_index: Res<'w, ChunkIndex>,
    dirty_chunks: Query<'w, 's, &'static mut ChunkDirtyState>,
}

impl ParticleChunksMut<'_, '_> {
    /// Returns the [`ParticleMap`] for lookups outside the dirty-rect walk.
    #[must_use]
    pub fn map(&self) -> &ParticleMap {
        &self.particle_map
    }

    /// Returns an iterator over every dirty particle in the world as `(position, entity)`.
    ///
    /// Read-only counterpart to [`for_each_dirty_particle`](Self::for_each_dirty_particle); use
    /// this when callers don't need to mutate [`ChunkDirtyState`].
    pub fn dirty_particles(&self) -> impl Iterator<Item = (IVec2, Entity)> + '_ {
        let map = &*self.particle_map;
        let dirty_chunks = &self.dirty_chunks;
        self.chunk_index
            .iter()
            .filter_map(move |(_, chunk_entity)| {
                dirty_chunks.get(chunk_entity).ok().and_then(|s| s.current)
            })
            .flat_map(move |rect| {
                (rect.min.y..=rect.max.y).flat_map(move |y| {
                    (rect.min.x..=rect.max.x).filter_map(move |x| {
                        let pos = IVec2::new(x, y);
                        map.get_copied(pos).ok().flatten().map(|e| (pos, e))
                    })
                })
            })
    }

    /// Invokes `f` for every particle inside the current frame's dirty rect of every chunk.
    ///
    /// The closure receives the [`ParticleMap`] (for neighborhood lookups), a mutable handle to
    /// the chunk's [`ChunkDirtyState`] (so callers can extend the next-frame dirty region), the
    /// world-space position, and the particle [`Entity`].
    pub fn for_each_dirty_particle(
        &mut self,
        mut f: impl FnMut(&ParticleMap, &mut ChunkDirtyState, IVec2, Entity),
    ) {
        for (_, chunk_entity) in self.chunk_index.iter() {
            let Ok(mut dirty_state) = self.dirty_chunks.get_mut(chunk_entity) else {
                continue;
            };

            let Some(dirty_rect) = dirty_state.current else {
                continue;
            };

            for y in dirty_rect.min.y..=dirty_rect.max.y {
                for x in dirty_rect.min.x..=dirty_rect.max.x {
                    let pos = IVec2::new(x, y);

                    let Ok(Some(entity)) = self.particle_map.get_copied(pos) else {
                        continue;
                    };

                    f(&self.particle_map, &mut dirty_state, pos, entity);
                }
            }
        }
    }
}
