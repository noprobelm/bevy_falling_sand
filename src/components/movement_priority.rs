use bevy::prelude::*;
use smallvec::{smallvec, SmallVec};

/// A particle's neighbors, represented as a nested SmallVec of IVec2. The outer vectors represent tiered "groups" of
/// priority, whereas the inner vectors are representative of relative coordinates a particle might move to. See
/// the [handle_particles](crate::handle_particles) system.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct MovementPriority(pub SmallVec<[SmallVec<[IVec2; 4]>; 8]>);

pub trait Material {
    fn into_movement_priority(&self) -> MovementPriority {
        MovementPriority(SmallVec::new())
    }
}

pub struct Wall;

impl Wall {
    pub fn new() -> Wall {
        Wall
    }
}

impl Material for Wall {}

pub struct Solid;

impl Solid {
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

pub struct MovableSolid;


impl MovableSolid {
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

pub struct Liquid {
    fluidity: usize,
}

impl Liquid {
    pub fn new(fluidity: usize) -> Liquid {
        if fluidity > 5 {
            Liquid { fluidity: 5 }
        } else {
            Liquid { fluidity }
        }
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

pub struct Gas {
    fluidity: usize,
}

impl Gas {
    pub fn new(fluidity: usize) -> Gas {
        if fluidity > 5 {
            Gas { fluidity: 5 }
        } else {
            Gas { fluidity }
        }
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
