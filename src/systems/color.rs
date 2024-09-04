use crate::*;

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

/// Flags the Particle component as changed so its color will be reset by the handle_new_particles system.
pub fn on_remove_random_colors(
    trigger: Trigger<OnRemove, RandomColors>,
    mut particles_query: Query<&mut Particle>,
) {
    let particle = particles_query.get_mut(trigger.entity()).unwrap();
    particle.into_inner();
}

/// Changes the color for particles with the ChangesColor component
pub fn color_flowing_particles (
    mut particles_query: Query<(&mut ParticleColor, &mut PhysicsRng, &FlowsColor), With<Particle>>) {
    particles_query.iter_mut().for_each(|(mut particle_color, mut rng, flows_color)| {
	if rng.chance(flows_color.rate) {
	    particle_color.set_next();
	}
    })
}

/// Randomizes the color for particles with the ChangesColor component
pub fn color_randomizing_particles (
    mut particles_query: Query<(&mut ParticleColor, &mut PhysicsRng, &RandomizesColor), With<Particle>>) {
    particles_query.iter_mut().for_each(|(mut particle_color, mut rng, randomizes_color)| {
	if rng.chance(randomizes_color.rate) {
	    particle_color.randomize(&mut rng);
	}
    })
}
