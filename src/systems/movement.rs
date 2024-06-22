use crate::*;
use bevy_turborand::prelude::*;

#[allow(unused_mut)]
pub fn handle_particles_transform(
    mut particle_query: Query<(&mut Coordinates, &mut Transform, &mut LastMoved)>,
    map: Res<ParticleMap>,
    time: Res<Time>,
) {
    map.par_iter().for_each(|(coords, entity)| unsafe {
        let (mut coordinates, mut transform, mut last_moved) =
            particle_query.get_unchecked(*entity).unwrap();
        transform.translation.x = coords.x as f32;
        transform.translation.y = coords.y as f32;
        if coordinates.0 != *coords {
            coordinates.0 = *coords
        } else {
            last_moved.0.tick(time.delta());
        }
    });
}

pub fn handle_velocity(mut particle_query: Query<(&LastMoved, &mut Velocity)>) {
    particle_query
        .par_iter_mut()
        .for_each(|(last_moved, mut velocity)| {
            if last_moved.0.elapsed_secs() == 0. {
                velocity.increment();
            } else {
                velocity.decrement();
            }
        })
}
