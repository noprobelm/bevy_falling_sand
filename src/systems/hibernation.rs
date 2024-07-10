use crate::*;

pub fn reset_chunks(
    commands: Commands,
    mut map: ResMut<ParticleMap>,
) {
    map.sleep_chunks(commands);
    map.deactivate_all_chunks();
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
