use bevy::prelude::*;
use bfs_color::{ChangesColor, ColorProfile};
use bfs_core::ParticleTypeId;
use bfs_movement::{Density, Gas, Liquid, Momentum, MovableSolid, Movement, Solid, Velocity, Wall};
use bfs_reactions::{Burning, Burns};
use serde::{Deserialize, Serialize};

/// A bundle used primarily to allow bulk removal of particle components from an entity.
#[doc(hidden)]
#[derive(Clone, PartialEq, Asset, TypePath, Bundle)]
pub struct ParticleBundle {
    pub colors: ColorProfile,
    pub flows: ChangesColor,
    pub density: Density,
    pub velocity: Velocity,
    pub momentum: Momentum,
    pub movement_priority: Movement,
    pub burns: Burns,
    pub burning: Burning,
    pub wall: Wall,
    pub solid: Solid,
    pub movable_solid: MovableSolid,
    pub liquid: Liquid,
    pub gas: Gas,
}

/// A bundle to quickly create a movable solid (e.g., sand) particle.
#[derive(Clone, PartialEq, Asset, TypePath, Bundle, Serialize, Deserialize)]
pub struct MovableSolidBundle {
    /// The particle type designator.
    pub particle_type: ParticleTypeId,
    /// The Density of the particle.
    pub density: Density,
    /// The maximum Velocity of the particle.
    pub velocity: Velocity,
    /// The color profile of the particle.
    pub colors: ColorProfile,
    /// The movable solid component.
    pub movable_solid: MovableSolid,
}

impl MovableSolidBundle {
    /// Create a new instance of movable solid bundle.
    #[must_use]
    pub const fn new(
        particle_type: ParticleTypeId,
        density: Density,
        velocity: Velocity,
        colors: ColorProfile,
    ) -> Self {
        Self {
            particle_type,
            density,
            velocity,
            colors,
            movable_solid: MovableSolid::new(),
        }
    }
}

#[derive(Clone, PartialEq, Asset, TypePath, Bundle, Serialize, Deserialize)]
/// A bundle to quickly create a solid particle.
pub struct SolidBundle {
    /// The particle type designator.
    pub particle_type: ParticleTypeId,
    /// The Density of the particle.
    pub density: Density,
    /// The maximum Velocity of the particle.
    pub velocity: Velocity,
    /// The color profile of the particle.
    pub colors: ColorProfile,
    /// The solid component.
    pub solid: Solid,
}

impl SolidBundle {
    /// Create a new instance of solid bundle.
    #[must_use]
    pub const fn new(
        particle_type: ParticleTypeId,
        density: Density,
        velocity: Velocity,
        colors: ColorProfile,
    ) -> Self {
        Self {
            particle_type,
            density,
            velocity,
            colors,
            solid: Solid::new(),
        }
    }
}

#[derive(Clone, PartialEq, Asset, TypePath, Bundle, Serialize, Deserialize)]
/// A bundle to quickly create a liquid particle.
pub struct LiquidBundle {
    /// The particle type designator.
    pub particle_type: ParticleTypeId,
    /// The Density of the particle.
    pub density: Density,
    /// The maximum Velocity of the particle.
    pub velocity: Velocity,
    /// The color profile of the particle.
    pub colors: ColorProfile,
    /// The liquid component.
    pub liquid: Liquid,
}

impl LiquidBundle {
    /// Create a new instance of liquid bundle.
    #[must_use]
    pub const fn new(
        particle_type: ParticleTypeId,
        density: Density,
        velocity: Velocity,
        fluidity: usize,
        colors: ColorProfile,
    ) -> Self {
        Self {
            particle_type,
            density,
            velocity,
            colors,
            liquid: Liquid::new(fluidity),
        }
    }
}

#[derive(Clone, Asset, TypePath, Bundle, Serialize, Deserialize)]
/// A bundle to quickly create a gas particle.
pub struct GasBundle {
    /// The particle type designator.
    pub particle_type: ParticleTypeId,
    /// The Density of the particle.
    pub density: Density,
    /// The maximum Velocity of the particle.
    pub velocity: Velocity,
    /// The color profile of the particle.
    pub colors: ColorProfile,
    /// The gas component.
    pub gas: Gas,
}

impl GasBundle {
    /// Create a new instance of gas bundle.
    #[must_use]
    pub const fn new(
        particle_type: ParticleTypeId,
        density: Density,
        velocity: Velocity,
        fluidity: usize,
        colors: ColorProfile,
    ) -> Self {
        Self {
            particle_type,
            density,
            velocity,
            colors,
            gas: Gas::new(fluidity),
        }
    }
}

#[derive(Clone, Asset, TypePath, Bundle, Serialize, Deserialize)]
/// A bundle to quickly create a wall particle.
pub struct WallBundle {
    /// The particle type designator.
    pub particle_type: ParticleTypeId,
    /// The color profile of the particle.
    pub colors: ColorProfile,
    /// The wall component.
    pub wall: Wall,
}

impl WallBundle {
    /// Create a new instance of gas bundle.
    #[must_use]
    pub const fn new(particle_type: ParticleTypeId, colors: ColorProfile) -> Self {
        Self {
            particle_type,
            colors,
            wall: Wall,
        }
    }
}
