use bevy::prelude::*;
use bfs_core::{Particle, ParticleSimulationSet};

use super::{FlowsColor, ParticleColor, RandomizesColor, ColorRng};

pub struct SystemsPlugin;

impl Plugin for SystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                color_particles,
                color_flowing_particles,
                color_randomizing_particles,
            )
                .in_set(ParticleSimulationSet),
        );
    }
}

pub fn color_particles(
    mut particle_query: Query<(&mut Sprite, &ParticleColor), Changed<ParticleColor>>,
) {
    particle_query.iter_mut().for_each(|(mut sprite, color)| {
        sprite.color = color.selected;
    });
}

pub fn color_flowing_particles(
    mut particles_query: Query<(&mut ParticleColor, &mut ColorRng, &FlowsColor), With<Particle>>,
) {
    particles_query
        .iter_mut()
        .for_each(|(mut particle_color, mut rng, flows_color)| {
            if rng.chance(flows_color.rate) {
                particle_color.set_next();
            }
        })
}

pub fn color_randomizing_particles(
    mut particles_query: Query<
        (&mut ParticleColor, &mut ColorRng, &RandomizesColor),
        With<Particle>,
    >,
) {
    particles_query
        .iter_mut()
        .for_each(|(mut particle_color, mut rng, randomizes_color)| {
            if rng.chance(randomizes_color.rate) {
                particle_color.randomize(&mut rng);
            }
        })
}
