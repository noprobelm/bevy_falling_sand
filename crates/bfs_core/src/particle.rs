//! Provides all core functionality for particles, including registration, mutation, removal, and
//! even extension through external plugins if desired.
use bevy::ecs::component::Mutable;
use bevy::platform::collections::hash_map::Entry;
use bevy::platform::hash::FixedHasher;
use bevy::prelude::*;
use bevy::{ecs::component::StorageType, platform::collections::HashMap};
use bevy_turborand::DelegatedRng;
use serde::{Deserialize, Serialize};
use std::ops::RangeBounds;

use crate::ParticleMap;

/// Adds Bevy plugin elements for core particle functionality.
pub(super) struct ParticleCorePlugin;

impl Plugin for ParticleCorePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Particle>()
            .register_type::<ParticleTypeId>()
            .register_type::<ParticlePosition>()
            .register_type::<AttachedToParticleType>()
            .register_type::<ParticleInstances>()
            .init_resource::<ParticleSimulationRun>()
            .configure_sets(
                Update,
                ParticleSimulationSet.run_if(
                    resource_exists::<ParticleSimulationRun>
                        .or(condition_ev_simulation_step_received),
                ),
            )
            .init_resource::<ParticleTypeMap>()
            .add_event::<SimulationStepEvent>()
            .add_event::<ParticleRegistrationEvent>()
            .add_event::<ResetParticleChildrenEvent>()
            .add_event::<ResetParticleEvent>()
            .add_event::<RemoveParticleEvent>()
            .add_systems(PreUpdate, handle_new_particles)
            .add_systems(
                Update,
                (
                    ev_reset_particle,
                    ev_reset_particle_children,
                    cleanup_orphaned_particle_instances,
                ),
            );
    }
}

/// A trait for RNG utilities used in particle systems.
pub trait ParticleRng: Component {
    /// The type of the internal RNG (must implement the required methods).
    type InnerRng: DelegatedRng;

    /// Get mutable access to the inner RNG.
    fn inner_mut(&mut self) -> &mut Self::InnerRng;

    /// Shuffle the given slice.
    fn shuffle<T>(&mut self, slice: &mut [T]) {
        self.inner_mut().shuffle(slice);
    }

    /// Return true with the given probability.
    fn chance(&mut self, rate: f64) -> bool {
        self.inner_mut().chance(rate)
    }

    /// Sample a random element from a list.
    fn sample<'a, T>(&mut self, list: &'a [T]) -> Option<&'a T> {
        self.inner_mut().sample(list)
    }

    /// Return a random index within the given bounds.
    fn index(&mut self, bound: impl RangeBounds<usize>) -> usize {
        self.inner_mut().index(bound)
    }
}

/// Convenience macro for implementing [`ParticleRng`] on a component.
#[macro_export]
macro_rules! impl_particle_rng {
    ($wrapper:ident, $inner:ty) => {
        impl ParticleRng for $wrapper {
            type InnerRng = $inner;

            fn inner_mut(&mut self) -> &mut Self::InnerRng {
                &mut self.0
            }
        }
    };
}

/// Marker resource to indicate whether the simulation should be running.
#[derive(Resource, Default)]
pub struct ParticleSimulationRun;

/// System set for particle simulation systems.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleSimulationSet;

/// Unique identifer for a particle type. No two particle types with the same name can exist.
#[derive(
    Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Reflect, Serialize, Deserialize,
)]
#[reflect(Component)]
pub struct ParticleTypeId {
    /// The particle type's name.
    pub name: String,
}

impl ParticleTypeId {
    /// Initialize a new `ParticleType`
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Component for ParticleTypeId {
    type Mutability = Mutable;

    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, context| {
            let particle_type = world.get::<Self>(context.entity).unwrap();
            let name = particle_type.name.clone();

            // Add ParticleInstances component
            world
                .commands()
                .entity(context.entity)
                .insert(ParticleInstances::default());

            // Register in ParticleTypeMap
            let mut type_map = world.resource_mut::<ParticleTypeMap>();
            type_map.insert(name, context.entity);
        });

        hooks.on_remove(|mut world, context| {
            let particle_type = world.get::<Self>(context.entity).unwrap();
            let name = particle_type.name.clone();

            let mut type_map = world.resource_mut::<ParticleTypeMap>();
            type_map.remove(&name);
        });
    }
}

/// Maps each [`ParticleType`] to their corresponding entity in the ECS world.
#[derive(Resource, Clone, Default, Debug)]
pub struct ParticleTypeMap {
    map: HashMap<String, Entity>,
}

impl ParticleTypeMap {
    /// Returns true if the designated key exists in the map.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    /// Iterate over key value pairs in the map.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Entity)> {
        self.map.iter()
    }

    /// Iterate over keys in the map.
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.map.keys()
    }

    /// Insert a new particle type entity.
    pub fn insert(&mut self, name: String, entity: Entity) -> Option<Entity> {
        self.map.insert(name, entity)
    }

    /// Get the [`bevy::platform::collections::hash_map::Entry`]
    pub fn entry(&mut self, name: String) -> Entry<'_, String, Entity, FixedHasher> {
        self.map.entry(name)
    }

    /// Get a particle type from the map if it exists.
    #[must_use]
    pub fn get(&self, name: &String) -> Option<&Entity> {
        self.map.get(name)
    }

    /// Remove a particle type from the map
    fn remove(&mut self, name: &String) -> Option<Entity> {
        self.map.remove(name)
    }
}

/// Component that tracks which ParticleTypeId entity a Particle belongs to.
/// This replaces the Parent/Child relationship for custom particle management.
#[derive(Clone, Debug, Eq, PartialEq, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct AttachedToParticleType(pub Entity);

/// Component that tracks all Particle entities belonging to a ParticleTypeId.
/// This replaces the Children component for custom particle management.
#[derive(Clone, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct ParticleInstances {
    /// The list of particle entity instances belonging to this ParticleTypeId.
    pub instances: Vec<Entity>,
}

impl ParticleInstances {
    /// Add a particle instance to this type.
    pub fn add(&mut self, entity: Entity) {
        if !self.instances.contains(&entity) {
            self.instances.push(entity);
        }
    }

    /// Remove a particle instance from this type.
    pub fn remove(&mut self, entity: Entity) {
        self.instances.retain(|&e| e != entity);
    }

    /// Get all particle instances.
    pub fn iter(&self) -> std::slice::Iter<Entity> {
        self.instances.iter()
    }

    /// Get the number of instances.
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// Check if there are no instances.
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    /// Clear all instances.
    pub fn clear(&mut self) {
        self.instances.clear();
    }
}

/// Marker component for a Particle entity.
#[derive(Clone, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Particle {
    /// The name of the particle, which corresponds to its [`ParticleType`] and can be used as an
    /// index in the  [`ParticleTypeMap`] resource.
    pub name: String,
}

impl Particle {
    /// Initialize a new `Particle`
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Component for Particle {
    type Mutability = Mutable;

    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_remove(|mut world, context| {
            if let Some(position) = world.get::<ParticlePosition>(context.entity) {
                let position = position.0;
                let mut map = world.resource_mut::<ParticleMap>();
                map.remove(&position);
            }

            // Clean up relationship when particle is removed
            if let Some(attached_to) = world.get::<AttachedToParticleType>(context.entity) {
                let parent_entity = attached_to.0;
                if let Some(mut particle_instances) =
                    world.get_mut::<ParticleInstances>(parent_entity)
                {
                    particle_instances.remove(context.entity);
                }
            }
        });
    }
}

/// Holds the position of a particle in the simulation
#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect, Serialize, Deserialize,
)]
#[reflect(Component)]
pub struct ParticlePosition(pub IVec2);

/// Event which is used to trigger the simulation to step forward by one tick.
#[derive(Clone, Event, Hash, Debug, Eq, PartialEq, PartialOrd)]
pub struct SimulationStepEvent;

/// An event which is sent each time a new [`Particle`] has been spawned into the world. Systems
/// which listen for this event can insert other Particle-type components to the subject entitiesa.
#[derive(Clone, Event, Hash, Debug, Eq, PartialEq, PartialOrd)]
pub struct ParticleRegistrationEvent {
    /// The new particle entities.
    pub entities: Vec<Entity>,
}

/// Event to send each tiem a Particle is removed from the simulation.
#[derive(Event)]
pub struct RemoveParticleEvent {
    /// The position of the particle.
    pub position: IVec2,
    /// Should the underlying entity be despawned
    pub despawn: bool,
}

#[derive(Event)]
/// Triggers a [`ParticleType`] to reset all of its children's data.
pub struct ResetParticleChildrenEvent {
    /// The particle type entity to reset children for.
    pub entity: Entity,
}

#[derive(Event)]
/// Triggers a particle to reset itself to its parent's blueprint data.
pub struct ResetParticleEvent {
    /// The entity to reset particle blueprint data for.
    pub entity: Entity,
}

#[allow(clippy::needless_pass_by_value)]
fn condition_ev_simulation_step_received(
    mut ev_simulation_step: EventReader<SimulationStepEvent>,
) -> bool {
    for _ in ev_simulation_step.read() {
        return true;
    }
    false
}

/// Handles new particles as they are added to the world. If a new particle is being added at the same
/// coordinate of an existing entity, the new particle is despawned.
#[allow(clippy::needless_pass_by_value)]
fn handle_new_particles(
    mut commands: Commands,
    mut particle_type_query: Query<&mut ParticleInstances, With<ParticleTypeId>>,
    particle_query: Query<(&Particle, &Transform, Entity), Changed<Particle>>,
    mut map: ResMut<ParticleMap>,
    type_map: Res<ParticleTypeMap>,
    mut ev_particle_registered: EventWriter<ParticleRegistrationEvent>,
) {
    let mut entities: Vec<Entity> = vec![];

    for (particle_type, transform, entity) in particle_query.iter() {
        let coordinates = IVec2::new(
            transform.translation.x as i32,
            transform.translation.y as i32,
        );

        if let Some(chunk) = map.chunk_mut(&coordinates) {
            if chunk.entry(coordinates).or_insert(entity) != &entity {
                commands.entity(entity).despawn();
                continue;
            }
        } else {
            // If the chunk is out of bounds, also despawn the entity
            commands.entity(entity).despawn();
            continue;
        }

        if let Some(parent_handle) = type_map.get(&particle_type.name) {
            if let Ok(mut particle_instances) = particle_type_query.get_mut(*parent_handle) {
                entities.push(entity);
                particle_instances.add(entity);
                commands.entity(entity).insert((
                    ParticlePosition(coordinates),
                    AttachedToParticleType(*parent_handle),
                ));
            }
        } else {
            warn!(
                "No parent entity found for particle type {:?}",
                particle_type
            );
        }
    }

    ev_particle_registered.write(ParticleRegistrationEvent { entities });
}

#[allow(clippy::needless_pass_by_value)]
fn ev_reset_particle(
    mut ev_reset_particle: EventReader<ResetParticleEvent>,
    mut particle_query: Query<&mut Particle>,
) {
    ev_reset_particle.read().for_each(|ev| {
        if let Ok(particle) = particle_query.get_mut(ev.entity) {
            particle.into_inner();
        } else {
            warn!("No Particle component found for entity {:?}", ev.entity);
        }
    });
}

#[allow(clippy::needless_pass_by_value)]
fn ev_reset_particle_children(
    mut ev_reset_particle_children: EventReader<ResetParticleChildrenEvent>,
    mut ev_reset_particle: EventWriter<ResetParticleEvent>,
    particle_type_query: Query<&ParticleInstances, With<ParticleTypeId>>,
) {
    ev_reset_particle_children.read().for_each(|ev| {
        if let Ok(particle_instances) = particle_type_query.get(ev.entity) {
            particle_instances.iter().for_each(|entity| {
                ev_reset_particle.write(ResetParticleEvent { entity: *entity });
            });
        }
    });
}

/// System to clean up orphaned particle instances from ParticleInstances components
/// when particles are despawned outside of the normal flow.
#[allow(clippy::needless_pass_by_value)]
fn cleanup_orphaned_particle_instances(
    mut particle_type_query: Query<&mut ParticleInstances, With<ParticleTypeId>>,
    particle_query: Query<Entity, With<Particle>>,
) {
    for mut particle_instances in particle_type_query.iter_mut() {
        particle_instances
            .instances
            .retain(|&entity| particle_query.get(entity).is_ok());
    }
}
