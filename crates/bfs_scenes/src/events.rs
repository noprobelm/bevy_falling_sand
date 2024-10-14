//! Events for saving/loading particle scenes.
use bevy::prelude::Event;
use std::path::PathBuf;

/// Triggers [save_scene_system](crate::save_scene_system) to save all particles in the world to the specified PathBuf.
#[derive(Event)]
pub struct SaveSceneEvent(pub PathBuf);

/// Triggers [load_scene_system](crate::load_scene_system) to load all particles in the world from the specified
/// PathBuf.
#[derive(Event)]
pub struct LoadSceneEvent(pub PathBuf);
