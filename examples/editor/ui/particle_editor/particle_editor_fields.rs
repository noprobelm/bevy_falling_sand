use bevy::{
    platform::{
        collections::{hash_map::Entry, HashMap},
        hash::FixedHasher,
    },
    prelude::*,
};
use bevy_falling_sand::prelude::*;

#[derive(Resource, Clone, Default, Debug)]
pub(crate) struct ParticleEditorTypeMap {
    map: HashMap<&'static str, Entity>,
}

impl ParticleEditorTypeMap {
    pub(crate) fn contains(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&'static str, &Entity)> {
        self.map.iter().map(|(&k, v)| (k, v))
    }

    pub(crate) fn keys(&self) -> impl Iterator<Item = &'static str> + use<'_> {
        self.map.keys().copied()
    }

    pub(crate) fn insert(&mut self, name: &'static str, entity: Entity) -> Option<Entity> {
        self.map.insert(name, entity)
    }

    pub(crate) fn entry(
        &mut self,
        name: &'static str,
    ) -> Entry<'_, &'static str, Entity, FixedHasher> {
        self.map.entry(name)
    }

    pub(crate) fn get(&self, name: &str) -> Option<&Entity> {
        self.map.get(name)
    }

    /// Remove a particle type from the map
    pub(crate) fn remove(&mut self, name: &str) -> Option<Entity> {
        self.map.remove(name)
    }

    /// Returns 'true' if the particle type map is empty
    pub(crate) fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

pub fn sync_particle_editor_type_map_fields(
    mut commands: Commands,
    particle_type_map: Res<ParticleTypeMap>,
    particle_editor_type_map: Res<ParticleEditorTypeMap>,
) {
}
