/// Events for saving/loading particle scenes.
use bevy::prelude::*;
use std::path::PathBuf;

#[derive(Event)]
pub struct SaveSceneEvent(pub PathBuf);

#[derive(Event)]
pub struct LoadSceneEvent(pub PathBuf);
