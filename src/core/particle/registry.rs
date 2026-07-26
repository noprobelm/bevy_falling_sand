use bevy::{platform::collections::HashMap, prelude::*};

use super::ParticleTypeId;

pub(super) struct RegistryPlugin;

impl Plugin for RegistryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticleTypeRegistry>();
    }
}

/// Associates unique [`ParticleTypeId`] values with their current
/// [`ParticleType`](crate::prelude::ParticleType) entity.
#[derive(Resource, Clone, Default, Eq, PartialEq, Debug, Reflect)]
#[reflect(Resource)]
pub struct ParticleTypeRegistry {
    map: HashMap<ParticleTypeId, Entity>,
}

impl ParticleTypeRegistry {
    /// Returns true if the designated key exists in the map.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ParticleTypeRegistry;
    ///
    /// fn check_type(registry: Res<ParticleTypeRegistry>) {
    ///     # let sand = bevy_falling_sand::core::ParticleTypeId::new();
    ///     if registry.contains(sand) {
    ///         println!("Sand type is registered");
    ///     }
    /// }
    /// ```
    #[must_use]
    #[inline(always)]
    pub fn contains(&self, id: impl Into<ParticleTypeId>) -> bool {
        self.map.contains_key(&id.into())
    }

    /// Iterate over key value pairs in the map.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ParticleTypeRegistry;
    ///
    /// fn list_types(registry: Res<ParticleTypeRegistry>) {
    ///     for (id, entity) in registry.iter() {
    ///         println!("{id:?}: {entity:?}");
    ///     }
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (&ParticleTypeId, &Entity)> {
        self.map.iter()
    }

    /// Iterate over entities in the map.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ParticleTypeRegistry;
    ///
    /// fn list_entities(registry: Res<ParticleTypeRegistry>) {
    ///     for entity in registry.entities() {
    ///         println!("ParticleType entity: {entity:?}");
    ///     }
    /// }
    /// ```
    pub fn entities(&self) -> impl Iterator<Item = &Entity> + use<'_> {
        self.map.values()
    }

    /// Get a particle type from the map if it exists.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ParticleTypeRegistry;
    ///
    /// fn get_sand(registry: Res<ParticleTypeRegistry>) {
    ///     # let sand = bevy_falling_sand::core::ParticleTypeId::new();
    ///     if let Some(entity) = registry.get(sand) {
    ///         println!("Sand type entity: {entity:?}");
    ///     }
    /// }
    /// ```
    #[must_use]
    #[inline(always)]
    pub fn get(&self, id: impl Into<ParticleTypeId>) -> Option<&Entity> {
        self.map.get(&id.into())
    }

    /// Returns `true` if the particle type map is empty.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ParticleTypeRegistry;
    ///
    /// fn check_registry(registry: Res<ParticleTypeRegistry>) {
    ///     if registry.is_empty() {
    ///         println!("No particle types registered");
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Insert a new [`ParticleType`](crate::ParticleType) and entity.
    #[inline(always)]
    pub(crate) fn insert(&mut self, id: ParticleTypeId, entity: Entity) -> Option<Entity> {
        self.map.insert(id, entity)
    }

    /// Remove a particle type from the map
    pub(crate) fn remove(&mut self, id: ParticleTypeId) -> Option<Entity> {
        self.map.remove(&id)
    }
}
