//! Events for saving/loading particle scenes.
use std::path::PathBuf;
use bevy::prelude::*;
use bfs_core::MutateParticleEvent;

pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadSceneEvent>()
            .add_event::<SaveSceneEvent>()
            .add_event::<MutateParticleEvent>();
    }
}

/// Triggers [save_scene_system](crate::save_scene_system) to save all particles in the world to the specified PathBuf.
#[derive(Event)]
pub struct SaveSceneEvent(pub PathBuf);

/// Triggers [load_scene_system](crate::load_scene_system) to load all particles in the world from the specified
/// PathBuf.
#[derive(Event)]
pub struct LoadSceneEvent(pub PathBuf);
