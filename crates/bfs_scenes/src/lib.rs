mod events;

use bevy::prelude::*;
use bfs_core::{Coordinates, MutateParticleEvent, Particle};
use ron::de::from_reader;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

pub use events::*;

/// Core plugin for Bevy Falling Sand.
pub struct FallingSandScenesPlugin;

impl Plugin for FallingSandScenesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadSceneEvent>()
            .add_event::<SaveSceneEvent>()
            .add_event::<MutateParticleEvent>()
            .add_systems(
                Update,
                save_scene_system.run_if(on_event::<crate::events::SaveSceneEvent>()),
            )
            .add_systems(
                Update,
                load_scene_system.run_if(on_event::<crate::events::LoadSceneEvent>()),
            );
    }
}

/// Systems for loading and saving particle scenes.

/// Particle data for loading scenes.
#[derive(Serialize, Deserialize)]
pub struct ParticleData {
    /// The particle type to load.
    pub particle_type: Particle,
    /// The coordinates of the particle.
    pub coordinates: Coordinates,
}

/// A collection of particles that make up a scene.
#[derive(Serialize, Deserialize)]
pub struct ParticleScene {
    /// The particles to load.
    pub particles: Vec<ParticleData>,
}

/// Saves a scene to the PathBuf specified by [`SaveSceneEvent`](crate::SaveSceneEvent)
pub fn save_scene_system(
    particle_query: Query<(&Particle, &Coordinates)>,
    mut ev_save_scene: EventReader<SaveSceneEvent>,
) {
    for ev in ev_save_scene.read() {
        let particles: Vec<ParticleData> = particle_query
            .iter()
            .map(|(particle_type, coordinates)| ParticleData {
                particle_type: particle_type.clone(),
                coordinates: *coordinates,
            })
            .collect();

        let particle_scene = ParticleScene { particles };
        let ron_string = ron::to_string(&particle_scene).unwrap();
        File::create(ev.0.clone())
            .and_then(|mut file| file.write(ron_string.as_bytes()))
            .expect("Error while writing scene to file");
    }
}

/// Loads a scene from the PathBuf specified by [`LoadSceneEvent`](crate::LoadSceneEvent)
pub fn load_scene_system(mut commands: Commands, mut ev_load_scene: EventReader<LoadSceneEvent>) {
    for ev in ev_load_scene.read() {
        let file = File::open(ev.0.clone()).expect("Failed to open RON file");
        let particle_scene: ParticleScene = from_reader(file).expect("Failed to load RON file");

        for particle_data in particle_scene.particles {
            let transform = Transform::from_xyz(
                particle_data.coordinates.0.x as f32,
                particle_data.coordinates.0.y as f32,
                0.,
            );

            commands.spawn((
                particle_data.particle_type.clone(),
                SpatialBundle::from_transform(transform),
            ));
        }
    }
}
