//! System set definitions for the falling sand simulation.
//!
//! All core system sets are defined here to provide a single reference point for
//! understanding the simulation's scheduling structure.
//!
//! ## Schedule Overview
//!
//! **`PreUpdate`:**
//! - [`ParticleSystems::Registration`] — handles registration for new particles
//! - [`ChunkSystems::Loading`] — handles chunk loading/unloading on origin shift
//! - [`ChunkSystems::DirtyAdvance`] — advances chunk dirty state (runs before movement)
//!
//! **`PostUpdate`:**
//! - [`ParticleSystems::Simulation`] — top-level gate for all simulation systems
//! - [`ChunkSystems::Cleanup`] — drains stale particles from unloaded regions

pub use super::chunk::ChunkSystems;
pub use super::particle::schedule::ParticleSystems;
