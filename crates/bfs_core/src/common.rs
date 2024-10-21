use bevy::prelude::*;

/// Common utilities that have no direct relation to particles, such as flag resources and system sets.
pub struct CommonUtilitiesPlugin;

impl Plugin for CommonUtilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            ParticleSimulationSet.run_if(resource_exists::<SimulationRun>),
        );
        app.init_resource::<SimulationRun>();
    }
}

/// Conditional systems that are considered part of the particle simulation should check if this
/// resource exists.
#[derive(Resource, Default)]
pub struct SimulationRun;

/// System set for systems that influence particle management.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleSimulationSet;

/// System set for systems that provide debugging functionality.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleDebugSet;
