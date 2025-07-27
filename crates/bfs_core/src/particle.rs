//! Provides all core functionality for particles, including registration, mutation, removal, and
//! even extension through external plugins if desired.
use bevy::ecs::component::Mutable;
use bevy::platform::collections::hash_map::Entry;
use bevy::platform::hash::FixedHasher;
use bevy::prelude::*;
use bevy::{ecs::component::StorageType, platform::collections::HashMap};
use bevy_turborand::DelegatedRng;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::ops::RangeBounds;

use crate::ParticleMap;

/// Adds Bevy plugin elements for core particle functionality.
pub(super) struct ParticleCorePlugin;

impl Plugin for ParticleCorePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Particle>()
            .register_type::<ParticleType>()
            .register_type::<ParticlePosition>()
            .init_resource::<ParticleSimulationRun>()
            .configure_sets(
                PostUpdate,
                ParticleSimulationSet.run_if(
                    resource_exists::<ParticleSimulationRun>
                        .or(condition_ev_simulation_step_received),
                ),
            )
            .configure_sets(PreUpdate, ParticleRegistrationSet)
            .init_resource::<ParticleTypeMap>()
            .add_event::<SimulationStepEvent>()
            .add_event::<ParticleRegistrationEvent>()
            .add_event::<ResetParticleChildrenEvent>()
            .add_event::<ResetParticleEvent>()
            .add_event::<RemoveParticleEvent>()
            .add_systems(
                PreUpdate,
                handle_new_particles.in_set(ParticleRegistrationSet),
            )
            .add_systems(
                PreUpdate,
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
    /// The type of the internal RNG
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

/// System set for registering new particles
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleRegistrationSet;

/// Unique identifer for a particle type. No two particle types with the same name can exist.
#[derive(
    Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Reflect, Serialize, Deserialize,
)]
#[reflect(Component)]
pub struct ParticleType {
    /// The particle type's name.
    pub name: Cow<'static, str>,
}

impl ParticleType {
    /// Initialize a new `ParticleType` from a static string
    #[must_use]
    pub fn new(name: &'static str) -> Self {
        Self {
            name: Cow::Borrowed(name),
        }
    }

    /// Initialize a new `ParticleType` from an owned string
    #[must_use]
    pub fn from_string(name: String) -> Self {
        Self {
            name: Cow::Owned(name),
        }
    }
}

impl From<&'static str> for ParticleType {
    fn from(name: &'static str) -> Self {
        Self::new(name)
    }
}

impl From<String> for ParticleType {
    fn from(name: String) -> Self {
        Self::from_string(name)
    }
}

impl From<Cow<'static, str>> for ParticleType {
    fn from(name: Cow<'static, str>) -> Self {
        Self { name }
    }
}

impl Into<Cow<'static, str>> for ParticleType {
    fn into(self) -> Cow<'static, str> {
        self.name
    }
}

impl Component for ParticleType {
    type Mutability = Mutable;

    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, context| {
            let name = {
                let particle_type = world.get::<Self>(context.entity).unwrap();
                // For now, only support static strings in the map
                if let Cow::Borrowed(name) = &particle_type.name {
                    Some(*name)
                } else {
                    warn!(
                        "ParticleType with owned string cannot be registered in map: '{}'",
                        particle_type.name
                    );
                    None
                }
            };

            if let Some(name) = name {
                // Add ParticleInstances component - relationships will handle synchronization
                world
                    .commands()
                    .entity(context.entity)
                    .insert(ParticleInstances::default());

                // Register in ParticleTypeMap
                let mut type_map = world.resource_mut::<ParticleTypeMap>();
                type_map.insert(name, context.entity);
            }
        });

        hooks.on_remove(|mut world, context| {
            let name = {
                let particle_type = world.get::<Self>(context.entity).unwrap();
                // Only remove if it was a static string
                if let Cow::Borrowed(name) = &particle_type.name {
                    Some(*name)
                } else {
                    None
                }
            };

            if let Some(name) = name {
                let mut type_map = world.resource_mut::<ParticleTypeMap>();
                type_map.remove(name);
            }
        });
    }
}

/// Maps each [`ParticleType`] to their corresponding entity in the ECS world.
#[derive(Resource, Clone, Default, Debug)]
pub struct ParticleTypeMap {
    map: HashMap<&'static str, Entity>,
}

impl ParticleTypeMap {
    /// Returns true if the designated key exists in the map.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    /// Returns a reference to the key-value pair for the given key, if it exists.
    #[must_use]
    pub fn get_key_value(&self, key: &str) -> Option<(&&str, &Entity)> {
        self.map.get_key_value(key)
    }

    /// Iterate over key value pairs in the map.
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &Entity)> {
        self.map.iter().map(|(&k, v)| (k, v))
    }

    /// Iterate over keys in the map.
    pub fn keys(&self) -> impl Iterator<Item = &'static str> + use<'_> {
        self.map.keys().copied()
    }

    /// Insert a new particle type entity.
    pub fn insert(&mut self, name: &'static str, entity: Entity) -> Option<Entity> {
        self.map.insert(name, entity)
    }

    /// Get the [`bevy::platform::collections::hash_map::Entry`]
    pub fn entry(&mut self, name: &'static str) -> Entry<'_, &'static str, Entity, FixedHasher> {
        self.map.entry(name)
    }

    /// Get a particle type from the map if it exists.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Entity> {
        self.map.get(name)
    }

    /// Remove a particle type from the map
    fn remove(&mut self, name: &str) -> Option<Entity> {
        self.map.remove(name)
    }

    /// Returns 'true' if the particle type map is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

/// Relationship component that tracks which ParticleType entity a Particle belongs to.
/// This is the relationship source pointing to the target ParticleType entity.
#[derive(Component)]
#[relationship(relationship_target = ParticleInstances)]
pub struct AttachedToParticleType(pub Entity);

/// RelationshipTarget component that tracks all Particle entities belonging to a ParticleType.
/// This component is automatically maintained by Bevy's relationship system.
#[derive(Component)]
#[relationship_target(relationship = AttachedToParticleType, linked_spawn)]
pub struct ParticleInstances(Vec<Entity>);

impl ParticleInstances {
    /// Create a new empty ParticleInstances collection.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Add a particle instance to this type.
    /// Note: With Bevy relationships, this is handled automatically
    pub fn add(&mut self, entity: Entity) {
        if !self.0.contains(&entity) {
            self.0.push(entity);
        }
    }

    /// Remove a particle instance from this type.
    /// Note: With Bevy relationships, this is handled automatically
    pub fn remove(&mut self, entity: Entity) {
        self.0.retain(|&e| e != entity);
    }

    /// Get all particle instances.
    pub fn iter(&self) -> std::slice::Iter<Entity> {
        self.0.iter()
    }

    /// Get the number of instances.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if there are no instances.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Clear all instances.
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

impl Default for ParticleInstances {
    fn default() -> Self {
        Self::new()
    }
}

/// Marker component for a Particle entity.
#[derive(Clone, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Particle {
    /// The name of the particle, which corresponds to its [`ParticleType`] and can be used as an
    /// index in the  [`ParticleTypeMap`] resource.
    pub name: Cow<'static, str>,
}

impl Particle {
    /// Initialize a new `Particle` from a static string
    #[must_use]
    pub fn new(name: &'static str) -> Self {
        Self {
            name: Cow::Borrowed(name),
        }
    }

    /// Initialize a new `Particle` from an owned string
    #[must_use]
    pub fn from_string(name: String) -> Self {
        Self {
            name: Cow::Owned(name),
        }
    }
}

impl From<&'static str> for Particle {
    fn from(name: &'static str) -> Self {
        Self::new(name)
    }
}

impl From<String> for Particle {
    fn from(name: String) -> Self {
        Self::from_string(name)
    }
}

impl From<Cow<'static, str>> for Particle {
    fn from(name: Cow<'static, str>) -> Self {
        Self { name }
    }
}

impl Into<Cow<'static, str>> for Particle {
    fn into(self) -> Cow<'static, str> {
        self.name
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

            // Bevy's relationship system automatically handles cleanup of AttachedToParticleType
            // and synchronization with ParticleInstances when entities are removed
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
            entities.push(entity);

            // Use Bevy's relationship system - the ParticleInstances will be
            // automatically updated when we add the AttachedToParticleType component
            commands.entity(entity).insert((
                ParticlePosition(coordinates),
                AttachedToParticleType(*parent_handle),
            ));
        } else {
            warn!(
                "Attempted to spawn particle without valid parent type: '{:?}'",
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
    particle_type_query: Query<&ParticleInstances, With<ParticleType>>,
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
/// Note: With Bevy's relationship system, this cleanup should be automatic,
/// but we keep this as a safety net for edge cases.
#[allow(clippy::needless_pass_by_value)]
fn cleanup_orphaned_particle_instances(
    mut particle_type_query: Query<&mut ParticleInstances, With<ParticleType>>,
    particle_query: Query<Entity, With<Particle>>,
) {
    for mut particle_instances in particle_type_query.iter_mut() {
        particle_instances
            .0
            .retain(|&entity| particle_query.get(entity).is_ok());
    }
}
