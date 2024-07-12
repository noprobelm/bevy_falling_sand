use crate::{
    ParticleColor, Particle
};
use bevy::prelude::*;

/// Colors newly added or changed particles
pub fn color_particles(
    mut particle_query: Query<
        (&mut Sprite, &ParticleColor),
        Changed<Particle>,
    >,
) {
    particle_query
        .iter_mut()
        .for_each(|(mut sprite, color)| {
            sprite.color = color.0;
        });
}
