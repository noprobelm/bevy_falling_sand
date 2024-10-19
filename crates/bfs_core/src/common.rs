use bevy::prelude::{Resource, SystemSet, Plugin, App};

/// Common utilities that have no direct relation to particles, such as flag resources and system sets.
pub struct CommonUtilitiesPlugin;

impl Plugin for CommonUtilitiesPlugin {
    fn build(&self, app: &mut App) {
	app.init_resource::<SimulationRun>();
    }
}

/// Resource to insert for running the simulation
#[derive(Resource, Default)]
pub struct SimulationRun;

/// System set for systems that influence particle management.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleSimulationSet;

/// System set for systems that provide debugging functionality.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleDebugSet;
