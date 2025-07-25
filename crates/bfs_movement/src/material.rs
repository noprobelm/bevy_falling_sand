use bevy::{
    ecs::{
        component::{Mutable, StorageType},
        system::SystemParam,
    },
    prelude::*,
};
use bfs_core::{ClearParticleTypeChildrenEvent, Particle, ParticleType};
use serde::{Deserialize, Serialize};

use crate::Moved;

use super::Movement;

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
            .add_systems(
                Update,
                (ev_clear_dynamic_particles, ev_clear_static_particles),
            );
    }
}

/// Used to describe a Material, which can be translated to a [`Movement`].
pub trait Material {
    #[allow(dead_code)]
    /// Get the [`Movement`] for the material type.
    fn to_movement_priority(&self) -> Movement {
        Movement::empty()
    }
}

/// A simple wall, which has no movement to it.
#[derive(
    Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Reflect, Serialize, Deserialize,
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

impl Component for Wall {
    type Mutability = Mutable;

    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, context| {
            if world.get::<Solid>(context.entity).is_some() {
                world.commands().entity(context.entity).remove::<Solid>();
            }
            if world.get::<MovableSolid>(context.entity).is_some() {
                world
                    .commands()
                    .entity(context.entity)
                    .remove::<MovableSolid>();
            }
            if world.get::<Liquid>(context.entity).is_some() {
                world.commands().entity(context.entity).remove::<Liquid>();
            }
            if world.get::<Gas>(context.entity).is_some() {
                world.commands().entity(context.entity).remove::<Gas>();
            }
            world.commands().entity(context.entity).remove::<Movement>();
            world.commands().entity(context.entity).insert(Moved(false));
        });
    }
}

/// A solid particle, which can move only downwards.
#[derive(
    Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Reflect, Serialize, Deserialize,
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
    fn to_movement_priority(&self) -> Movement {
        Movement::from(vec![vec![IVec2::NEG_Y]])
    }
}

impl Component for Solid {
    type Mutability = Mutable;

    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, context| {
            if world.get::<Wall>(context.entity).is_some() {
                world.commands().entity(context.entity).remove::<Wall>();
            }
            if world.get::<MovableSolid>(context.entity).is_some() {
                world
                    .commands()
                    .entity(context.entity)
                    .remove::<MovableSolid>();
            }
            if world.get::<Liquid>(context.entity).is_some() {
                world.commands().entity(context.entity).remove::<Liquid>();
            }
            if world.get::<Gas>(context.entity).is_some() {
                world.commands().entity(context.entity).remove::<Gas>();
            }
            world.commands().entity(context.entity).insert(Moved(true));
        });
    }
}

/// A movable solid particle, which can move downwards and diagonally.
#[derive(
    Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Reflect, Serialize, Deserialize,
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
    fn to_movement_priority(&self) -> Movement {
        Movement::from(vec![
            vec![IVec2::NEG_Y],
            vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
        ])
    }
}

impl Component for MovableSolid {
    type Mutability = Mutable;

    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, context| {
            if world.get::<Wall>(context.entity).is_some() {
                world.commands().entity(context.entity).remove::<Wall>();
            }
            if world.get::<Solid>(context.entity).is_some() {
                world.commands().entity(context.entity).remove::<Solid>();
            }
            if world.get::<Liquid>(context.entity).is_some() {
                world.commands().entity(context.entity).remove::<Liquid>();
            }
            if world.get::<Gas>(context.entity).is_some() {
                world.commands().entity(context.entity).remove::<Gas>();
                world.commands().entity(context.entity).insert(Moved(true));
            }
        });
    }
}

/// A liquid particle, which can move downwards, diagonally, and horizontally.
///
/// A liquid particle will first attempt to move downards, then downwards diagonally. If no valid
/// positions are found, it will attempt to move horizontally n spaces as a function of its fluidity
/// + 1.
#[derive(
    Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Reflect, Serialize, Deserialize,
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
    fn to_movement_priority(&self) -> Movement {
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

        Movement::from(neighbors)
    }
}

impl Component for Liquid {
    type Mutability = Mutable;

    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, context| {
            if world.get::<Wall>(context.entity).is_some() {
                world.commands().entity(context.entity).remove::<Wall>();
            }
            if world.get::<Solid>(context.entity).is_some() {
                world.commands().entity(context.entity).remove::<Solid>();
            }
            if world.get::<MovableSolid>(context.entity).is_some() {
                world
                    .commands()
                    .entity(context.entity)
                    .remove::<MovableSolid>();
            }
            if world.get::<Gas>(context.entity).is_some() {
                world.commands().entity(context.entity).remove::<Gas>();
                world.commands().entity(context.entity).insert(Moved(true));
            }
        });
    }
}

/// A gas particle, which can move upwards, upwards diagonally, and horizontally.
#[derive(
    Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Reflect, Serialize, Deserialize,
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
    fn to_movement_priority(&self) -> Movement {
        let mut neighbors: Vec<Vec<IVec2>> =
            vec![vec![IVec2::Y, IVec2::new(1, 1), IVec2::new(-1, 1)]];

        for i in 0..self.fluidity {
            neighbors.push(vec![
                IVec2::X * (i + 2) as i32,
                IVec2::NEG_X * (i + 2) as i32,
            ]);
        }

        Movement::from(neighbors)
    }
}

impl Component for Gas {
    type Mutability = Mutable;

    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, context| {
            if world.get::<Wall>(context.entity).is_some() {
                world.commands().entity(context.entity).remove::<Wall>();
            }
            if world.get::<Solid>(context.entity).is_some() {
                world.commands().entity(context.entity).remove::<Solid>();
            }
            if world.get::<MovableSolid>(context.entity).is_some() {
                world
                    .commands()
                    .entity(context.entity)
                    .remove::<MovableSolid>();
            }
            if world.get::<Liquid>(context.entity).is_some() {
                world.commands().entity(context.entity).remove::<Liquid>();
                world.commands().entity(context.entity).insert(Moved(true));
            }
        });
    }
}
/// Clear all dynamic particles from the world.
#[derive(Event)]
pub struct ClearDynamicParticlesEvent;

/// Clear all static particles from the world.
#[derive(Event)]
pub struct ClearStaticParticlesEvent;

/// System param to fetch particle types by material type.
#[derive(SystemParam)]
pub struct ParticleTypeMaterialsParam<'w, 's> {
    walls: Query<'w, 's, &'static ParticleType, With<Wall>>,
    solids: Query<'w, 's, &'static ParticleType, With<Solid>>,
    movable_solids: Query<'w, 's, &'static ParticleType, With<MovableSolid>>,
    liquids: Query<'w, 's, &'static ParticleType, With<Liquid>>,
    gases: Query<'w, 's, &'static ParticleType, With<Gas>>,
    other: Query<
        'w,
        's,
        &'static ParticleType,
        (
            Without<Wall>,
            Without<Solid>,
            Without<MovableSolid>,
            Without<Liquid>,
            Without<Gas>,
        ),
    >,
}

impl ParticleTypeMaterialsParam<'_, '_> {
    /// Returns all particle types that have the `Wall` component.
    #[must_use]
    pub fn walls(&self) -> Vec<&ParticleType> {
        self.walls.iter().collect()
    }

    /// Return the number of wall particles
    #[must_use]
    pub fn num_walls(&self) -> u64 {
        self.walls.iter().len() as u64
    }

    /// Returns all particle types that have the `Solid` component.
    #[must_use]
    pub fn solids(&self) -> Vec<&ParticleType> {
        self.solids.iter().collect()
    }

    /// Return the number of solid particles
    #[must_use]
    pub fn num_solids(&self) -> u64 {
        self.solids.iter().len() as u64
    }

    /// Returns all particle types that have the `MovableSolid` component.
    #[must_use]
    pub fn movable_solids(&self) -> Vec<&ParticleType> {
        self.movable_solids.iter().collect()
    }

    /// Return the number of movable solid particles
    #[must_use]
    pub fn num_movable_solids(&self) -> u64 {
        self.movable_solids.iter().len() as u64
    }

    /// Returns all particle types that have the `Liquid` component.
    #[must_use]
    pub fn liquids(&self) -> Vec<&ParticleType> {
        self.liquids.iter().collect()
    }

    /// Return the number of liquid particles
    #[must_use]
    pub fn num_liquids(&self) -> u64 {
        self.liquids.iter().len() as u64
    }

    /// Returns all particle types that have the `Gas` component.
    #[must_use]
    pub fn gases(&self) -> Vec<&ParticleType> {
        self.gases.iter().collect()
    }

    /// Return the number of gas particles
    #[must_use]
    pub fn num_gases(&self) -> u64 {
        self.gases.iter().len() as u64
    }

    /// Returns all particle types that have none of the material components.
    #[must_use]
    pub fn other(&self) -> Vec<&ParticleType> {
        self.other.iter().collect()
    }

    /// Return the number of other particles
    #[must_use]
    pub fn num_other(&self) -> u64 {
        self.other.iter().len() as u64
    }
}

/// System param to fetch particle types by material type.
#[derive(SystemParam)]
pub struct ParticleMaterialsParam<'w, 's> {
    walls: Query<'w, 's, &'static Particle, With<Wall>>,
    solids: Query<'w, 's, &'static Particle, With<Solid>>,
    movable_solids: Query<'w, 's, &'static Particle, With<MovableSolid>>,
    liquids: Query<'w, 's, &'static Particle, With<Liquid>>,
    gases: Query<'w, 's, &'static Particle, With<Gas>>,
    other: Query<
        'w,
        's,
        &'static Particle,
        (
            Without<Wall>,
            Without<Solid>,
            Without<MovableSolid>,
            Without<Liquid>,
            Without<Gas>,
        ),
    >,
}

impl ParticleMaterialsParam<'_, '_> {
    /// Returns all particle types that have the `Wall` component.
    #[must_use]
    pub fn walls(&self) -> Vec<&Particle> {
        self.walls.iter().collect()
    }

    /// Return the number of wall particles
    #[must_use]
    pub fn num_walls(&self) -> u64 {
        self.walls.iter().len() as u64
    }

    /// Returns all particle types that have the `Solid` component.
    #[must_use]
    pub fn solids(&self) -> Vec<&Particle> {
        self.solids.iter().collect()
    }

    /// Return the number of solid particles
    #[must_use]
    pub fn num_solids(&self) -> u64 {
        self.solids.iter().len() as u64
    }

    /// Returns all particle types that have the `MovableSolid` component.
    #[must_use]
    pub fn movable_solids(&self) -> Vec<&Particle> {
        self.movable_solids.iter().collect()
    }

    /// Return the number of movable solid particles
    #[must_use]
    pub fn num_movable_solids(&self) -> u64 {
        self.movable_solids.iter().len() as u64
    }

    /// Returns all particle types that have the `Liquid` component.
    #[must_use]
    pub fn liquids(&self) -> Vec<&Particle> {
        self.liquids.iter().collect()
    }

    /// Return the number of liquid particles
    #[must_use]
    pub fn num_liquids(&self) -> u64 {
        self.liquids.iter().len() as u64
    }

    /// Returns all particle types that have the `Gas` component.
    #[must_use]
    pub fn gases(&self) -> Vec<&Particle> {
        self.gases.iter().collect()
    }

    /// Return the number of gas particles
    #[must_use]
    pub fn num_gases(&self) -> u64 {
        self.gases.iter().len() as u64
    }

    /// Returns all particle types that have none of the material components.
    #[must_use]
    pub fn other(&self) -> Vec<&Particle> {
        self.other.iter().collect()
    }

    /// Return the number of other particles
    #[must_use]
    pub fn num_other(&self) -> u64 {
        self.other.iter().len() as u64
    }
}

fn ev_clear_dynamic_particles(
    mut ev_clear_dynamic_particles: EventReader<ClearDynamicParticlesEvent>,
    mut ev_clear_particle_type_children: EventWriter<ClearParticleTypeChildrenEvent>,
    dynamic_particle_types_query: Query<&ParticleType, Without<Wall>>,
) {
    ev_clear_dynamic_particles.read().for_each(|_| {
        dynamic_particle_types_query
            .iter()
            .for_each(|particle_type| {
                ev_clear_particle_type_children.write(ClearParticleTypeChildrenEvent(
                    particle_type.name.to_string(),
                ));
            });
    });
}

fn ev_clear_static_particles(
    mut ev_clear_static_particles: EventReader<ClearStaticParticlesEvent>,
    mut ev_clear_particle_type_children: EventWriter<ClearParticleTypeChildrenEvent>,
    static_particle_types_query: Query<&ParticleType, With<Wall>>,
) {
    ev_clear_static_particles.read().for_each(|_| {
        static_particle_types_query
            .iter()
            .for_each(|particle_type| {
                ev_clear_particle_type_children.write(ClearParticleTypeChildrenEvent(
                    particle_type.name.to_string(),
                ));
            });
    });
}
