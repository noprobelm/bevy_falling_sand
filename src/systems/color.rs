use crate::{Particle, ParticleColor, PhysicsRng, RandomColors, RandomizeColors};
use bevy::prelude::*;

/// Colors newly added or changed particles
pub fn color_particles(
    mut particle_query: Query<(&mut Sprite, &ParticleColor), Changed<ParticleColor>>,
) {
    particle_query.iter_mut().for_each(|(mut sprite, color)| {
        sprite.color = color.selected;
    });
}

/// Updates the color of a randomly colored particle.
pub fn color_random_particles(
    mut random_color_query: Query<(&mut ParticleColor, &RandomColors, &mut PhysicsRng)>,
) {
    random_color_query
        .iter_mut()
        .for_each(|(mut color, colors, mut rng)| {
            color.selected = colors.random_with_color_rng(&mut rng);
        });
}

/// Updates the color of particles with the RandomizeColors component
pub fn color_randomizing_particles(
    mut color_query: Query<(&RandomizeColors, &mut ParticleColor, &mut PhysicsRng), With<Particle>>,
) {
    color_query.iter_mut().for_each(|(random_colors, mut color, mut rng)| {
	if rng.chance(random_colors.chance) {
	    color.selected = *rng.sample(&color.palette).unwrap();
	}
    });
}

/// Flags the Particle component as changed so its color will be reset by the handle_new_particles system.
pub fn on_remove_random_colors(
    trigger: Trigger<OnRemove, RandomColors>,
    mut particles_query: Query<&mut Particle>,
) {
    let particle = particles_query.get_mut(trigger.entity()).unwrap();
    particle.into_inner();
}

