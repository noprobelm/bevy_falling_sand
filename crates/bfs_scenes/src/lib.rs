#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(
    clippy::nursery,
    clippy::pedantic,
    nonstandard_style,
    rustdoc::broken_intra_doc_links,
    missing_docs
)]
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]
//! Provides scene loading and saving functionality for the Falling Sand simulation.

use bevy::prelude::*;
use bfs_assets::ParticleSceneAsset;
use bfs_core::{Particle, ParticlePosition};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// Provides the constructs and systems necessary for saving and loading particle scenes in the
/// Falling Sand simulation.
pub struct FallingSandScenesPlugin;

impl Plugin for FallingSandScenesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadSceneAssetEvent>()
            .add_event::<SaveSceneEvent>()
            .add_event::<LoadSceneEvent>()
            .add_systems(
                Update,
                (
                    save_scene_system.run_if(on_event::<SaveSceneEvent>),
                    load_scene_system.run_if(on_event::<LoadSceneEvent>),
                    load_scene_asset_system.run_if(on_event::<LoadSceneAssetEvent>),
                ),
            );
    }
}

/// Particle type and position data for saving/loading scenes.
#[derive(Serialize, Deserialize)]
pub struct RuntimeParticleSceneData {
    /// The particle type.
    pub particle: Particle,
    /// The particle position.
    pub position: ParticlePosition,
}

/// [`ParticleData`] wrapped in a scene for loading.
#[derive(Serialize, Deserialize)]
pub struct ParticleScene {
    /// The particles the scene is composed of.
    pub particles: Vec<RuntimeParticleSceneData>,
}

/// Triggers systems to save the current particle scene to a file.
#[derive(Event)]
pub struct SaveSceneEvent(pub PathBuf);

/// Triggers systems to load particles from a file.
#[derive(Event)]
pub struct LoadSceneEvent(pub PathBuf);


/// Triggers systems to load particles from an asset handle.
#[derive(Event)]
pub struct LoadSceneAssetEvent(pub Handle<ParticleSceneAsset>);

fn save_scene_system(
    particle_query: Query<(&Particle, &ParticlePosition)>,
    mut ev_save_scene: EventReader<SaveSceneEvent>,
) {
    for ev in ev_save_scene.read() {
        let particles: Vec<RuntimeParticleSceneData> = particle_query
            .iter()
            .map(|(particle_type, position)| RuntimeParticleSceneData {
                particle: particle_type.clone(),
                position: *position,
            })
            .collect();

        let particle_scene = ParticleScene { particles };
        let ron_string = ron::to_string(&particle_scene).unwrap();
        File::create(ev.0.clone())
            .and_then(|mut file| file.write_all(ron_string.as_bytes()))
            .expect("Error while writing scene to file");
    }
}

fn load_scene_system(mut commands: Commands, mut ev_load_scene: EventReader<LoadSceneEvent>) {
    for ev in ev_load_scene.read() {
        let file = File::open(ev.0.clone()).expect("Failed to open RON file");
        let particle_scene: ParticleScene = ron::de::from_reader(file).expect("Failed to load RON file");

        for particle_data in particle_scene.particles {
            let transform = Transform::from_xyz(
                particle_data.position.0.x as f32,
                particle_data.position.0.y as f32,
                0.,
            );

            commands.spawn((particle_data.particle.clone(), transform));
        }
    }
}

fn load_scene_asset_system(
    mut commands: Commands,
    mut ev_load_scene_asset: EventReader<LoadSceneAssetEvent>,
    scene_assets: Res<Assets<ParticleSceneAsset>>,
) {
    for ev in ev_load_scene_asset.read() {
        if let Some(scene_asset) = scene_assets.get(&ev.0) {
            info!("Loading scene with {} particles", scene_asset.len());
            
            for particle_data in &scene_asset.particles {
                let transform = Transform::from_xyz(
                    particle_data.position[0] as f32,
                    particle_data.position[1] as f32,
                    0.,
                );

                let particle = Particle::new(Box::leak(particle_data.particle.clone().into_boxed_str()));
                commands.spawn((particle, transform));
            }
        } else {
            warn!("Scene asset not found or not loaded yet");
        }
    }
}
