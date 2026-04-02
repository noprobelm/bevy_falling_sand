//! Signals for controlling simulation progression.
use bevy::prelude::*;

pub(super) struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticleSimulationRun>()
            .add_message::<SimulationStepSignal>();
    }
}

/// Marker resource used to control particle simulation systems.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::core::ParticleSimulationRun;
///
/// fn check_simulation(sim: Option<Res<ParticleSimulationRun>>) {
///     if sim.is_some() {
///         println!("Simulation resource is present");
///     }
/// }
/// ```
#[derive(Resource, Default)]
pub struct ParticleSimulationRun;

/// Signal used to trigger the simulation to step forward by one tick.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::core::SimulationStepSignal;
///
/// fn step_simulation(mut writer: MessageWriter<SimulationStepSignal>) {
///     writer.write(SimulationStepSignal);
/// }
/// ```
#[derive(Event, Message, Copy, Clone, Hash, Default, Debug, Eq, PartialEq, PartialOrd)]
pub struct SimulationStepSignal;

/// Flag indicating particle simulation systems should be running.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::core::condition_msg_simulation_step_received;
///
/// fn setup(app: &mut App) {
///     app.add_systems(
///         Update,
///         my_system.run_if(condition_msg_simulation_step_received),
///     );
/// }
///
/// fn my_system() {}
/// ```
#[allow(clippy::needless_pass_by_value)]
#[must_use]
pub fn condition_msg_simulation_step_received(
    msgr_simulation_step: MessageReader<SimulationStepSignal>,
) -> bool {
    if !msgr_simulation_step.is_empty() {
        return true;
    }
    false
}
