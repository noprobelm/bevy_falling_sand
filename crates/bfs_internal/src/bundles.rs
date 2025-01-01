//! Convenience bundles for common particle configurations.
use bevy::prelude::*;
use bfs_color::*;
use bfs_core::*;
use bfs_movement::*;
use bfs_reactions::{BurningBlueprint, BurnsBlueprint};
use serde::{Deserialize, Serialize};

/// Bundle with all possible particle components (excluding ParticleType). This struct is intended
/// for stripping an existing ParticleType of its components.
#[derive(Bundle)]
pub struct ParticleBundle {
    /// The particle type's color blueprint.
    pub colors: ParticleColorBlueprint,
    /// The particle type's FlowsColor blueprint.
    pub flows: FlowsColorBlueprint,
    /// The particle type's density blueprint.
    pub density: DensityBlueprint,
    /// The particle type's velocity blueprint.
    pub velocity: VelocityBlueprint,
    /// The particle type's momentum blueprint.
    pub momentum: MomentumBlueprint,
    /// The particle type's movement priority blueprint.
    pub movement_priority: MovementPriorityBlueprint,
    /// The particle type's burns blueprint
    pub burns: BurnsBlueprint,
    /// The particle type's burning blueprint
    pub burning: BurningBlueprint,
    /// The Wall blueprint
    pub wall: WallBlueprint,
    /// The Solid blueprint
    pub solid: SolidBlueprint,
    /// The MovableSolid blueprint
    pub movable_solid: MovableSolidBlueprint,
    /// The Liquid blueprint
    pub liquid: LiquidBlueprint,
    /// The Gas blueprint
    pub gas: GasBlueprint,
}

/// Convenience bundle for adding new static particle types.
#[derive(Bundle)]
pub struct StaticParticleTypeBundle {
    /// The unique identifier for the particle.
    pub particle_type: ParticleType,
    /// The particle type's color blueprint.
    pub colors: ParticleColorBlueprint,
    /// The particle type's global transform.
    pub transform: Transform,
    /// The particle type's global visibility.
    pub visibility: Visibility,
}

impl StaticParticleTypeBundle {
    /// Creates a new StaticParticleTypeBundle
    pub fn new(particle_type: ParticleType, colors: ColorProfile) -> StaticParticleTypeBundle {
        StaticParticleTypeBundle {
            particle_type,
            colors: ParticleColorBlueprint(colors),
            transform: Transform::default(),
            visibility: Visibility::default(),
        }
    }
}

/// Convenience bundle for adding new dynamic particle types.
#[derive(Bundle)]
pub struct DynamicParticleTypeBundle {
    /// The unique identifier for the particle.
    pub particle_type: ParticleType,
    /// The particle type's density blueprint.
    pub density: DensityBlueprint,
    /// The particle type's velocity blueprint.
    pub velocity: VelocityBlueprint,
    /// The particle type's movement priority blueprint.
    pub movement_priority: MovementPriorityBlueprint,
    /// The particle type's colors blueprint.
    pub colors: ParticleColorBlueprint,
    /// The particle type's global transform.
    pub transform: Transform,
    /// The particle type's global visibility.
    pub visibility: Visibility,
}

impl DynamicParticleTypeBundle {
    /// Creates a new DynamicParticleTypeBundle
    pub fn new(
        particle_type: ParticleType,
        density: Density,
        velocity: Velocity,
        movement_priority: MovementPriority,
        colors: ColorProfile,
    ) -> DynamicParticleTypeBundle {
        DynamicParticleTypeBundle {
            particle_type,
            density: DensityBlueprint(density),
            velocity: VelocityBlueprint(velocity),
            movement_priority: MovementPriorityBlueprint(movement_priority),
            colors: ParticleColorBlueprint(colors),
            transform: Transform::default(),
            visibility: Visibility::default(),
        }
    }
}

/// Convenience bundle for adding new particles in a movable solid state.
#[derive(Bundle)]
pub struct MovableSolidBundle {
    /// The unique identifier for the particle.
    pub particle_type: ParticleType,
    /// The particle type's density blueprint.
    pub density: DensityBlueprint,
    /// The particle type's velocity blueprint.
    pub velocity: VelocityBlueprint,
    /// The particle type's colors blueprint.
    pub colors: ParticleColorBlueprint,
    /// The MovableSolid blueprint
    pub movable_solid: MovableSolidBlueprint,
}

impl MovableSolidBundle {
    /// Creates a new MovableSolidBundle
    pub fn new(
        particle_type: ParticleType,
        density: Density,
        velocity: Velocity,
        colors: ColorProfile,
    ) -> MovableSolidBundle {
        MovableSolidBundle {
            particle_type,
            density: DensityBlueprint(density),
            velocity: VelocityBlueprint(velocity),
            colors: ParticleColorBlueprint(colors),
            movable_solid: MovableSolidBlueprint(MovableSolid::new()),
        }
    }
}

/// Convenience bundle for adding new particles in a solid state.
#[derive(Bundle)]
pub struct SolidBundle {
    /// The unique identifier for the particle.
    pub particle_type: ParticleType,
    /// The particle type's density blueprint.
    pub density: DensityBlueprint,
    /// The particle type's velocity blueprint.
    pub velocity: VelocityBlueprint,
    /// The particle type's colors blueprint.
    pub colors: ParticleColorBlueprint,
    /// The Solid component blueprint.
    pub solid: SolidBlueprint,
}

impl SolidBundle {
    /// Creates a new SolidBundle
    pub fn new(
        particle_type: ParticleType,
        density: Density,
        velocity: Velocity,
        colors: ColorProfile,
    ) -> SolidBundle {
        SolidBundle {
            particle_type,
            density: DensityBlueprint(density),
            velocity: VelocityBlueprint(velocity),
            colors: ParticleColorBlueprint(colors),
            solid: SolidBlueprint(Solid::new()),
        }
    }
}

/// Convenience bundle for adding new particles in a liquid state.
#[derive(Bundle)]
pub struct LiquidBundle {
    /// The unique identifier for the particle.
    pub particle_type: ParticleType,
    /// The particle type's density blueprint.
    pub density: DensityBlueprint,
    /// The particle type's velocity blueprint.
    pub velocity: VelocityBlueprint,
    /// The particle type's colors blueprint.
    pub colors: ParticleColorBlueprint,
    /// The Liquid component blueprint.
    pub liquid: LiquidBlueprint,
}

impl LiquidBundle {
    /// Creates a new LiquidBundle
    pub fn new(
        particle_type: ParticleType,
        density: Density,
        velocity: Velocity,
        fluidity: usize,
        colors: ColorProfile,
    ) -> LiquidBundle {
        LiquidBundle {
            particle_type,
            density: DensityBlueprint(density),
            velocity: VelocityBlueprint(velocity),
            colors: ParticleColorBlueprint(colors),
            liquid: LiquidBlueprint(Liquid::new(fluidity)),
        }
    }
}

/// Convenience bundle for adding new dynamic particles in a gaseous state.
#[derive(Asset, TypePath, Bundle, Serialize, Deserialize)]
pub struct GasBundle {
    /// The unique identifier for the particle.
    pub particle_type: ParticleType,
    /// The particle type's density blueprint.
    pub density: DensityBlueprint,
    /// The particle type's velocity blueprint.
    pub velocity: VelocityBlueprint,
    /// The particle type's colors blueprint.
    pub colors: ParticleColorBlueprint,
    /// The Gas component blueprint.
    pub gas: GasBlueprint,
}

impl GasBundle {
    /// Creates a new GasBundle
    pub fn new(
        particle_type: ParticleType,
        density: Density,
        velocity: Velocity,
        fluidity: usize,
        colors: ColorProfile,
    ) -> GasBundle {
        GasBundle {
            particle_type,
            density: DensityBlueprint(density),
            velocity: VelocityBlueprint(velocity),
            colors: ParticleColorBlueprint(colors),
            gas: GasBlueprint(Gas::new(fluidity)),
        }
    }
}

/// Convenience bundle for adding new wall particle types.
#[derive(Bundle)]
pub struct WallBundle {
    /// The unique identifier for the particle.
    pub particle_type: ParticleType,
    /// The particle type's colors blueprint.
    pub colors: ParticleColorBlueprint,
    /// The Wall component blueprint.
    pub wall: WallBlueprint,
    /// The particle type's global transform.
    pub transform: Transform,
    /// The particle type's visibility
    pub visibility: Visibility,
}

impl WallBundle {
    /// Creates a new StaticParticleTypeBundle
    pub fn new(particle_type: ParticleType, colors: ColorProfile) -> WallBundle {
        WallBundle {
            particle_type,
            colors: ParticleColorBlueprint(colors),
            wall: WallBlueprint(Wall),
            transform: Transform::default(),
            visibility: Visibility::default(),
        }
    }
}
