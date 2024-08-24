//! Collection of particle type defintions.
//!
//! A variant of [ParticleType] should have a corresponding bundle of components, which are then mapped to each other in the [setup_particle_types](crate::setup_particles) system.
//! When a [ParticleType](ParticleType) and [Transform](bevy::transform::components::Transform) components are added to an entity, the [handle_new_particles](crate::systems::handle_new_particles)
//! system will pick it up and include it in the simulation.
//!
//! A particle bundle should, at a minimum, possess these components. Any particle without these components will result in undefined behavior (likely a panic):
//! - `density`: [Density](crate::Density): A particle's "weight" when being evaluated for movement with neighbors.
//! - `neighbors`: [`Vec<Vec<MovementPriority>>`](crate::MovementPriority): A nested sequence of neighbors to consider for particle movement.
//! - `velocity`: [Velocity]: Measures the number of times (and maximum) a particle should move in a given frame.
//!
//! Optionally, a particle can possess these components:
//! - `colors`: `Vec<Color>`: A sequence of colors, one of which will be assigned to a child particle at random for rendering.
//! - `momentum`: [Momentum]: If a particle is capable of gaining momentum, it should be included in its bundle. Any starting value is valid, though Momentum::ZERO is recommended.
//! - `anchored`: [Anchored]: If a particle should not be evaluated, and block the movement of all other particles (e.g., a 'wall'), it should have this component.
//! - `name`: [Name]: Can be used for organizing data if `bevy_reflect` being used.

use serde::{Serialize, Deserialize};
use bevy::prelude::*;

/// Marker component for particle parents.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct ParticleParent;

/// Holds the particle type's name. Used to map to parent particle data.
#[derive(Component, Clone, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Particle {
    /// The particle's unique name.
    pub name: String
}
