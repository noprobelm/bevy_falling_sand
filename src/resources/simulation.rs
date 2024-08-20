//! Resources related how/when the particle simulation runs.
use bevy::prelude::Resource;

/// Resource to insert for running the simulation
#[derive(Resource, Default)]
pub struct SimulationRun;

/// Resource to insert for parallel queries and batching
#[derive(Resource, Default)]
pub struct SimulationBatch;
