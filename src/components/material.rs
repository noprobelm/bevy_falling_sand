//! Convenience module for inserting common types of movement priorities.

use bevy::prelude::*;
use smallvec::{smallvec, SmallVec};
use crate::components::MovementPriority;

/// A trait for defining a material type. Materials can be translated into commonly used movement priorities.
pub trait Material {
    fn into_movement_priority(&self) -> MovementPriority {
        MovementPriority(SmallVec::new())
    }
}

/// A wall, which has no movement priority.
pub struct Wall;

impl Wall {
    pub fn new() -> Wall {
        Wall
    }
}

impl Material for Wall {}

/// A solid material, which can only move downard.
pub struct Solid;

impl Solid {
    /// Creates a new Solid
    pub fn new() -> Solid {
        Solid
    }
}

impl Material for Solid {
    fn into_movement_priority(&self) -> MovementPriority {
        MovementPriority(
            smallvec![
                smallvec![IVec2::NEG_Y],
            ]
        )
    }
}

/// A movable solid material, like sand.
pub struct MovableSolid;

impl MovableSolid {
    /// Creates a new MovableSolid
    pub fn new() -> MovableSolid {
        MovableSolid
    }
}

impl Material for MovableSolid {
    fn into_movement_priority(&self) -> MovementPriority {
        MovementPriority(
            smallvec![
                smallvec![IVec2::NEG_Y],
                smallvec![IVec2::NEG_ONE, IVec2::new(1, -1)],
            ]
        )
    }
}

/// A liquid material which flows like water.
pub struct Liquid {
    /// How fluid the liquid should be.
    fluidity: usize,
}

impl Liquid {
    /// Creates a new Liquid
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
            neighbors.push(smallvec![IVec2::X * (i + 2) as i32, IVec2::NEG_X * (i + 2) as i32]);
        }

        MovementPriority(neighbors)
    }
}

/// A gaseous material, which flows upward.
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
        let mut neighbors: SmallVec<[SmallVec<[IVec2; 4]>; 8]> = smallvec![
            smallvec![IVec2::Y, IVec2::new(1, 1), IVec2::new(-1, 1)]
        ];

        for i in 0..self.fluidity {
            neighbors.push(smallvec![IVec2::X * (i + 2) as i32, IVec2::NEG_X * (i + 2) as i32]);
        }

        MovementPriority(neighbors)
    }
}
