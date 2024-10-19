//! Provides core functionality for `bevy_falling_sand`. The types exposed in this crate are
//! typically necessary for extending the functionality of the particle simulation, such as:
//!   - Basic Particle type definitions
//!   - Particle spatial mapping data structures
//!   - System sets
//!   - Particle mutation/reset events

#![forbid(missing_docs)]
#![warn(
    clippy::nursery,
    clippy::pedantic,
    nonstandard_style,
    rustdoc::broken_intra_doc_links
)]
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]

//! This crate provides core functionality for particles.

mod particle;
mod map;

use bevy::prelude::*;

pub use particle::*;
pub use map::*;

/// Core plugin for Bevy Falling Sand.
pub struct FallingSandCorePlugin;

impl Plugin for FallingSandCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ParticlePlugin, ChunkMapPlugin));
        app.init_resource::<SimulationRun>();
    }
}
/// Resource to insert for running the simulation
#[derive(Resource, Default)]
pub struct SimulationRun;

/// System set for systems that influence particle management.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleSimulationSet;

/// System set for systems that provide debugging functionality.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleDebugSet;
