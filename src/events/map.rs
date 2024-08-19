//! Events for particle map management.
use bevy::prelude::*;

/// Triggeres [on_remove_particle](crate::on_remove_particle) to remove a particle from the simulation.
#[derive(Event)]
pub struct RemoveParticle {
    /// The coordinates of the particle to remove.
    pub coordinates: IVec2
}

/// Triggeres [on_clear_chunk_map](crate::on_clear_chunk_map) to remove a particle from the simulation.
#[derive(Event)]
pub struct ClearChunkMap;
