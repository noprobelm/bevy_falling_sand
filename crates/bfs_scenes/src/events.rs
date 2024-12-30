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

#[derive(Event)]
pub struct SaveSceneEvent(pub PathBuf);

#[derive(Event)]
pub struct LoadSceneEvent(pub PathBuf);
