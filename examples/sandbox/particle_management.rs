use bevy::{
    input::common_conditions::{input_just_pressed, input_pressed},
    prelude::*,
};
use bevy_egui::EguiContexts;
use bevy_falling_sand::{ClearChunkMapEvent, Particle, SimulationRun};

use crate::*;

/// Particle Management Plugin
pub(super) struct ParticleManagementPlugin;

impl bevy::prelude::Plugin for ParticleManagementPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<SelectedParticle>()
            .init_resource::<ParticleList>();

	app.add_systems(Update, update_particle_list);
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
                .before(handle_particles)
                .after(update_cursor_coordinates),
        );
        app.add_systems(
            Update,
            toggle_simulation.run_if(input_just_pressed(KeyCode::Space)),
        );
    }
}

/// HashMap keys are unordered. This will ensure an ordered list of particles is available.
#[derive(Resource, Default)]
pub struct ParticleList {
    pub particle_list: Vec<String>,
}

impl ParticleList {
    /// Iterates through all particles in the ParticleList. This is used to construct buttons in the UI.
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.particle_list.iter()
    }

    /// Adds to the ParticleList.
    pub fn push(&mut self, value: String) {
        self.particle_list.push(value);
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

/// UI for particle control mechanics.
pub struct ParticleControlUI;

impl ParticleControlUI {
    /// Renders the particle control UI
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        particle_types: &ParticleList,
        selected_particle: &mut SelectedParticle,
        brush_state: &mut ResMut<NextState<BrushState>>,
        commands: &mut Commands,
    ) {
        ui.horizontal_wrapped(|ui| {
            particle_types.iter().for_each(|particle| {
                if ui.button(particle).clicked() {
                    selected_particle.0 = particle.clone();
                    brush_state.set(BrushState::Spawn);
                }
            });
            if ui.button("Remove").clicked() {
                brush_state.set(BrushState::Despawn);
            }
        });

        ui.separator();

        if ui.button("Despawn All Particles").clicked() {
            commands.trigger(ClearChunkMapEvent);
        }
    }
}

pub fn update_particle_list(new_particle_query: Query<&ParticleType, Added<ParticleType>>, mut particle_list: ResMut<ParticleList>) {
    new_particle_query.iter().for_each(|particle_type| {
	particle_list.push(particle_type.name.clone())
    });
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
        cursor_coords.0,
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

    brush_type.remove_particles(&mut commands, cursor_coords.0.as_ivec2(), brush_size as f32)
}

/// Stops or starts the simulation when scheduled.
pub fn toggle_simulation(mut commands: Commands, simulation_pause: Option<Res<SimulationRun>>) {
    if simulation_pause.is_some() {
        commands.remove_resource::<SimulationRun>();
    } else {
        commands.init_resource::<SimulationRun>();
    }
}
