use bevy::prelude::*;
use bfs_core::{ParticleBlueprint, impl_particle_blueprint, ParticleType};
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

pub trait Material {
    #[allow(dead_code)]
    fn into_movement_priority(&self) -> MovementPriority {
        MovementPriority::empty()
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
pub struct Wall;

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
    pub fn new() -> Wall {
        Wall
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
pub struct Solid;

impl Solid {
    pub fn new() -> Solid {
        Solid
    }
}

impl Material for Solid {
    fn into_movement_priority(&self) -> MovementPriority {
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
pub struct MovableSolid;

impl MovableSolid {
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
    pub fluidity: usize,
}

impl Liquid {
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
    pub fluidity: usize,
}

impl Gas {
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
