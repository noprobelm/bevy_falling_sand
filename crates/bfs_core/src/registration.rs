//! Provides particle type registration and management functionality.
use bevy::ecs::component::Mutable;
use bevy::platform::collections::hash_map::Entry;
use bevy::platform::hash::FixedHasher;
use bevy::prelude::*;
use bevy::{ecs::component::StorageType, platform::collections::HashMap};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use crate::{ParticleMap, ParticleRegistrationEvent};

/// Adds Bevy plugin elements for particle type registration.
pub(super) struct ParticleRegistrationPlugin;

impl Plugin for ParticleRegistrationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ParticleType>()
            .register_type::<Particle>()
            .register_type::<ParticlePosition>()
            .configure_sets(PreUpdate, ParticleRegistrationSet)
            .add_event::<ResetParticleChildrenEvent>()
            .add_event::<ResetParticleEvent>()
            .init_resource::<ParticleTypeMap>()
            .add_systems(
                PreUpdate,
                (
                    evr_reset_particle,
                    evr_reset_particle_children,
                    handle_new_particles.in_set(ParticleRegistrationSet),
                ),
            );
    }
}

/// System set for registering new particles
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleRegistrationSet;

/// Unique identifer for a particle type.
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
    pub const fn new(name: &'static str) -> Self {
        Self {
            name: Cow::Borrowed(name),
        }
    }

    /// Initialize a new `ParticleType` from an owned string
    #[must_use]
    pub const fn from_string(name: String) -> Self {
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

impl From<ParticleType> for Cow<'static, str> {
    fn from(val: ParticleType) -> Self {
        val.name
    }
}

impl Component for ParticleType {
    type Mutability = Mutable;

    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, context| {
            let name = {
                let particle_type = world.get::<Self>(context.entity).unwrap();
                match &particle_type.name {
                    Cow::Borrowed(name) => Some(*name),
                    Cow::Owned(owned_name) => {
                        // Convert owned string to static string by leaking it
                        // This is necessary for scene deserialization where strings come as owned
                        let static_name: &'static str =
                            Box::leak(owned_name.clone().into_boxed_str());
                        Some(static_name)
                    }
                }
            };

            if let Some(name) = name {
                world
                    .commands()
                    .entity(context.entity)
                    .insert(ParticleInstances::default());

                let mut type_map = world.resource_mut::<ParticleTypeMap>();
                type_map.insert(name, context.entity);
            }
        });

        hooks.on_remove(|mut world, context| {
            let name = {
                let particle_type = world.get::<Self>(context.entity).unwrap();
                match &particle_type.name {
                    Cow::Borrowed(name) => Some(*name),
                    Cow::Owned(owned_name) => {
                        let type_map = world.resource::<ParticleTypeMap>();

                        let found_key = type_map.keys().find(|&key| key == owned_name.as_str());
                        found_key
                    }
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
    /// Create a new empty [`ParticleInstances`] collection.
    #[must_use]
    pub const fn new() -> Self {
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
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if there are no instances.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
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
    pub const fn new(name: &'static str) -> Self {
        Self {
            name: Cow::Borrowed(name),
        }
    }

    /// Initialize a new `Particle` from an owned string
    #[must_use]
    pub const fn from_string(name: String) -> Self {
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

impl From<Particle> for Cow<'static, str> {
    fn from(val: Particle) -> Self {
        val.name
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
        });
    }
}

/// Holds the position of a particle in the simulation
#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect, Serialize, Deserialize,
)]
#[reflect(Component)]
pub struct ParticlePosition(pub IVec2);

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

/// Triggers a [`ParticleType`] to reset all of its children's data.
#[derive(Clone, Event, Hash, Debug, Eq, PartialEq, PartialOrd)]
pub struct ResetParticleChildrenEvent {
    /// The particle type entity to reset children for.
    pub entity: Entity,
}

/// Triggers a particle to reset itself to its parent's blueprint data.
#[derive(Clone, Event, Hash, Debug, Eq, PartialEq, PartialOrd)]
pub struct ResetParticleEvent {
    /// The entity to reset particle blueprint data for.
    pub entity: Entity,
}

#[allow(clippy::needless_pass_by_value)]
fn evr_reset_particle(
    mut evr_reset_particle: EventReader<ResetParticleEvent>,
    mut particle_query: Query<&mut crate::Particle>,
) {
    evr_reset_particle.read().for_each(|ev| {
        if let Ok(particle) = particle_query.get_mut(ev.entity) {
            particle.into_inner();
        } else {
            warn!("No Particle component found for entity {:?}", ev.entity);
        }
    });
}

#[allow(clippy::needless_pass_by_value)]
fn evr_reset_particle_children(
    mut evr_reset_particle_children: EventReader<ResetParticleChildrenEvent>,
    mut evw_reset_particle: EventWriter<ResetParticleEvent>,
    particle_type_query: Query<&ParticleInstances, With<ParticleType>>,
) {
    evr_reset_particle_children.read().for_each(|ev| {
        if let Ok(particle_instances) = particle_type_query.get(ev.entity) {
            particle_instances.iter().for_each(|entity| {
                evw_reset_particle.write(ResetParticleEvent { entity: *entity });
            });
        }
    });
}
