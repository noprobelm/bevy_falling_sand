use bevy::prelude::*;
use bfs_core::{impl_particle_blueprint, ParticleComponent, ParticleType};
use serde::{Deserialize, Serialize};

use super::{MovementPriority, MovementPriorityBlueprint};

pub(super) struct MaterialPlugin;

impl Plugin for MaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_solid_blueprint_added)
            .add_observer(on_movable_solid_blueprint_added)
            .add_observer(on_liquid_blueprint_added)
            .add_observer(on_wall_added)
            .add_observer(on_gas_blueprint_added);
    }
}

impl_particle_blueprint!(WallBlueprint, Wall);
impl_particle_blueprint!(SolidBlueprint, Solid);
impl_particle_blueprint!(MovableSolidBlueprint, MovableSolid);
impl_particle_blueprint!(LiquidBlueprint, Liquid);
impl_particle_blueprint!(GasBlueprint, Gas);

/// Used to describe a Material, which can be translated to a [`MovementPriority`].
pub trait Material {
    #[allow(dead_code)]
    /// Get the [`MovementPriority`] for the material type.
    fn to_movement_priority(&self) -> MovementPriority {
        MovementPriority::empty()
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Component)]
/// A simple wall, which has no movement to it.
pub struct Wall;

/// Blueprint for a [`Wall`]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Component)]
pub struct WallBlueprint(pub Wall);

impl Wall {
    /// Initialize a new `Wall`
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Material for Wall {}

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
/// A solid particle, which can move only downwards.
pub struct Solid;

impl Solid {
    /// Initialize a new `Solid`
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Material for Solid {
    fn to_movement_priority(&self) -> MovementPriority {
        MovementPriority::from(vec![vec![IVec2::NEG_Y]])
    }
}

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
/// Blueprint for a [`Solid`]
pub struct SolidBlueprint(pub Solid);

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
/// A movable solid particle, which can move downwards and diagonally.
pub struct MovableSolid;

impl MovableSolid {
    /// Initialize a new `MovableSolid`
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Material for MovableSolid {
    fn to_movement_priority(&self) -> MovementPriority {
        MovementPriority::from(vec![
            vec![IVec2::NEG_Y],
            vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
        ])
    }
}

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
/// Blueprint for a [`MovableSolid`]
pub struct MovableSolidBlueprint(pub MovableSolid);

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
/// A liquid particle, which can move downwards, diagonally, and horizontally.
///
/// A liquid particle will first attempt to move downards, then downwards diagonally. If no valid
/// positions are found, it will attempt to move horizontally n spaces as a function of its fluidity
/// + 1.
pub struct Liquid {
    /// The fluidity of a liquid. Higher values equate to more fluid-like behavior.
    pub fluidity: usize,
}

impl Liquid {
    /// Initialize a new `Liquid` with a specified fluidity.
    #[must_use]
    pub const fn new(fluidity: usize) -> Self {
        Self { fluidity }
    }
}

impl Material for Liquid {
    fn to_movement_priority(&self) -> MovementPriority {
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
/// Blueprint for a [`Liquid`]
pub struct LiquidBlueprint(pub Liquid);

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
/// A gas particle, which can move upwards, upwards diagonally, and horizontally.
pub struct Gas {
    /// The fluidity of the gas. Higher values equate to more fluid-like behavior.
    pub fluidity: usize,
}

impl Gas {
    /// Initialize a new `Gas` with a specified fluidity.
    #[must_use]
    pub const fn new(fluidity: usize) -> Self {
        Self { fluidity }
    }
}

impl Material for Gas {
    fn to_movement_priority(&self) -> MovementPriority {
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
/// Blueprint for a [`Gas`]
pub struct GasBlueprint(pub Gas);

pub fn on_solid_blueprint_added(
    trigger: Trigger<OnAdd, SolidBlueprint>,
    mut commands: Commands,
    particle_query: Query<&SolidBlueprint, With<ParticleType>>,
) {
    let entity = trigger.target();
    if let Ok(solid) = particle_query.get(entity) {
        commands
            .entity(entity)
            .insert(MovementPriorityBlueprint(solid.0.to_movement_priority()));
    }
}

pub fn on_movable_solid_blueprint_added(
    trigger: Trigger<OnAdd, MovableSolidBlueprint>,
    mut commands: Commands,
    particle_query: Query<&MovableSolidBlueprint, With<ParticleType>>,
) {
    let entity = trigger.target();
    if let Ok(movable_solid) = particle_query.get(entity) {
        commands.entity(entity).insert(MovementPriorityBlueprint(
            movable_solid.0.to_movement_priority(),
        ));
    }
}

pub fn on_liquid_blueprint_added(
    trigger: Trigger<OnAdd, LiquidBlueprint>,
    mut commands: Commands,
    particle_query: Query<&LiquidBlueprint, With<ParticleType>>,
) {
    let entity = trigger.target();
    if let Ok(liquid) = particle_query.get(entity) {
        commands
            .entity(entity)
            .insert(MovementPriorityBlueprint(liquid.0.to_movement_priority()));
    }
}

pub fn on_gas_blueprint_added(
    trigger: Trigger<OnAdd, GasBlueprint>,
    mut commands: Commands,
    particle_query: Query<&GasBlueprint, With<ParticleType>>,
) {
    let entity = trigger.target();
    if let Ok(gas) = particle_query.get(entity) {
        commands
            .entity(entity)
            .insert(MovementPriorityBlueprint(gas.0.to_movement_priority()));
    }
}

pub fn on_wall_added(
    trigger: Trigger<OnAdd, WallBlueprint>,
    mut commands: Commands,
    particle_query: Query<&WallBlueprint, With<ParticleType>>,
) {
    let entity = trigger.target();
    if let Ok(gas) = particle_query.get(entity) {
        commands
            .entity(entity)
            .insert(MovementPriorityBlueprint(gas.0.to_movement_priority()));
    }
}
