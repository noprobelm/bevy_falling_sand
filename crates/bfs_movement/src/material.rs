//! Convenience module for inserting common types of movement priorities.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use smallvec::{smallvec, SmallVec};

use super::MovementPriority;

/// A trait for defining a material type. Materials can be translated into commonly used movement priorities.
pub trait Material {
    #[allow(dead_code)]
    /// Builds a new movement priority.
    fn into_movement_priority(&self) -> MovementPriority {
        MovementPriority::empty()
    }
}

/// A wall, which has no movement.
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
pub struct Wall;

impl Wall {
    /// Creates a new Wall.
    pub fn new() -> Wall {
        Wall
    }
}

impl Material for Wall {}

/// A solid material, which can only move downward.
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
pub struct Solid;

impl Solid {
    /// Creates a new Solid.
    pub fn new() -> Solid {
        Solid
    }
}

impl Material for Solid {
    fn into_movement_priority(&self) -> MovementPriority {
        MovementPriority::from(smallvec![smallvec![IVec2::NEG_Y],])
    }
}

/// A movable solid material, like sand.
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
pub struct MovableSolid;

impl MovableSolid {
    /// Creates a new MovableSolid.
    pub fn new() -> MovableSolid {
        MovableSolid
    }
}

impl Material for MovableSolid {
    fn into_movement_priority(&self) -> MovementPriority {
        MovementPriority::from(smallvec![
            smallvec![IVec2::NEG_Y],
            smallvec![IVec2::NEG_ONE, IVec2::new(1, -1)],
        ])
    }
}

/// A liquid material which flows like water.
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
pub struct Liquid {
    /// How fluid the liquid should be.
    fluidity: usize,
}

impl Liquid {
    /// Creates a new Liquid.
    pub fn new(fluidity: usize) -> Liquid {
        Liquid { fluidity }
    }
}

impl Material for Liquid {
    fn into_movement_priority(&self) -> MovementPriority {
        let mut neighbors: SmallVec<[SmallVec<[IVec2; 4]>; 8]> = smallvec![
            smallvec![IVec2::NEG_Y],
            smallvec![IVec2::NEG_ONE, IVec2::new(1, -1)],
            smallvec![IVec2::X, IVec2::NEG_X]
        ];

        for i in 0..self.fluidity {
            neighbors.push(smallvec![
                IVec2::X * (i + 2) as i32,
                IVec2::NEG_X * (i + 2) as i32
            ]);
        }

        MovementPriority::from(neighbors)
    }
}

/// A gaseous material, which flows upward.
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
pub struct Gas {
    /// How fluid the gas should be.
    fluidity: usize,
}

impl Gas {
    /// Creates a new Gas.
    pub fn new(fluidity: usize) -> Gas {
        Gas { fluidity }
    }
}

impl Material for Gas {
    fn into_movement_priority(&self) -> MovementPriority {
        let mut neighbors: SmallVec<[SmallVec<[IVec2; 4]>; 8]> =
            smallvec![smallvec![IVec2::Y, IVec2::new(1, 1), IVec2::new(-1, 1)]];

        for i in 0..self.fluidity {
            neighbors.push(smallvec![
                IVec2::X * (i + 2) as i32,
                IVec2::NEG_X * (i + 2) as i32
            ]);
        }

        MovementPriority::from(neighbors)
    }
}

/// Enum to mark different material types
#[derive(
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Reflect,
    Serialize,
    Deserialize,
)]
pub enum MaterialType {
    /// Marker for solid materials.
    Solid,
    /// Marker for movable solid materials.
    MovableSolid,
    /// Marker for liquid materials.
    Liquid,
    /// Marker for gaseous materials.
    Gas,
    /// Marker for custom materials.
    Custom
}
