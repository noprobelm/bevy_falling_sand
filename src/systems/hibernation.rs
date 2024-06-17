use crate::{Coordinates, Hibernating, LastMoved};
use bevy::prelude::*;

pub fn handle_hibernation(
    mut commands: Commands,
    particle_query: Query<(Entity, &LastMoved, Option<&Hibernating>)>,
) {
    for (entity, last_moved, hibernating) in particle_query.iter() {
        if hibernating.is_some() && last_moved.0.elapsed_secs() < 0.3 {
            commands.entity(entity).remove::<Hibernating>();
        } else if hibernating.is_none() && last_moved.0.elapsed_secs() > 1.0 {
            commands.entity(entity).insert(Hibernating::default());
        }
    }
}

pub fn tick_hibernation_timer(mut particle_query: Query<&mut Hibernating>, time: Res<Time>) {
    particle_query.par_iter_mut().for_each(|mut hibernating| {
        hibernating.0.tick(time.delta());
    });
}

pub fn reset_last_moved(mut particle_query: Query<&mut LastMoved, Changed<Coordinates>>) {
    particle_query.par_iter_mut().for_each(|mut last_moved| {
        last_moved.0.reset();
    })
}
