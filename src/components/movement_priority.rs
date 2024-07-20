use bevy::prelude::*;

/// A particle's neighbors, represented as a nested vector of IVec2. The outer vectors represent tiered "groups" of
/// priority, whereas the inner vectors are representative of relative coordinates a particle might move to. See
/// the [handle_particles](crate::handle_particles) system.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct MovementPriority(pub Vec<Vec<IVec2>>);

pub trait Material {
    fn into_movement_priority(&self) -> MovementPriority {
	MovementPriority(Vec::new())
    }
}

pub struct Wall;

impl Wall {
    pub fn new() -> Wall {
	Wall
    }
}

impl Material for Wall {}

pub struct MovableSolid;

impl MovableSolid {
    pub fn new() -> MovableSolid {
	MovableSolid
    }
}

impl Material for MovableSolid {
     fn into_movement_priority(&self) -> MovementPriority {
        MovementPriority(vec![
            vec![IVec2::NEG_Y],
            vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
        ])
    }
}

pub struct Liquid {
    fluidity: usize
}

impl Liquid {
    pub fn new(fluidity: usize) -> Liquid {
	if fluidity > 5 {
	    return Liquid {fluidity: 5}
	} else {
	    return Liquid { fluidity }
	}
    }
}

impl Material for Liquid {
    fn into_movement_priority(&self) -> MovementPriority {
	let mut neighbors: Vec<Vec<IVec2>> = vec![
	    vec![IVec2::NEG_Y],
	    vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
	    vec![IVec2::X, IVec2::NEG_X]];

	for i in 0..self.fluidity {
	    neighbors.push(vec![IVec2::X * (i + 2) as i32, IVec2::NEG_X * (i + 2) as i32])
	}

	MovementPriority(neighbors)
    }}

pub struct Gas {
    fluidity: usize
}

impl Gas {
    pub fn new(fluidity: usize) -> Gas {
	if fluidity > 5 {
	    return Gas {fluidity: 5}
	} else {
	    return Gas { fluidity }
	}
    }


}

impl Material for Gas {
    fn into_movement_priority(&self) -> MovementPriority {
	let mut neighbors: Vec<Vec<IVec2>> = vec![
	    vec![IVec2::Y, IVec2::new(1, 1), IVec2::new(-1, 1)]];

	for i in 0..self.fluidity {
	    neighbors.push(vec![IVec2::X * (i + 2) as i32, IVec2::NEG_X * (i + 2) as i32])
	}

	MovementPriority(neighbors)
    }
}
