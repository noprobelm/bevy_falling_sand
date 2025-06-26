use bevy::prelude::*;
use std::path::PathBuf;

pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadSceneEvent>()
            .add_event::<SaveSceneEvent>();
    }
}

#[derive(Event)]
pub struct SaveSceneEvent(pub PathBuf);

#[derive(Event)]
pub struct LoadSceneEvent(pub PathBuf);
