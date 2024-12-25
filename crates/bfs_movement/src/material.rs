//! Convenience module for inserting common types of movement priorities.
use bevy::prelude::*;
use bfs_core::ParticleType;
use serde::{Deserialize, Serialize};
use smallvec::{smallvec, SmallVec};

use super::{MovementPriority, MovementPriorityBlueprint};

pub struct MaterialPlugin;

impl Plugin for MaterialPlugin {
    fn build(&self, app: &mut App) {
        app.observe(on_solid_blueprint_added)
            .observe(on_movable_solid_blueprint_added)
            .observe(on_liquid_blueprint_added)
            .observe(on_wall_added)
            .observe(on_gas_blueprint_added);
    }
}

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

/// A wall blueprint.
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
pub struct WallBlueprint(pub Wall);

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
        MovementPriority::from(vec![vec![IVec2::NEG_Y]])
    }
}

/// A solid blueprint
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
pub struct SolidBlueprint(pub Solid);

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
        MovementPriority::from(vec![
            vec![IVec2::NEG_Y],
            vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
        ])
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
pub struct MovableSolidBlueprint(pub MovableSolid);

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
    pub fluidity: usize,
}

impl Liquid {
    /// Creates a new Liquid.
    pub fn new(fluidity: usize) -> Liquid {
        Liquid { fluidity }
    }
}

impl Material for Liquid {
    fn into_movement_priority(&self) -> MovementPriority {
        let mut neighbors: Vec<Vec<IVec2>> = vec![
            vec![IVec2::NEG_Y],
            vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
            vec![IVec2::X, IVec2::NEG_X],
        ];

        for i in 0..self.fluidity {
            neighbors.push(vec![
                IVec2::X * (i + 2) as i32,
                IVec2::NEG_X * (i + 2) as i32,
            ]);
        }

        MovementPriority::from(neighbors)
    }
}

/// A liquid blueprint.
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
pub struct LiquidBlueprint(pub Liquid);

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
    pub fluidity: usize,
}

impl Gas {
    /// Creates a new Gas.
    pub fn new(fluidity: usize) -> Gas {
        Gas { fluidity }
    }
}

impl Material for Gas {
    fn into_movement_priority(&self) -> MovementPriority {
        let mut neighbors: Vec<Vec<IVec2>> =
            vec![vec![IVec2::Y, IVec2::new(1, 1), IVec2::new(-1, 1)]];

        for i in 0..self.fluidity {
            neighbors.push(vec![
                IVec2::X * (i + 2) as i32,
                IVec2::NEG_X * (i + 2) as i32,
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
pub struct GasBlueprint(pub Gas);

/// Enum to mark different material types
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Reflect, Serialize, Deserialize)]
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
    Custom,
}

/// Observer for adding movement priority when a particle is given a new state of matter.
pub fn on_solid_blueprint_added(
    trigger: Trigger<OnAdd, SolidBlueprint>,
    mut commands: Commands,
    particle_query: Query<&SolidBlueprint, With<ParticleType>>,
) {
    let entity = trigger.entity();
    if let Ok(solid) = particle_query.get(entity) {
        commands
            .entity(entity)
            .insert(MovementPriorityBlueprint(solid.0.into_movement_priority()));
    }
}

/// Observer for adding movement priority when a particle is given a new state of matter.
pub fn on_movable_solid_blueprint_added(
    trigger: Trigger<OnAdd, MovableSolidBlueprint>,
    mut commands: Commands,
    particle_query: Query<&MovableSolidBlueprint, With<ParticleType>>,
) {
    let entity = trigger.entity();
    if let Ok(movable_solid) = particle_query.get(entity) {
        commands.entity(entity).insert(MovementPriorityBlueprint(
            movable_solid.0.into_movement_priority(),
        ));
    }
}

/// Observer for adding movement priority when a particle is given a new state of matter.
pub fn on_liquid_blueprint_added(
    trigger: Trigger<OnAdd, LiquidBlueprint>,
    mut commands: Commands,
    particle_query: Query<&LiquidBlueprint, With<ParticleType>>,
) {
    let entity = trigger.entity();
    if let Ok(liquid) = particle_query.get(entity) {
        commands
            .entity(entity)
            .insert(MovementPriorityBlueprint(liquid.0.into_movement_priority()));
    }
}

/// Observer for adding movement priority when a particle is given a new state of matter.
pub fn on_gas_blueprint_added(
    trigger: Trigger<OnAdd, GasBlueprint>,
    mut commands: Commands,
    particle_query: Query<&GasBlueprint, With<ParticleType>>,
) {
    let entity = trigger.entity();
    if let Ok(gas) = particle_query.get(entity) {
        commands
            .entity(entity)
            .insert(MovementPriorityBlueprint(gas.0.into_movement_priority()));
    }
}

/// Observer for adding movement priority when a particle is given a new state of matter.
pub fn on_wall_added(
    trigger: Trigger<OnAdd, WallBlueprint>,
    mut commands: Commands,
    particle_query: Query<&WallBlueprint, With<ParticleType>>,
) {
    let entity = trigger.entity();
    if let Ok(gas) = particle_query.get(entity) {
        commands
            .entity(entity)
            .insert(MovementPriorityBlueprint(gas.0.into_movement_priority()));
    }
}
