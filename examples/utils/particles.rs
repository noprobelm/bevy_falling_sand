use bevy::prelude::*;
use bevy_falling_sand::prelude::*;

pub fn reset_dynamic_particles(mut ev_clear_particle_map: EventWriter<ClearParticleMapEvent>) {
    ev_clear_particle_map.write(ClearParticleMapEvent);
}
