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

use crate::*;

/// Marker component for entities holding data for a unique particle type.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct ParticleType {
    /// The particle type's unique name.
    pub name: String,
}

impl ParticleType {
    /// Creates a new ParticleType
    pub fn new(name: &str) -> ParticleType {
        ParticleType {
            name: name.to_string(),
        }
    }
}

/// Holds the particle type's name. Used to map to particle type data.
#[derive(Component, Clone, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Particle {
    /// The particle's unique name.
    pub name: String,
}

impl Particle {
    /// Creates a new Particle
    pub fn new(name: &str) -> Particle {
        Particle {
            name: name.to_string(),
        }
    }
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

/// Convenience bundle for adding new particles in a movable solid state.
#[derive(Bundle)]
pub struct MovableSolidBundle {
    /// The unique identifier for the particle.
    pub particle_type: ParticleType,
    /// The particle type's density.
    pub density: Density,
    /// The particle type's velocity.
    pub velocity: Velocity,
    /// The particle type's colors.
    pub colors: ParticleColors,
    /// The MovableSolid component
    pub movable_solid: MovableSolid,
    /// The particle type's global transform.
    pub spatial: SpatialBundle,
}

impl MovableSolidBundle {
    /// Creates a new MovableSolidBundle
    pub fn new(
        particle_type: ParticleType,
        density: Density,
        velocity: Velocity,
        colors: ParticleColors,
    ) -> MovableSolidBundle {
        MovableSolidBundle {
            particle_type,
            density,
            velocity,
            colors,
            movable_solid: MovableSolid::new(),
            spatial: SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        }
    }
}

/// Convenience bundle for adding new particles in a solid state.
#[derive(Bundle)]
pub struct SolidBundle {
    /// The unique identifier for the particle.
    pub particle_type: ParticleType,
    /// The particle type's density.
    pub density: Density,
    /// The particle type's velocity.
    pub velocity: Velocity,
    /// The particle type's colors.
    pub colors: ParticleColors,
    /// The MovableSolid component
    pub solid: Solid,
    /// The particle type's global transform.
    pub spatial: SpatialBundle,
}

impl SolidBundle {
    /// Creates a new SolidBundle
    pub fn new(
        particle_type: ParticleType,
        density: Density,
        velocity: Velocity,
        colors: ParticleColors,
    ) -> SolidBundle {
        SolidBundle {
            particle_type,
            density,
            velocity,
            colors,
            solid: Solid::new(),
            spatial: SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        }
    }
}

/// Convenience bundle for adding new particles in a liquid state.
#[derive(Bundle)]
pub struct LiquidBundle {
    /// The unique identifier for the particle.
    pub particle_type: ParticleType,
    /// The particle type's density.
    pub density: Density,
    /// The particle type's velocity.
    pub velocity: Velocity,
    /// The particle type's colors.
    pub colors: ParticleColors,
    /// The MovableSolid component
    pub liquid: Liquid,
    /// The particle type's global transform.
    pub spatial: SpatialBundle,
}

impl LiquidBundle {
    /// Creates a new LiquidBundle
    pub fn new(
        particle_type: ParticleType,
        density: Density,
        velocity: Velocity,
        fluidity: usize,
        colors: ParticleColors,
    ) -> LiquidBundle {
        LiquidBundle {
            particle_type,
            density,
            velocity,
            colors,
            liquid: Liquid::new(fluidity),
            spatial: SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        }
    }
}

/// Convenience bundle for adding new dynamic particles in a gaseous state.
#[derive(Bundle)]
pub struct GasBundle {
    /// The unique identifier for the particle.
    pub particle_type: ParticleType,
    /// The particle type's density.
    pub density: Density,
    /// The particle type's velocity.
    pub velocity: Velocity,
    /// The particle type's colors.
    pub colors: ParticleColors,
    /// The MovableSolid component
    pub gas: Gas,
    /// The particle type's global transform.
    pub spatial: SpatialBundle,
}

impl GasBundle {
    /// Creates a new GasBundle
    pub fn new(
        particle_type: ParticleType,
        density: Density,
        velocity: Velocity,
        fluidity: usize,
        colors: ParticleColors,
    ) -> GasBundle {
        GasBundle {
            particle_type,
            density,
            velocity,
            colors,
            gas: Gas::new(fluidity),
            spatial: SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        }
    }
}
