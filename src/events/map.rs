use bevy::prelude::*;

#[derive(Event)]
pub struct RemoveParticle {
    pub coordinates: IVec2
}

#[derive(Event)]
pub struct ClearChunkMap;
