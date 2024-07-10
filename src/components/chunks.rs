use bevy::prelude::*;

#[derive(Component)]
pub struct ChunkID(pub usize);

#[derive(Component)]
pub struct Sleeping;

#[derive(Component)]
pub struct Moved;
