//! Collection of particle type defintions.
//!
//! A variant of [ParticleType] should have a corresponding bundle of components, which are then mapped to each other in the [setup_particle_types](crate::setup_particles) system.
//! When a [ParticleType](ParticleType) and [Transform](bevy::transform::components::Transform) components are added to an entity, the [handle_new_particles](crate::systems::handle_new_particles)
//! system will pick it up and include it in the simulation.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::*;

/// Marker component for entities holding data for a unique particle type.
#[derive(
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Default,
    Component,
    Reflect,
    Serialize,
    Deserialize,
)]
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
    pub colors: ParticleColor,
    /// The particle type's global transform.
    pub spatial: SpatialBundle,
}

impl StaticParticleTypeBundle {
    /// Creates a new StaticParticleTypeBundle
    pub fn new(particle_type: ParticleType, colors: ParticleColor) -> StaticParticleTypeBundle {
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
    pub colors: ParticleColor,
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
        colors: ParticleColor,
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
    pub colors: ParticleColor,
    /// The MovableSolid component
    pub movable_solid: MovableSolid,
}

impl MovableSolidBundle {
    /// Creates a new MovableSolidBundle
    pub fn new(
        particle_type: ParticleType,
        density: Density,
        velocity: Velocity,
        colors: ParticleColor,
    ) -> MovableSolidBundle {
        MovableSolidBundle {
            particle_type,
            density,
            velocity,
            colors,
            movable_solid: MovableSolid::new(),
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
    pub colors: ParticleColor,
    /// The MovableSolid component
    pub solid: Solid,
}

impl SolidBundle {
    /// Creates a new SolidBundle
    pub fn new(
        particle_type: ParticleType,
        density: Density,
        velocity: Velocity,
        colors: ParticleColor,
    ) -> SolidBundle {
        SolidBundle {
            particle_type,
            density,
            velocity,
            colors,
            solid: Solid::new(),
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
    pub colors: ParticleColor,
    /// The MovableSolid component
    pub liquid: Liquid,
}

impl LiquidBundle {
    /// Creates a new LiquidBundle
    pub fn new(
        particle_type: ParticleType,
        density: Density,
        velocity: Velocity,
        fluidity: usize,
        colors: ParticleColor,
    ) -> LiquidBundle {
        LiquidBundle {
            particle_type,
            density,
            velocity,
            colors,
            liquid: Liquid::new(fluidity),
        }
    }
}

/// Convenience bundle for adding new dynamic particles in a gaseous state.
#[derive(Bundle, Serialize, Deserialize)]
pub struct GasBundle {
    /// The unique identifier for the particle.
    pub particle_type: ParticleType,
    /// The particle type's density.
    pub density: Density,
    /// The particle type's velocity.
    pub velocity: Velocity,
    /// The particle type's colors.
    pub colors: ParticleColor,
    /// The MovableSolid component
    pub gas: Gas,
}

impl GasBundle {
    /// Creates a new GasBundle
    pub fn new(
        particle_type: ParticleType,
        density: Density,
        velocity: Velocity,
        fluidity: usize,
        colors: ParticleColor,
    ) -> GasBundle {
        GasBundle {
            particle_type,
            density,
            velocity,
            colors,
            gas: Gas::new(fluidity),
        }
    }
}
