use bevy::prelude::*;

pub struct CommonUtilitiesPlugin;

impl Plugin for CommonUtilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            ParticleSimulationSet.run_if(resource_exists::<SimulationRun>),
        );
        app.add_event::<ParticleRegistrationEvent>();
        app.init_resource::<SimulationRun>();
    }
}

#[derive(Resource, Default)]
pub struct SimulationRun;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleSimulationSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleDebugSet;

#[derive(Clone, Event, Hash, Debug, Eq, PartialEq, PartialOrd)]
pub struct ParticleRegistrationEvent {
    pub entities: Vec<Entity>
}
