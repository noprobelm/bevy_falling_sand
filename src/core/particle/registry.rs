use bevy::{platform::collections::HashMap, prelude::*};
use std::borrow::Cow;

pub(super) struct RegistryPlugin;

impl Plugin for RegistryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticleTypeRegistry>();
    }
}

/// Associates unique [`ParticleType`](crate::prelude::ParticleType) with their entity ID
#[derive(Resource, Clone, Default, Eq, PartialEq, Debug, Reflect)]
#[reflect(Resource)]
pub struct ParticleTypeRegistry {
    map: HashMap<Cow<'static, str>, Entity>,
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
    ///     if registry.contains("Sand") {
    ///         println!("Sand type is registered");
    ///     }
    /// }
    /// ```
    #[must_use]
    #[inline(always)]
    pub fn contains(&self, name: &str) -> bool {
        self.map.contains_key(name)
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
    ///     for (name, entity) in registry.iter() {
    ///         println!("{name}: {entity:?}");
    ///     }
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Entity)> {
        self.map.iter().map(|(k, v)| (k.as_ref(), v))
    }

    /// Iterate over particle names in the map.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ParticleTypeRegistry;
    ///
    /// fn list_names(registry: Res<ParticleTypeRegistry>) {
    ///     for name in registry.names() {
    ///         println!("Particle type: {name}");
    ///     }
    /// }
    /// ```
    pub fn names(&self) -> impl Iterator<Item = &str> + use<'_> {
        self.map.keys().map(AsRef::as_ref)
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
    ///     if let Some(entity) = registry.get("Sand") {
    ///         println!("Sand type entity: {entity:?}");
    ///     }
    /// }
    /// ```
    #[must_use]
    #[inline(always)]
    pub fn get(&self, name: &str) -> Option<&Entity> {
        self.map.get(name)
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
    pub(crate) fn insert(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        entity: Entity,
    ) -> Option<Entity> {
        self.map.insert(name.into(), entity)
    }

    /// Remove a particle type from the map
    pub(crate) fn remove(&mut self, name: &str) -> Option<Entity> {
        self.map.remove(name)
    }
}
