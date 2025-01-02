use bevy::prelude::*;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            ParticleSimulationSet.run_if(resource_exists::<SimulationRun>),
        );
        app.init_resource::<SimulationRun>();
    }
}

#[derive(Resource, Default)]
pub struct SimulationRun;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleSimulationSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleDebugSet;
