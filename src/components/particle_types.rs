use bevy::prelude::*;
use crate::*;

/// Possible particle types. Add a variant of this enum to an entity to spawn the corresponding type into the world.
///
/// If there is not a corresponding parent entity mapped (defined as the bundles found in the remainder of this module)
/// and mapped as part of the 'setup_particles' system, the program will panic when a child particle of that type is
/// spawned into the world
#[derive(Component, States, Resource, Reflect, Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[reflect(Component)]
pub enum ParticleType {
    #[default]
    Water,
    Oil,
    Whiskey,
    Sand,
    Wall,
    DirtWall,
    RockWall,
    DenseRockWall,
    GrassWall,
    Steam,
}

/// Bundle of common particle components for whiskey
#[derive(Bundle)]
pub struct WhiskeyBundle {
    pub colors: ParticleColors,
    pub density: Density,
    pub neighbors: Neighbors,
    pub velocity: Velocity,
    pub momentum: Momentum,
    pub name: Name,
}

impl Default for WhiskeyBundle {
    fn default() -> Self {
        WhiskeyBundle {
            colors: ParticleColors::new(vec![Color::srgba(0.84, 0.6, 0.44, 0.5)]),
            density: Density(3),
            neighbors: Neighbors(vec![
                vec![IVec2::NEG_Y],
                vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
                vec![IVec2::X, IVec2::NEG_X],
                vec![IVec2::X * 2, IVec2::NEG_X * 2],
                vec![IVec2::X * 3, IVec2::NEG_X * 3],
                vec![IVec2::X * 4, IVec2::NEG_X * 4],
                vec![IVec2::X * 5, IVec2::NEG_X * 5],
                vec![IVec2::X * 6, IVec2::NEG_X * 6],
            ]),
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
    pub neighbors: Neighbors,
    pub velocity: Velocity,
    pub momentum: Momentum,
    pub name: Name,
}

impl Default for WaterBundle {
    fn default() -> Self {
        WaterBundle {
            colors: ParticleColors::new(vec![Color::srgba(0.043, 0.5, 0.67, 0.5)]),
            density: Density(2),
            neighbors: Neighbors(vec![
                vec![IVec2::NEG_Y],
                vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
                vec![IVec2::X, IVec2::NEG_X],
                vec![IVec2::X * 2, IVec2::NEG_X * 2],
                vec![IVec2::X * 3, IVec2::NEG_X * 3],
                vec![IVec2::X * 4, IVec2::NEG_X * 4],
                vec![IVec2::X * 5, IVec2::NEG_X * 5],
                vec![IVec2::X * 6, IVec2::NEG_X * 6],
            ]),
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
    pub neighbors: Neighbors,
    velocity: Velocity,
    momentum: Momentum,
    pub name: Name,
}

impl Default for OilBundle {
    fn default() -> Self {
        OilBundle {
            colors: ParticleColors::new(vec![Color::srgba(0.16, 0.12, 0.18, 0.5)]),
            density: Density(1),
            neighbors: Neighbors(vec![
                vec![IVec2::NEG_Y],
                vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
                vec![IVec2::X, IVec2::NEG_X],
                vec![IVec2::X * 2, IVec2::NEG_X * 2],
                vec![IVec2::X * 3, IVec2::NEG_X * 3],
                vec![IVec2::X * 4, IVec2::NEG_X * 4],
            ]),
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
    pub neighbors: Neighbors,
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
            neighbors: Neighbors(vec![
                vec![IVec2::NEG_Y],
                vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
            ]),
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
    pub neighbors: Neighbors,
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
            neighbors: Neighbors(Vec::new()),
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
    pub neighbors: Neighbors,
    pub velocity: Velocity,
    pub name: Name,
}

impl Default for SteamBundle {
    fn default() -> Self {
        SteamBundle {
            colors: ParticleColors::new(vec![Color::srgba(0.78, 0.84, 0.88, 1.)]),
            density: Density(1),
            neighbors: Neighbors(vec![
                vec![IVec2::Y, IVec2::new(1, 1), IVec2::new(-1, 1)],
                vec![IVec2::X * 2, IVec2::NEG_X * 2],
                vec![IVec2::X * 3, IVec2::NEG_X * 3],
                vec![IVec2::X * 4, IVec2::NEG_X * 4],
            ]),
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
    pub neighbors: Neighbors,
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
            neighbors: Neighbors(Vec::new()),
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
    pub neighbors: Neighbors,
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
            neighbors: Neighbors(Vec::new()),
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
    pub neighbors: Neighbors,
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
            neighbors: Neighbors(Vec::new()),
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
    pub neighbors: Neighbors,
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
            neighbors: Neighbors(Vec::new()),
            anchored: Anchored,
            velocity: Velocity::new(0, 0),
            name: Name::new("Dense Rock Wall"),
        }
    }
}
