use bevy::prelude::*;
use crate::*;

pub fn handle_particles(
    mut commands: Commands,
    particle_query: Query<(
        Entity,
        &ParticleType,
        &Parent,
        &Coordinates,
        &Velocity,
        Option<&Momentum>,
        Option<&Hibernating>,
    )>,
    parent_query: Query<
        (&Density, Option<&Applies>, Option<&Receives>, &Neighbors),
        (With<ParticleParent>, Without<Anchored>),
    >,
    chunk_groups: Res<ChunkGroups>,
    mut rng: ResMut<PhysicsRng>,
    mut map: ResMut<ParticleMap>,
) {

}
