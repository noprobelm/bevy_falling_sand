mod state;
mod systems;

use bevy::prelude::*;

pub use state::*;

use crate::core::{ChunkDirtyState, ChunkRegion};
use crate::movement::schedule::ParticleMovementSystems;
use systems::{
    handle_movement_by_particles, par_handle_movement_by_chunks, serial_handle_movement_by_chunks,
};

pub(super) struct ProcessingPlugin;

impl Plugin for ProcessingPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MovementSystemState>()
            .add_sub_state::<ChunkIterationState>()
            .init_resource::<MovementState>()
            .register_type::<MovementSystemState>()
            .register_type::<ChunkIterationState>()
            .add_systems(OnEnter(MovementSystemState::Chunks), mark_all_chunks_dirty)
            .add_systems(
                OnEnter(MovementSystemState::Particles),
                mark_all_chunks_dirty,
            )
            .add_systems(
                OnEnter(ChunkIterationState::Parallel),
                mark_all_chunks_dirty,
            )
            .add_systems(OnEnter(ChunkIterationState::Serial), mark_all_chunks_dirty)
            .add_systems(
                PostUpdate,
                (
                    par_handle_movement_by_chunks.run_if(in_state(ChunkIterationState::Parallel)),
                    serial_handle_movement_by_chunks.run_if(in_state(ChunkIterationState::Serial)),
                    handle_movement_by_particles.run_if(in_state(MovementSystemState::Particles)),
                )
                    .in_set(ParticleMovementSystems),
            );
    }
}

fn mark_all_chunks_dirty(mut chunk_query: Query<(&ChunkRegion, &mut ChunkDirtyState)>) {
    for (region, mut dirty_state) in &mut chunk_query {
        *dirty_state = ChunkDirtyState::fully_dirty(region.region());
    }
}
