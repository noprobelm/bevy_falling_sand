use bevy::prelude::*;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationRun>().configure_sets(
            Update,
            ParticleSimulationSet.run_if(resource_exists::<SimulationRun>),
        );
    }
}

#[derive(Resource, Default)]
pub struct SimulationRun;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleSimulationSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleDebugSet;
