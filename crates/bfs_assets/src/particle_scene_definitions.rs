use bevy::{
    prelude::*,
    scene::{DynamicScene, DynamicSceneBuilder, SceneSpawner},
};
use bfs_core::ParticleType;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// Event to trigger saving of all particle type definitions.
#[derive(Event)]
pub struct SaveParticleDefinitionsEvent(pub PathBuf);

/// Event to trigger loading particle definitions from a scene file.
#[derive(Event)]
pub struct LoadParticleDefinitionsSceneEvent(pub PathBuf);

/// System to save all ParticleType entities and their components to a scene file.
pub fn save_particle_definitions_system(
    world: &World,
    particle_type_query: Query<Entity, With<ParticleType>>,
    mut ev_save: EventReader<SaveParticleDefinitionsEvent>,
) {
    for ev in ev_save.read() {
        let entities: Vec<Entity> = particle_type_query.iter().collect();

        if entities.is_empty() {
            warn!("No particle types found to save");
            continue;
        }

        let scene = DynamicSceneBuilder::from_world(world)
            .extract_entities(entities.into_iter())
            .build();

        let type_registry = world.resource::<AppTypeRegistry>();
        let serialized = scene
            .serialize(&type_registry.read())
            .expect("Failed to serialize particle definitions");

        let mut path = ev.0.clone();
        if !path.extension().map_or(false, |ext| ext == "ron") {
            path.set_extension("particles.scn.ron");
        }

        File::create(&path)
            .and_then(|mut file| file.write_all(serialized.as_bytes()))
            .expect("Error while writing particle definitions to file");

        info!("Particle definitions saved to: {:?}", path);
    }
}

/// System to load particle definitions from a scene file.
pub fn load_particle_definitions_scene_system(
    mut ev_load: EventReader<LoadParticleDefinitionsSceneEvent>,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    for ev in ev_load.read() {
        let path_str = ev.0.to_string_lossy();
        let relative_path = if let Some(index) = path_str.find("/assets/") {
            &path_str[index + 8..]
        } else {
            &path_str
        };

        info!("Loading particle definitions from scene: {}", relative_path);

        let scene_handle: Handle<DynamicScene> = asset_server.load(relative_path);

        scene_spawner.spawn_dynamic(scene_handle);
    }
}

