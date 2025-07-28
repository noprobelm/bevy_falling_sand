use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_falling_sand::prelude::*;

use crate::app_state::AppState;

pub(super) struct QuickActionsPlugin;

impl Plugin for QuickActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                toggle_particle_map_overlay.run_if(input_just_pressed(KeyCode::F1)),
                toggle_dirty_rects_overlay.run_if(input_just_pressed(KeyCode::F2)),
                toggle_particle_movement_logic.run_if(input_just_pressed(KeyCode::F3)),
                toggle_simulation_run.run_if(input_just_pressed(KeyCode::Space)),
                step_simulation.run_if(input_just_pressed(KeyCode::Enter)),
            ),
        );
    }
}

fn toggle_particle_map_overlay(
    mut commands: Commands,
    debug_particle_map: Option<Res<DebugParticleMap>>,
) {
    if debug_particle_map.is_some() {
        commands.remove_resource::<DebugParticleMap>();
    } else {
        commands.init_resource::<DebugParticleMap>();
    }
}

fn toggle_dirty_rects_overlay(
    mut commands: Commands,
    debug_dirty_rects: Option<Res<DebugDirtyRects>>,
) {
    if debug_dirty_rects.is_some() {
        commands.remove_resource::<DebugDirtyRects>();
    } else {
        commands.init_resource::<DebugDirtyRects>();
    }
}

fn toggle_particle_movement_logic(
    particle_movement_state_current: Res<State<MovementSource>>,
    mut particle_movement_state_next: ResMut<NextState<MovementSource>>,
) {
    match particle_movement_state_current.get() {
        MovementSource::Chunks => {
            particle_movement_state_next.set(MovementSource::Particles);
        }
        MovementSource::Particles => {
            particle_movement_state_next.set(MovementSource::Chunks);
        }
    }
}

fn toggle_simulation_run(
    mut commands: Commands,
    simulation_pause: Option<Res<ParticleSimulationRun>>,
    app_state: Res<State<AppState>>,
    mut time: ResMut<Time<Virtual>>,
) {
    if app_state.get() == &AppState::Canvas {
        if simulation_pause.is_some() {
            commands.remove_resource::<ParticleSimulationRun>();
        } else {
            commands.init_resource::<ParticleSimulationRun>();
        }
        if time.is_paused() {
            time.unpause();
        } else {
            time.pause();
        }
    }
}

fn step_simulation(mut evw_simulation_step: EventWriter<SimulationStepEvent>) {
    evw_simulation_step.write(SimulationStepEvent);
}
