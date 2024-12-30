use bevy::prelude::*;

use crate::{Particle, MutateParticleEvent, ParticleSimulationSet};

pub struct ParticleSystemsPlugin;

impl Plugin for ParticleSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ev_mutate_particle.in_set(ParticleSimulationSet));
    }
}

pub fn ev_mutate_particle(
    mut ev_change_particle: EventReader<MutateParticleEvent>,
    mut particle_query: Query<&mut Particle>,
) {
    for ev in ev_change_particle.read() {
        let mut particle = particle_query.get_mut(ev.entity).unwrap();
        particle.name = ev.particle.name.clone();
    }
}

