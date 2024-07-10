use crate::{
    ParticleColor, Particle, Sleeping
};
use bevy::prelude::*;

#[allow(dead_code)]
pub fn color_particles_debug(
    mut particle_query: Query<(&mut Sprite, &ParticleColor, Option<&Sleeping>)>,
) {
    particle_query
        .par_iter_mut()
        .for_each(|(mut sprite, color, hibernating)| {
            if hibernating.is_some() {
                sprite.color = Color::RED;
            } else {
                sprite.color = color.0;
            }
        });
}

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
