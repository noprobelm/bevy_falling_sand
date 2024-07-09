use bevy::prelude::*;

#[derive(Component)]
pub struct ChunkID(pub usize);

#[derive(Component)]
pub struct Idle;

#[derive(Component)]
pub struct Moved;
