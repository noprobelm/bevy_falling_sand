use bevy::{
    input::common_conditions::{input_just_pressed, input_pressed},
    prelude::*,
};
use bevy_falling_sand::prelude::*;

use crate::{
    app_state::{
        step_simulation, toggle_particle_movement_logic, toggle_resource, toggle_simulation_run,
        AppState, CanvasState,
    },
    brush::{
        despawn_particles, resize_brush, spawn_particles, update_brush_spawn_state,
        update_brush_type_state, BrushMode,
    },
    cursor::update_cursor_position,
};

use super::{
    overlays::{draw_cursor_guide, DrawCursorGuide},
    RenderGui,
};

pub(super) struct QuickActionsPlugin;

impl Plugin for QuickActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Resource toggling
                toggle_resource::<RenderGui>
                    .run_if(input_just_pressed(KeyCode::KeyH).and(in_state(AppState::Canvas))),
                toggle_resource::<DebugParticleMap>.run_if(input_just_pressed(KeyCode::F1)),
                toggle_resource::<DrawCursorGuide>.run_if(input_just_pressed(KeyCode::F2)),
                toggle_resource::<DebugDirtyRects>.run_if(input_just_pressed(KeyCode::F3)),
                toggle_particle_movement_logic.run_if(input_just_pressed(KeyCode::F4)),
                toggle_simulation_run
                    .run_if(input_just_pressed(KeyCode::Space))
                    .run_if(in_state(AppState::Canvas)),
                // Step simulation
                step_simulation.run_if(input_just_pressed(KeyCode::Enter)),
                // Brush actions
                update_brush_type_state.run_if(input_just_pressed(MouseButton::Back)),
                update_brush_spawn_state.run_if(input_just_pressed(MouseButton::Right)),
                resize_brush.run_if(in_state(AppState::Canvas).and(in_state(CanvasState::Edit))),
                // Spawning/despawning
                spawn_particles
                    .run_if(input_pressed(MouseButton::Left))
                    .run_if(in_state(BrushMode::Spawn))
                    .run_if(in_state(AppState::Canvas))
                    .before(ParticleSimulationSet)
                    .after(update_cursor_position),
                despawn_particles
                    .run_if(input_pressed(MouseButton::Left))
                    .run_if(in_state(BrushMode::Despawn))
                    .run_if(in_state(AppState::Canvas))
                    .before(ParticleSimulationSet)
                    .after(update_cursor_position),
                draw_cursor_guide
                    .run_if(resource_exists::<DrawCursorGuide>)
                    .after(ParticleDebugSet),
            ),
        );
    }
}
