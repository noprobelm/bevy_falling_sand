use bevy::prelude::*;
use bevy_falling_sand::prelude::*;

pub fn disable_chunk_loading(mut commands: Commands) {
    commands.remove_resource::<ChunkLoadingRun>();
}

pub fn ev_clear_particle_map(mut ev_clear_particle_map: MessageWriter<DespawnAllParticlesSignal>) {
    ev_clear_particle_map.write(DespawnAllParticlesSignal);
}

pub fn ev_clear_dynamic_particles(
    mut ev_clear_dynamic_particles: MessageWriter<DespawnDynamicParticlesSignal>,
) {
    ev_clear_dynamic_particles.write(DespawnDynamicParticlesSignal);
}

pub fn change_movement_source(
    state: Res<State<MovementSystemState>>,
    mut next: ResMut<NextState<MovementSystemState>>,
) {
    let new_state = match state.get() {
        MovementSystemState::Particles => MovementSystemState::Chunks,
        MovementSystemState::Chunks => MovementSystemState::Particles,
    };
    next.set(new_state);
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
