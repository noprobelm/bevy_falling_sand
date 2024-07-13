//! 
use bevy::prelude::Resource;

/// Resource to insert for pausing the simulation
#[derive(Resource)]
pub struct SimulationPause;

/// Resource to insert for parallel queries and batching
#[derive(Resource)]
pub struct SimulationBatch;
