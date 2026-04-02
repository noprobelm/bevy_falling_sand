//! Particle type definition persistence signals and systems.
//!
//! This module handles saving and loading particle type definitions to/from RON files.

use crate::core::ParticleType;
#[cfg(feature = "movement")]
use crate::movement::{Momentum, MovementRng};
#[cfg(feature = "reactions")]
use crate::reactions::ReactionRng;
#[cfg(feature = "render")]
use crate::render::ColorRng;
use bevy::{
    prelude::*,
    scene::{serde::SceneDeserializer, DynamicScene, DynamicSceneBuilder, SceneSpawner},
};
use serde::de::DeserializeSeed;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// Internal plugin for particle type persistence signals.
pub(super) struct ParticleTypePersistencePlugin;

impl Plugin for ParticleTypePersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PersistParticleTypesSignal>()
            .add_message::<ParticleTypesPersistedSignal>()
            .add_message::<LoadParticleTypesSignal>()
            .add_message::<ParticleTypesLoadedSignal>()
            .add_systems(Update, (msgr_save_particle_types, msgr_load_particle_types));
    }
}

/// Signal to trigger saving of all particle type definitions to RON format.
#[derive(Event, Message, Clone, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub struct PersistParticleTypesSignal(pub PathBuf);

/// Signal to indicate particle types were successfully saved to disk
#[derive(Event, Message, Clone, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub struct ParticleTypesPersistedSignal(pub PathBuf);

/// Signal to trigger loading particle definitions from RON scene file.
#[derive(Event, Message, Clone, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub struct LoadParticleTypesSignal(pub PathBuf);

/// Signal to indicate particle types were successfully loaded from disk
#[derive(Event, Message, Clone, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub struct ParticleTypesLoadedSignal(pub PathBuf);

/// System to save all `ParticleType` entities and their components to RON.
fn msgr_save_particle_types(world: &mut World) {
    let signals: Vec<PathBuf> = world
        .resource_mut::<Messages<PersistParticleTypesSignal>>()
        .drain()
        .map(|s| s.0)
        .collect();

    if signals.is_empty() {
        return;
    }

    let entities: Vec<Entity> = world
        .query_filtered::<Entity, With<ParticleType>>()
        .iter(world)
        .collect();

    if entities.is_empty() {
        warn!("No particle types found to save");
        return;
    }

    let mut builder = DynamicSceneBuilder::from_world(world);

    // Deny runtime-only components that shouldn't be persisted.
    // These are re-initialized automatically when particle types are loaded.
    #[cfg(feature = "render")]
    {
        builder = builder.deny_component::<ColorRng>();
    }
    #[cfg(feature = "movement")]
    {
        builder = builder
            .deny_component::<Momentum>()
            .deny_component::<MovementRng>();
    }
    #[cfg(feature = "reactions")]
    {
        builder = builder.deny_component::<ReactionRng>();
    }

    let scene = builder.extract_entities(entities.into_iter()).build();

    let type_registry = world.resource::<AppTypeRegistry>();
    let value = scene.serialize(&type_registry.read());
    let serialized = match value {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to serialize particle definitions: {}", e);
            return;
        }
    };

    let mut saved_paths = Vec::new();
    for signal_path in signals {
        let mut path = signal_path.clone();
        if path.extension().is_none_or(|ext| ext != "ron") {
            path.set_extension("particles.ron");
        }

        if let Err(e) =
            File::create(&path).and_then(|mut file| file.write_all(serialized.as_bytes()))
        {
            error!("Failed to write particle definitions to {:?}: {}", path, e);
            continue;
        }

        info!("Particle definitions saved to: {:?}", path);
        saved_paths.push(signal_path);
    }

    let mut msgw = world.resource_mut::<Messages<ParticleTypesPersistedSignal>>();
    for path in saved_paths {
        msgw.write(ParticleTypesPersistedSignal(path));
    }
}

/// Deserialize a RON scene file from a string.
fn deserialize_scene(
    contents: &str,
    type_registry: &AppTypeRegistry,
) -> Result<DynamicScene, Box<dyn std::error::Error>> {
    let mut deserializer = ron::de::Deserializer::from_str(contents)?;
    let scene_deserializer = SceneDeserializer {
        type_registry: &type_registry.read(),
    };
    Ok(scene_deserializer.deserialize(&mut deserializer)?)
}

/// System to load particle types from RON scene file.
#[allow(clippy::needless_pass_by_value)]
fn msgr_load_particle_types(
    mut msgr_load_particle_types: MessageReader<LoadParticleTypesSignal>,
    mut msgw_particle_types_loaded: MessageWriter<ParticleTypesLoadedSignal>,
    app_type_registry: Res<AppTypeRegistry>,
    mut scene_spawner: ResMut<SceneSpawner>,
    mut scenes: ResMut<Assets<DynamicScene>>,
) {
    for signal in msgr_load_particle_types.read() {
        let path = &signal.0;

        if !path.exists() {
            error!("Particle types file does not exist: {:?}", path);
            continue;
        }

        let scene = match std::fs::read_to_string(path)
            .map_err(std::convert::Into::into)
            .and_then(|contents| deserialize_scene(&contents, &app_type_registry))
        {
            Ok(scene) => scene,
            Err(e) => {
                error!("Failed to load particle types from {:?}: {}", path, e);
                continue;
            }
        };

        let scene_handle = scenes.add(scene);
        scene_spawner.spawn_dynamic(scene_handle);

        info!("Successfully loaded particle types from {:?}", path);
        msgw_particle_types_loaded.write(ParticleTypesLoadedSignal(path.clone()));
    }
}
