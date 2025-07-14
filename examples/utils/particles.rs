use bevy::prelude::*;
use bevy_falling_sand::prelude::*;

pub fn reset_dynamic_particles(
    mut ev_clear_dynamic_particles: EventWriter<ClearDynamicParticlesEvent>,
) {
    ev_clear_dynamic_particles.write(ClearDynamicParticlesEvent);
}
