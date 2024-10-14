use bevy::prelude::*;
use bevy_falling_sand::debug::DebugParticles;

/// UI plugin
pub(super) struct DebugPlugin;

impl bevy::prelude::Plugin for DebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<DebugParticles>();
    }
}

/// UI for showing `bevy_falling_sand` debug capability.
pub struct DebugUI;

impl DebugUI {
    /// Render the debug UI
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        debug_particles: &Option<Res<DebugParticles>>,
        total_particle_count: u64,
        commands: &mut Commands,
    ) {
        let mut debugging = debug_particles.is_some();
        if ui.checkbox(&mut debugging, "Debug Mode").clicked() {
            if debugging {
                commands.init_resource::<DebugParticles>();
            } else {
                commands.remove_resource::<DebugParticles>();
            }
        }

        if debug_particles.is_some() {
            ui.label(format!("Total Particles: {}", total_particle_count));
        }
    }
}
