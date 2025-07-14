use bevy::prelude::*;
use bevy_falling_sand::prelude::*;

pub fn ev_clear_particle_map(mut ev_clear_particle_map: EventWriter<ClearParticleMapEvent>) {
    ev_clear_particle_map.write(ClearParticleMapEvent);
}

pub fn ev_clear_dynamic_particles(
    mut ev_clear_dynamic_particles: EventWriter<ClearDynamicParticlesEvent>,
) {
    ev_clear_dynamic_particles.write(ClearDynamicParticlesEvent);
}

pub fn change_movement_source(
    state: Res<State<MovementSource>>,
    mut next_state: ResMut<NextState<MovementSource>>,
) {
    match state.get() {
        MovementSource::Particles => next_state.set(MovementSource::Chunks),
        MovementSource::Chunks => next_state.set(MovementSource::Particles),
    }
}

pub fn toggle_debug_map(mut commands: Commands, debug_map: Option<Res<DebugParticleMap>>) {
    if debug_map.is_some() {
        commands.remove_resource::<DebugParticleMap>();
    } else {
        commands.init_resource::<DebugParticleMap>();
    }
}

pub fn toggle_debug_dirty_rects(
    mut commands: Commands,
    debug_dirty_rects: Option<Res<DebugDirtyRects>>,
) {
    if debug_dirty_rects.is_some() {
        commands.remove_resource::<DebugDirtyRects>();
    } else {
        commands.init_resource::<DebugDirtyRects>();
    }
}
