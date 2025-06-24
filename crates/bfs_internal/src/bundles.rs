use bevy::prelude::*;
use bfs_color::{ChangesColorBlueprint, ColorProfile, ColorProfileBlueprint};
use bfs_core::ParticleType;
use bfs_movement::{
    Density, DensityBlueprint, Gas, GasBlueprint, Liquid, LiquidBlueprint, MomentumBlueprint,
    MovableSolid, MovableSolidBlueprint, MovementPriority, MovementPriorityBlueprint, Solid,
    SolidBlueprint, Velocity, VelocityBlueprint, Wall, WallBlueprint,
};
use bfs_reactions::{BurningBlueprint, BurnsBlueprint};
use serde::{Deserialize, Serialize};

#[derive(Bundle)]
pub struct ParticleBundle {
    pub colors: ColorProfileBlueprint,
    pub flows: ChangesColorBlueprint,
    pub density: DensityBlueprint,
    pub velocity: VelocityBlueprint,
    pub momentum: MomentumBlueprint,
    pub movement_priority: MovementPriorityBlueprint,
    pub burns: BurnsBlueprint,
    pub burning: BurningBlueprint,
    pub wall: WallBlueprint,
    pub solid: SolidBlueprint,
    pub movable_solid: MovableSolidBlueprint,
    pub liquid: LiquidBlueprint,
    pub gas: GasBlueprint,
}

#[derive(Bundle)]
pub struct StaticParticleTypeBundle {
    pub particle_type: ParticleType,
    pub colors: ColorProfileBlueprint,
    pub transform: Transform,
    pub visibility: Visibility,
}

impl StaticParticleTypeBundle {
    pub fn new(particle_type: ParticleType, colors: ColorProfile) -> StaticParticleTypeBundle {
        StaticParticleTypeBundle {
            particle_type,
            colors: ColorProfileBlueprint(colors),
            transform: Transform::default(),
            visibility: Visibility::default(),
        }
    }
}

#[derive(Bundle)]
pub struct DynamicParticleTypeBundle {
    pub particle_type: ParticleType,
    pub density: DensityBlueprint,
    pub velocity: VelocityBlueprint,
    pub movement_priority: MovementPriorityBlueprint,
    pub colors: ColorProfileBlueprint,
    pub transform: Transform,
    pub visibility: Visibility,
}

impl DynamicParticleTypeBundle {
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
            colors: ColorProfileBlueprint(colors),
            transform: Transform::default(),
            visibility: Visibility::default(),
        }
    }
}

#[derive(Bundle)]
pub struct MovableSolidBundle {
    pub particle_type: ParticleType,
    pub density: DensityBlueprint,
    pub velocity: VelocityBlueprint,
    pub colors: ColorProfileBlueprint,
    pub movable_solid: MovableSolidBlueprint,
}

impl MovableSolidBundle {
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
            colors: ColorProfileBlueprint(colors),
            movable_solid: MovableSolidBlueprint(MovableSolid::new()),
        }
    }
}

#[derive(Bundle)]
pub struct SolidBundle {
    pub particle_type: ParticleType,
    pub density: DensityBlueprint,
    pub velocity: VelocityBlueprint,
    pub colors: ColorProfileBlueprint,
    pub solid: SolidBlueprint,
}

impl SolidBundle {
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
            colors: ColorProfileBlueprint(colors),
            solid: SolidBlueprint(Solid::new()),
        }
    }
}

#[derive(Bundle)]
pub struct LiquidBundle {
    pub particle_type: ParticleType,
    pub density: DensityBlueprint,
    pub velocity: VelocityBlueprint,
    pub colors: ColorProfileBlueprint,
    pub liquid: LiquidBlueprint,
}

impl LiquidBundle {
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
            colors: ColorProfileBlueprint(colors),
            liquid: LiquidBlueprint(Liquid::new(fluidity)),
        }
    }
}

#[derive(Asset, TypePath, Bundle, Serialize, Deserialize)]
pub struct GasBundle {
    pub particle_type: ParticleType,
    pub density: DensityBlueprint,
    pub velocity: VelocityBlueprint,
    pub colors: ColorProfileBlueprint,
    pub gas: GasBlueprint,
}

impl GasBundle {
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
            colors: ColorProfileBlueprint(colors),
            gas: GasBlueprint(Gas::new(fluidity)),
        }
    }
}

#[derive(Bundle)]
pub struct WallBundle {
    pub particle_type: ParticleType,
    pub colors: ColorProfileBlueprint,
    pub wall: WallBlueprint,
    pub transform: Transform,
    pub visibility: Visibility,
}

impl WallBundle {
    pub fn new(particle_type: ParticleType, colors: ColorProfile) -> WallBundle {
        WallBundle {
            particle_type,
            colors: ColorProfileBlueprint(colors),
            wall: WallBlueprint(Wall),
            transform: Transform::default(),
            visibility: Visibility::default(),
        }
    }
}
