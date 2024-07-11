use crate::{
    ParticleColor, Particle
};
use bevy::prelude::*;

pub fn color_particles(
    mut particle_query: Query<
        (&mut Sprite, &ParticleColor),
        Added<Particle>,
    >,
) {
    particle_query
        .par_iter_mut()
        .for_each(|(mut sprite, color)| {
            sprite.color = color.0;
        });
}
