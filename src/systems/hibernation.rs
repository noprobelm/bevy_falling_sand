use crate::*;
use bevy::prelude::*;

pub fn wake_up_chunks(
    particle_query: Query<&Coordinates, (With<Particle>, Changed<Coordinates>)>,
    chunks_query: Query<(&ChunkID, Option<&Moved>)>,
    chunk_entity_map: Res<ChunkEntityMap>,
) {
}
pub fn hibernate_particles(
    par_commands: ParallelCommands,
    particle_query: Query<(Entity, &LastMoved, Option<&Hibernating>)>,
) {
    particle_query
        .par_iter()
        .for_each(|(entity, last_moved, hibernating)| {
            if hibernating.is_none() && last_moved.0.elapsed().as_millis() > 300 {
                par_commands.command_scope(|mut commands| {
                    commands.entity(entity).insert(Hibernating::default());
                });
            } else if hibernating.is_some() && last_moved.0.elapsed().as_millis() < 300 {
                par_commands.command_scope(|mut commands| {
                    commands.entity(entity).remove::<Hibernating>();
                });
            }
        });
}
