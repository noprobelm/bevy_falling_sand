use bevy::prelude::*;
use std::path::PathBuf;

/// Sends an event with a path to a RON file containing particle types for the deserializer.
#[derive(Event)]
pub struct DeserializeParticleTypesEvent(pub PathBuf);
