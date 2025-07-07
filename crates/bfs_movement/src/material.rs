use bevy::prelude::*;
use bfs_core::{ClearParticleTypeChildrenEvent, ParticleType};
use serde::{Deserialize, Serialize};

use super::MovementPriority;

pub(super) struct MaterialPlugin;

impl Plugin for MaterialPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Wall>()
            .register_type::<Solid>()
            .register_type::<MovableSolid>()
            .register_type::<Liquid>()
            .register_type::<Gas>()
            .add_event::<ClearDynamicParticlesEvent>()
            .add_event::<ClearStaticParticlesEvent>()
            .add_systems(Update, ev_clear_dynamic_particles)
            .add_observer(on_clear_static_particles);
    }
}

/// Used to describe a Material, which can be translated to a [`MovementPriority`].
pub trait Material {
    #[allow(dead_code)]
    /// Get the [`MovementPriority`] for the material type.
    fn to_movement_priority(&self) -> MovementPriority {
        MovementPriority::empty()
    }
}

/// A simple wall, which has no movement to it.
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
#[reflect(Component)]
pub struct Wall;

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
#[reflect(Component)]
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
#[reflect(Component)]
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
#[reflect(Component)]
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
#[reflect(Component)]
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

/// Clear all dynamic particles from the world.
#[derive(Event)]
pub struct ClearDynamicParticlesEvent;

/// Clear all static particles from the world.
#[derive(Event)]
pub struct ClearStaticParticlesEvent;

fn ev_clear_dynamic_particles(
    mut ev_clear_dynamic_particles: EventReader<ClearDynamicParticlesEvent>,
    mut ev_clear_particle_type_children: EventWriter<ClearParticleTypeChildrenEvent>,
    dynamic_particle_types_query: Query<&ParticleType, Without<Wall>>,
) {
    ev_clear_dynamic_particles.read().for_each(|_| {
        dynamic_particle_types_query
            .iter()
            .for_each(|particle_type| {
                ev_clear_particle_type_children
                    .write(ClearParticleTypeChildrenEvent(particle_type.name.clone()));
            });
    });
}

fn on_clear_static_particles(
    _trigger: Trigger<ClearStaticParticlesEvent>,
    mut commands: Commands,
    dynamic_particle_types_query: Query<&ParticleType, With<Wall>>,
) {
    dynamic_particle_types_query
        .iter()
        .for_each(|particle_type| {
            commands.trigger(ClearParticleTypeChildrenEvent(particle_type.name.clone()));
        });
}
