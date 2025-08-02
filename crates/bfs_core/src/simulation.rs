//! Provides simulation control and utilities.
use bevy::prelude::*;
use bevy_turborand::DelegatedRng;
use std::ops::RangeBounds;

/// Adds Bevy plugin elements for simulation control.
pub(super) struct ParticleSimulationPlugin;

impl Plugin for ParticleSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticleSimulationRun>()
            .configure_sets(
                PostUpdate,
                ParticleSimulationSet.run_if(
                    resource_exists::<ParticleSimulationRun>
                        .or(condition_ev_simulation_step_received),
                ),
            )
            .add_event::<SimulationStepEvent>()
            .add_event::<ParticleRegistrationEvent>();
    }
}

/// A trait for RNG utilities used in particle systems.
pub trait ParticleRng: Component {
    /// The type of the internal RNG
    type InnerRng: DelegatedRng;

    /// Get mutable access to the inner RNG.
    fn inner_mut(&mut self) -> &mut Self::InnerRng;

    /// Shuffle the given slice.
    fn shuffle<T>(&mut self, slice: &mut [T]) {
        self.inner_mut().shuffle(slice);
    }

    /// Return true with the given probability.
    fn chance(&mut self, rate: f64) -> bool {
        self.inner_mut().chance(rate)
    }

    /// Sample a random element from a list.
    fn sample<'a, T>(&mut self, list: &'a [T]) -> Option<&'a T> {
        self.inner_mut().sample(list)
    }

    /// Return a random index within the given bounds.
    fn index(&mut self, bound: impl RangeBounds<usize>) -> usize {
        self.inner_mut().index(bound)
    }
}

/// Convenience macro for implementing [`ParticleRng`] on a component.
#[macro_export]
macro_rules! impl_particle_rng {
    ($wrapper:ident, $inner:ty) => {
        impl ParticleRng for $wrapper {
            type InnerRng = $inner;

            fn inner_mut(&mut self) -> &mut Self::InnerRng {
                &mut self.0
            }
        }
    };
}

/// Marker resource to indicate whether the simulation should be running.
#[derive(Resource, Default)]
pub struct ParticleSimulationRun;

/// System set for particle simulation systems.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleSimulationSet;

/// Event which is used to trigger the simulation to step forward by one tick.
#[derive(Clone, Event, Hash, Debug, Eq, PartialEq, PartialOrd)]
pub struct SimulationStepEvent;

/// An event which is sent each time a new [`Particle`] has been spawned into the world. Systems
/// which listen for this event can insert other Particle-type components to the subject entitiesa.
#[derive(Clone, Event, Hash, Debug, Eq, PartialEq, PartialOrd)]
pub struct ParticleRegistrationEvent {
    /// The new particle entities.
    pub entities: Vec<Entity>,
}

#[allow(clippy::needless_pass_by_value)]
fn condition_ev_simulation_step_received(
    mut ev_simulation_step: EventReader<SimulationStepEvent>,
) -> bool {
    // For some reason, ev_simulation_step.is_empty() will not cause the simulation to step
    // forward. We have to actually read the event. I'm probably just doing something wrong but I
    // haven't figured out what it is yet.
    for _ in ev_simulation_step.read() {
        return true;
    }
    false
}
