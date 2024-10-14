use bevy::{
    input::common_conditions::{input_just_pressed, input_pressed},
    prelude::*,
    utils::{Entry, HashMap},
};
use bevy_egui::EguiContexts;
use bevy_falling_sand::movement::*;
use bevy_falling_sand::core::*;

use crate::*;

/// Particle Management Plugin
pub(super) struct ParticleManagementPlugin;

impl bevy::prelude::Plugin for ParticleManagementPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<SelectedParticle>()
            .init_resource::<ParticleList>()
            .init_resource::<ParticleTypeList>();

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
                .before(ParticleSimulationSet)
                .after(update_cursor_coordinates),
        );
        app.add_systems(
            Update,
            toggle_simulation.run_if(input_just_pressed(KeyCode::Space)),
        );
    }
}

/// A list of particle types organized by material type.
#[derive(Resource, Default)]
pub struct ParticleTypeList {
    map: HashMap<String, Vec<String>>,
}

impl ParticleTypeList {
    /// Insert a list of particles into the map for a given material. If the material already exists, modify the
    /// existing list. Lists are sorted after each call to this method.
    pub fn insert_or_modify(&mut self, material: String, particles: Vec<String>) {
        match self.map.entry(material) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().extend(particles);
                entry.get_mut().sort();
            }
            Entry::Vacant(entry) => {
                let mut sorted_particles = particles;
                sorted_particles.sort();
                entry.insert(sorted_particles);
            }
        }
    }
}

/// HashMap keys are unordered. This will ensure an ordered list of particles is available.
#[derive(Resource, Default)]
pub struct ParticleList {
    pub particle_list: Vec<String>,
}

impl ParticleList {
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
        particle_type_list: &Res<ParticleTypeList>,
        selected_particle: &mut ResMut<SelectedParticle>,
        brush_state: &mut ResMut<NextState<BrushState>>,
        commands: &mut Commands,
    ) {
        ui.vertical(|ui| {
            // Define the fixed order of categories
            let categories = ["Walls", "Solids", "Movable Solids", "Liquids", "Gases"];

            // Iterate through categories in a deterministic order
            for &category in &categories {
                if let Some(particles) = particle_type_list.map.get(category) {
                    egui::CollapsingHeader::new(category) // Use the category as the header title
                        .default_open(false)
                        .show(ui, |ui| {
                            particles.iter().for_each(|particle_name| {
                                // Create a button for each particle name
                                if ui.button(particle_name).clicked() {
                                    selected_particle.0 = particle_name.clone();
                                    brush_state.set(BrushState::Spawn);
                                }
                            });
                        });
                }
            }

            // Existing UI elements for Remove and Despawn All Particles
            ui.horizontal_wrapped(|ui| {
                if ui.button("Remove").clicked() {
                    brush_state.set(BrushState::Despawn);
                }
            });

            if ui.button("Despawn All Particles").clicked() {
                commands.trigger(ClearMapEvent);
            }
        });
    }
}

pub fn update_particle_list(
    new_particle_query: Query<
        (
            &ParticleType,
            Option<&Wall>,
            Option<&MovableSolid>,
            Option<&Solid>,
            Option<&Liquid>,
            Option<&Gas>,
        ),
        Added<ParticleType>,
    >,
    mut particle_list: ResMut<ParticleList>,
    mut particle_type_list: ResMut<ParticleTypeList>,
) {
    new_particle_query.iter().for_each(
        |(particle_type, wall, movable_solid, solid, liquid, gas)| {
            // Add the particle type name to the particle_list
            particle_list.push(particle_type.name.clone());

            // Check for the presence of each optional component and update particle_type_list accordingly
            if wall.is_some() {
                particle_type_list
                    .insert_or_modify("Walls".to_string(), vec![particle_type.name.clone()]);
            }
            if movable_solid.is_some() {
                particle_type_list.insert_or_modify(
                    "Movable Solids".to_string(),
                    vec![particle_type.name.clone()],
                );
            }
            if solid.is_some() {
                particle_type_list
                    .insert_or_modify("Solids".to_string(), vec![particle_type.name.clone()]);
            }
            if liquid.is_some() {
                particle_type_list
                    .insert_or_modify("Liquids".to_string(), vec![particle_type.name.clone()]);
            }
            if gas.is_some() {
                particle_type_list
                    .insert_or_modify("Gases".to_string(), vec![particle_type.name.clone()]);
            }
        },
    );
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
