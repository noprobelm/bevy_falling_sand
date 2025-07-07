use bevy::prelude::*;
use bfs_core::{Particle, ParticleRng, ParticleSimulationSet};

use super::{ChangesColor, ColorProfile, ColorRng};

pub(super) struct SystemsPlugin;

impl Plugin for SystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (color_particles, handle_changes_color).in_set(ParticleSimulationSet),
        );
    }
}

fn color_particles(mut particle_query: Query<(&mut Sprite, &ColorProfile), Changed<ColorProfile>>) {
    particle_query.iter_mut().for_each(|(mut sprite, color)| {
        sprite.color = color.color;
    });
}

fn handle_changes_color(
    mut particles_query: Query<(&mut ColorProfile, &mut ColorRng, &ChangesColor), With<Particle>>,
) {
    particles_query
        .iter_mut()
        .for_each(|(mut particle_color, mut rng, flows_color)| {
            if rng.chance(flows_color.chance) {
                particle_color.set_next();
            }
        })
}
