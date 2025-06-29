use bevy::prelude::*;
use bfs_core::{
    impl_particle_blueprint, ClearParticleTypeChildrenEvent, ParticleComponent, ParticleType,
};
use serde::{Deserialize, Serialize};

use super::{MovementPriority, MovementPriorityBlueprint};

pub(super) struct MaterialPlugin;

impl Plugin for MaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ParticleMaterialTransitionEvent>()
            .add_event::<ClearDynamicParticlesEvent>()
            .add_event::<ClearStaticParticlesEvent>()
            .add_observer(on_solid_blueprint_added)
            .add_observer(on_movable_solid_blueprint_added)
            .add_observer(on_liquid_blueprint_added)
            .add_observer(on_wall_added)
            .add_observer(on_gas_blueprint_added)
            .add_observer(on_clear_dynamic_particles)
            .add_observer(on_clear_static_particles);
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

/// A simple wall, which has no movement to it.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Component)]
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

/// A solid particle, which can move only downwards.
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

/// Blueprint for a [`Solid`]
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

/// A movable solid particle, which can move downwards and diagonally.
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

/// Blueprint for a [`MovableSolid`]
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

/// A liquid particle, which can move downwards, diagonally, and horizontally.
///
/// A liquid particle will first attempt to move downards, then downwards diagonally. If no valid
/// positions are found, it will attempt to move horizontally n spaces as a function of its fluidity
/// + 1.
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

/// Blueprint for a [`Liquid`]
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

/// A gas particle, which can move upwards, upwards diagonally, and horizontally.
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

/// Blueprint for a [`Gas`]
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

#[derive(Event)]
pub struct ParticleMaterialTransitionEvent {
    pub particle_type: ParticleType,
    pub new_state: Box<dyn Material + Send + Sync>,
}

/// Clear all dynamic particles from the world.
#[derive(Event)]
pub struct ClearDynamicParticlesEvent;

/// Clear all static particles from the world.
#[derive(Event)]
pub struct ClearStaticParticlesEvent;

fn on_particle_material_transition(trigger: Trigger<ParticleMaterialTransitionEvent>) {}

fn on_clear_dynamic_particles(
    _trigger: Trigger<ClearDynamicParticlesEvent>,
    mut commands: Commands,
    dynamic_particle_types_query: Query<&ParticleType, Without<WallBlueprint>>,
) {
    dynamic_particle_types_query
        .iter()
        .for_each(|particle_type| {
            commands.trigger(ClearParticleTypeChildrenEvent(particle_type.name.clone()));
        });
}

fn on_clear_static_particles(
    _trigger: Trigger<ClearStaticParticlesEvent>,
    mut commands: Commands,
    dynamic_particle_types_query: Query<&ParticleType, With<WallBlueprint>>,
) {
    dynamic_particle_types_query
        .iter()
        .for_each(|particle_type| {
            commands.trigger(ClearParticleTypeChildrenEvent(particle_type.name.clone()));
        });
}

#[allow(clippy::needless_pass_by_value)]
fn on_solid_blueprint_added(
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

#[allow(clippy::needless_pass_by_value)]
fn on_movable_solid_blueprint_added(
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

#[allow(clippy::needless_pass_by_value)]
fn on_liquid_blueprint_added(
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

#[allow(clippy::needless_pass_by_value)]
fn on_gas_blueprint_added(
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

#[allow(clippy::needless_pass_by_value)]
fn on_wall_added(
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
