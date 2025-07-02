use bevy::prelude::*;
use bfs_color::{ChangesColor, ColorProfile, ColorProfileBlueprint};
use bfs_core::ParticleType;
use bfs_movement::{
    Density, DensityBlueprint, Gas, GasBlueprint, Liquid, LiquidBlueprint, Momentum, MovableSolid,
    MovableSolidBlueprint, MovementPriority, Solid, SolidBlueprint, Velocity, VelocityBlueprint,
    Wall, WallBlueprint,
};
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
    pub movement_priority: MovementPriority,
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
    pub particle_type: ParticleType,
    /// The Density of the particle.
    pub density: DensityBlueprint,
    /// The maximum Velocity of the particle.
    pub velocity: VelocityBlueprint,
    /// The color profile of the particle.
    pub colors: ColorProfileBlueprint,
    /// The movable solid component.
    pub movable_solid: MovableSolidBlueprint,
}

impl MovableSolidBundle {
    /// Create a new instance of movable solid bundle.
    #[must_use]
    pub fn new(
        particle_type: ParticleType,
        density: Density,
        velocity: Velocity,
        colors: ColorProfile,
    ) -> Self {
        Self {
            particle_type,
            density: DensityBlueprint(density),
            velocity: VelocityBlueprint(velocity),
            colors: ColorProfileBlueprint(colors),
            movable_solid: MovableSolidBlueprint(MovableSolid::new()),
        }
    }
}

#[derive(Clone, PartialEq, Asset, TypePath, Bundle, Serialize, Deserialize)]
/// A bundle to quickly create a solid particle.
pub struct SolidBundle {
    /// The particle type designator.
    pub particle_type: ParticleType,
    /// The Density of the particle.
    pub density: DensityBlueprint,
    /// The maximum Velocity of the particle.
    pub velocity: VelocityBlueprint,
    /// The color profile of the particle.
    pub colors: ColorProfileBlueprint,
    /// The solid component.
    pub solid: SolidBlueprint,
}

impl SolidBundle {
    /// Create a new instance of solid bundle.
    #[must_use]
    pub fn new(
        particle_type: ParticleType,
        density: Density,
        velocity: Velocity,
        colors: ColorProfile,
    ) -> Self {
        Self {
            particle_type,
            density: DensityBlueprint(density),
            velocity: VelocityBlueprint(velocity),
            colors: ColorProfileBlueprint(colors),
            solid: SolidBlueprint(Solid::new()),
        }
    }
}

#[derive(Clone, PartialEq, Asset, TypePath, Bundle, Serialize, Deserialize)]
/// A bundle to quickly create a liquid particle.
pub struct LiquidBundle {
    /// The particle type designator.
    pub particle_type: ParticleType,
    /// The Density of the particle.
    pub density: DensityBlueprint,
    /// The maximum Velocity of the particle.
    pub velocity: VelocityBlueprint,
    /// The color profile of the particle.
    pub colors: ColorProfileBlueprint,
    /// The liquid component.
    pub liquid: LiquidBlueprint,
}

impl LiquidBundle {
    /// Create a new instance of liquid bundle.
    #[must_use]
    pub fn new(
        particle_type: ParticleType,
        density: Density,
        velocity: Velocity,
        fluidity: usize,
        colors: ColorProfile,
    ) -> Self {
        Self {
            particle_type,
            density: DensityBlueprint(density),
            velocity: VelocityBlueprint(velocity),
            colors: ColorProfileBlueprint(colors),
            liquid: LiquidBlueprint(Liquid::new(fluidity)),
        }
    }
}

#[derive(Clone, Asset, TypePath, Bundle, Serialize, Deserialize)]
/// A bundle to quickly create a gas particle.
pub struct GasBundle {
    /// The particle type designator.
    pub particle_type: ParticleType,
    /// The Density of the particle.
    pub density: DensityBlueprint,
    /// The maximum Velocity of the particle.
    pub velocity: VelocityBlueprint,
    /// The color profile of the particle.
    pub colors: ColorProfileBlueprint,
    /// The gas component.
    pub gas: GasBlueprint,
}

impl GasBundle {
    /// Create a new instance of gas bundle.
    #[must_use]
    pub const fn new(
        particle_type: ParticleType,
        density: Density,
        velocity: Velocity,
        fluidity: usize,
        colors: ColorProfile,
    ) -> Self {
        Self {
            particle_type,
            density: DensityBlueprint(density),
            velocity: VelocityBlueprint(velocity),
            colors: ColorProfileBlueprint(colors),
            gas: GasBlueprint(Gas::new(fluidity)),
        }
    }
}

#[derive(Clone, Asset, TypePath, Bundle, Serialize, Deserialize)]
/// A bundle to quickly create a wall particle.
pub struct WallBundle {
    /// The particle type designator.
    pub particle_type: ParticleType,
    /// The color profile of the particle.
    pub colors: ColorProfileBlueprint,
    /// The wall component.
    pub wall: WallBlueprint,
}

impl WallBundle {
    /// Create a new instance of gas bundle.
    #[must_use]
    pub const fn new(particle_type: ParticleType, colors: ColorProfile) -> Self {
        Self {
            particle_type,
            colors: ColorProfileBlueprint(colors),
            wall: WallBlueprint(Wall),
        }
    }
}
