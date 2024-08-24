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

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::{Density, MovementPriority, ParticleColors, Velocity};

/// Marker component for entities holding data for a unique particle type.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct ParticleType {
    /// The particle type's unique name.
    pub name: String,
}

/// Holds the particle type's name. Used to map to particle type data.
#[derive(Component, Clone, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Particle {
    /// The particle's unique name.
    pub name: String,
}

/// Convenience bundle for adding new static particle types.
#[derive(Bundle)]
pub struct StaticParticleTypeBundle {
    /// The unique identifier for the particle.
    pub particle_type: ParticleType,
    /// The particle type's colors.
    pub colors: ParticleColors,
    /// The particle type's global transform.
    pub spatial: SpatialBundle,
}

impl StaticParticleTypeBundle {
    /// Creates a new StaticParticleTypeBundle
    pub fn new(particle_type: ParticleType, colors: ParticleColors) -> StaticParticleTypeBundle {
        StaticParticleTypeBundle {
            particle_type,
            colors,
            spatial: SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        }
    }
}

/// Convenience bundle for adding new dynamic particle types.
#[derive(Bundle)]
pub struct DynamicParticleTypeBundle {
    /// The unique identifier for the particle.
    pub particle_type: ParticleType,
    /// The particle type's density.
    pub density: Density,
    /// The particle type's velocity.
    pub velocity: Velocity,
    /// The particle type's movement priority.
    pub movement_priority: MovementPriority,
    /// The particle type's colors.
    pub colors: ParticleColors,
    /// The particle type's global transform.
    pub spatial: SpatialBundle,
}

impl DynamicParticleTypeBundle {
    /// Creates a new DynamicParticleTypeBundle
    pub fn new(
        particle_type: ParticleType,
        density: Density,
        velocity: Velocity,
        movement_priority: MovementPriority,
        colors: ParticleColors,
    ) -> DynamicParticleTypeBundle {
        DynamicParticleTypeBundle {
            particle_type,
            density,
            velocity,
            movement_priority,
            colors,
            spatial: SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        }
    }
}
