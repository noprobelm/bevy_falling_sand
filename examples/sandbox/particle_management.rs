use bevy::{
    input::common_conditions::{input_just_pressed, input_pressed},
    prelude::*,
};
use bevy_egui::EguiContexts;
use bevy_falling_sand::core::*;

use crate::*;

/// Particle Management Plugin
pub(super) struct ParticleManagementPlugin;

impl bevy::prelude::Plugin for ParticleManagementPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<SelectedParticle>();
        app.add_systems(
            Update,
            spawn_particles
                .run_if(input_pressed(MouseButton::Left))
                .run_if(in_state(BrushState::Spawn))
                .run_if(in_state(AppState::Canvas))
                .after(update_cursor_coordinates),
        );
        app.add_systems(
            Update,
            despawn_particles
                .run_if(input_pressed(MouseButton::Left))
                .run_if(in_state(BrushState::Despawn))
                .run_if(in_state(AppState::Canvas))
                .before(ParticleSimulationSet)
                .after(update_cursor_coordinates),
        );
        app.add_systems(
            Update,
            toggle_simulation.run_if(input_just_pressed(KeyCode::Space)),
        );
    }
}

/// The currently selected particle for spawning.
#[derive(Resource)]
pub struct SelectedParticle(pub String);

impl Default for SelectedParticle {
    fn default() -> SelectedParticle {
        SelectedParticle("Dirt Wall".to_string())
    }
}

/// Spawns particles using current brush position and size information.
pub fn spawn_particles(
    mut commands: Commands,
    cursor_coords: Res<CursorCoords>,
    selected: Res<SelectedParticle>,
    brush_type: Res<State<BrushType>>,
    brush_query: Query<&Brush>,
) {
    let brush = brush_query.single();
    let brush_type = brush_type.get();
    brush_type.spawn_particles(
        &mut commands,
        cursor_coords,
        brush.size as f32,
        Particle {
            name: selected.0.clone(),
        },
    );
}

/// Despawns particles using current brush position and size information.
pub fn despawn_particles(
    mut commands: Commands,
    cursor_coords: Res<CursorCoords>,
    brush_type: Res<State<BrushType>>,
    brush_query: Query<&Brush>,
    mut contexts: EguiContexts,
) {
    let ctx = contexts.ctx_mut();
    if ctx.is_pointer_over_area() {
        return;
    }

    let brush = brush_query.single();
    let brush_size = brush.size;

    brush_type.remove_particles(&mut commands, cursor_coords.current.as_ivec2(), brush_size as f32)
}

/// Stops or starts the simulation when scheduled.
pub fn toggle_simulation(mut commands: Commands, simulation_pause: Option<Res<SimulationRun>>) {
    if simulation_pause.is_some() {
        commands.remove_resource::<SimulationRun>();
    } else {
        commands.init_resource::<SimulationRun>();
    }
}
