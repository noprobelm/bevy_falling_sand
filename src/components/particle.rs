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
use crate::components::Material;
use crate::*;

/// Marker component for particle parents.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct ParticleParent;

/// Marker component for particles.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Component)]
pub struct Particle;

#[derive(Component, Clone, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ParticleType {
    pub name: String
}

/// Bundle of common particle components for whiskey
#[derive(Bundle)]
pub struct WhiskeyBundle {
    pub colors: ParticleColors,
    pub density: Density,
    pub movement_priority: MovementPriority,
    pub velocity: Velocity,
    pub momentum: Momentum,
    pub name: Name,
}

impl Default for WhiskeyBundle {
    fn default() -> Self {
        WhiskeyBundle {
            colors: ParticleColors::new(vec![Color::srgba(0.84, 0.6, 0.44, 0.5)]),
            density: Density(3),
            movement_priority: Liquid::new(5).into_movement_priority(),
            velocity: Velocity::new(1, 3),
            momentum: Momentum(IVec2::ZERO),
            name: Name::new("Whiskey"),
        }
    }
}

/// Bundle of common particle components for water
#[derive(Bundle)]
pub struct WaterBundle {
    pub colors: ParticleColors,
    pub density: Density,
    pub neighbors: MovementPriority,
    pub velocity: Velocity,
    pub momentum: Momentum,
    pub name: Name,
}

impl Default for WaterBundle {
    fn default() -> Self {
        WaterBundle {
            colors: ParticleColors::new(vec![Color::srgba(0.043, 0.5, 0.67, 0.5)]),
            density: Density(2),
            neighbors: Liquid::new(5).into_movement_priority(),
            velocity: Velocity::new(1, 3),
            momentum: Momentum(IVec2::ZERO),
            name: Name::new("Water"),
        }
    }
}

/// Bundle of common particle components for oil
#[derive(Bundle)]
pub struct OilBundle {
    pub colors: ParticleColors,
    pub density: Density,
    pub neighbors: MovementPriority,
    velocity: Velocity,
    momentum: Momentum,
    pub name: Name,
}

impl Default for OilBundle {
    fn default() -> Self {
        OilBundle {
            colors: ParticleColors::new(vec![Color::srgba(0.16, 0.12, 0.18, 0.5)]),
            density: Density(1),
            neighbors: Liquid::new(3).into_movement_priority(),
            velocity: Velocity::new(1, 3),
            momentum: Momentum(IVec2::ZERO),
            name: Name::new("Oil"),
        }
    }
}

/// Bundle of common particle components for sand
#[derive(Bundle)]
pub struct SandBundle {
    pub colors: ParticleColors,
    pub density: Density,
    pub neighbors: MovementPriority,
    pub velocity: Velocity,
    pub momentum: Momentum,
    pub name: Name,
}

impl Default for SandBundle {
    fn default() -> Self {
        SandBundle {
            colors: ParticleColors::new(vec![
                Color::srgba(0.95, 0.88, 0.42, 1.0),
                Color::srgba(1., 0.92, 0.54, 1.),
            ]),
            density: Density(4),
            neighbors: MovableSolid::new().into_movement_priority(),
            velocity: Velocity::new(1, 3),
            momentum: Momentum(IVec2::ZERO),
            name: Name::new("Sand"),
        }
    }
}

/// Bundle of common particle components for wall
#[derive(Bundle)]
pub struct WallBundle {
    pub colors: ParticleColors,
    pub density: Density,
    pub neighbors: MovementPriority,
    pub anchored: Anchored,
    pub velocity: Velocity,
    pub name: Name,
}

impl Default for WallBundle {
    fn default() -> Self {
        WallBundle {
            colors: ParticleColors::new(vec![
                Color::srgba(0.82, 0.84, 0.83, 1.),
                Color::srgba(0.74, 0.76, 0.78, 1.),
            ]),
            density: Density(0),
            neighbors: Wall::new().into_movement_priority(),
            anchored: Anchored,
            velocity: Velocity::new(0, 0),
            name: Name::new("Wall"),
        }
    }
}

/// Bundle of common particle components for steam
#[derive(Bundle)]
pub struct SteamBundle {
    pub colors: ParticleColors,
    pub density: Density,
    pub neighbors: MovementPriority,
    pub velocity: Velocity,
    pub name: Name,
}

impl Default for SteamBundle {
    fn default() -> Self {
        SteamBundle {
            colors: ParticleColors::new(vec![Color::srgba(0.78, 0.84, 0.88, 1.)]),
            density: Density(1),
            neighbors: Gas::new(3).into_movement_priority(),
            velocity: Velocity::new(1, 1),
            name: Name::new("Steam"),
        }
    }
}

/// Bundle of common particle components for dirt wall
#[derive(Bundle)]
pub struct DirtWallBundle {
    pub colors: ParticleColors,
    pub density: Density,
    pub neighbors: MovementPriority,
    pub anchored: Anchored,
    pub velocity: Velocity,
    pub name: Name,
}

impl Default for DirtWallBundle {
    fn default() -> Self {
        DirtWallBundle {
            colors: ParticleColors::new(vec![
                Color::srgba(0.57, 0.42, 0.3, 1.),
                Color::srgba(0.45, 0.34, 0.24, 1.),
            ]),
            density: Density(0),
            neighbors: Wall::new().into_movement_priority(),
            anchored: Anchored,
            velocity: Velocity::new(0, 0),
            name: Name::new("Dirt Wall"),
        }
    }
}

/// Bundle of common particle components for grass wall
#[derive(Bundle)]
pub struct GrassWallBundle {
    pub colors: ParticleColors,
    pub density: Density,
    pub neighbors: MovementPriority,
    pub anchored: Anchored,
    pub velocity: Velocity,
    pub name: Name,
}

impl Default for GrassWallBundle {
    fn default() -> Self {
        GrassWallBundle {
            colors: ParticleColors::new(vec![
                Color::srgba(0.36, 0.53, 0.19, 1.),
                Color::srgba(0.24, 0.36, 0.13, 1.),
                Color::srgba(0.32, 0.48, 0.18, 1.),
                Color::srgba(0.36, 0.55, 0.2, 1.),
            ]),
            density: Density(0),
            neighbors: Wall::new().into_movement_priority(),
            anchored: Anchored,
            velocity: Velocity::new(0, 0),
            name: Name::new("Grass Wall"),
        }
    }
}

/// Bundle of common particle components for rock wall
#[derive(Bundle)]
pub struct RockWallBundle {
    pub colors: ParticleColors,
    pub density: Density,
    pub neighbors: MovementPriority,
    pub anchored: Anchored,
    pub velocity: Velocity,
    pub name: Name,
}

impl Default for RockWallBundle {
    fn default() -> Self {
        RockWallBundle {
            colors: ParticleColors::new(vec![
                Color::srgba(0.23, 0.2, 0.2, 1.),
                Color::srgba(0.29, 0.24, 0.24, 1.),
                Color::srgba(0.36, 0.29, 0.29, 1.),
                Color::srgba(0.4, 0.33, 0.33, 1.),
            ]),
            density: Density(0),
            neighbors: Wall::new().into_movement_priority(),
            anchored: Anchored,
            velocity: Velocity::new(0, 0),
            name: Name::new("Rock Wall"),
        }
    }
}

/// Bundle of common particle components for dense rock wall
#[derive(Bundle)]
pub struct DenseRockWallBundle {
    pub colors: ParticleColors,
    pub density: Density,
    pub neighbors: MovementPriority,
    pub anchored: Anchored,
    pub velocity: Velocity,
    pub name: Name,
}

impl Default for DenseRockWallBundle {
    fn default() -> Self {
        DenseRockWallBundle {
            colors: ParticleColors::new(vec![
                Color::srgba(0.42, 0.45, 0.55, 1.),
                Color::srgba(0.55, 0.59, 0.67, 1.),
                Color::srgba(0.7, 0.77, 0.84, 1.),
            ]),
            density: Density(0),
            neighbors: Wall::new().into_movement_priority(),
            anchored: Anchored,
            velocity: Velocity::new(0, 0),
            name: Name::new("Dense Rock Wall"),
        }
    }
}
