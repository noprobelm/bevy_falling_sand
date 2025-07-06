//! Provides all core functionality for particles, including registration, mutation, removal, and
//! even extension through external plugins if desired.
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_turborand::DelegatedRng;
use serde::{Deserialize, Serialize};
use std::ops::RangeBounds;

use crate::ParticleMap;

/// Adds Bevy plugin elements for core particle functionality.
pub(super) struct ParticleCorePlugin;

impl Plugin for ParticleCorePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Particle>()
            .register_type::<ParticleType>()
            .register_type::<ParticlePosition>()
            .init_resource::<ParticleSimulationRun>()
            .configure_sets(
                Update,
                ParticleSimulationSet.run_if(
                    resource_exists::<ParticleSimulationRun>
                        .or(condition_ev_simulation_step_received),
                ),
            )
            .init_resource::<ParticleTypeMap>()
            .add_event::<SimulationStepEvent>()
            .add_event::<ParticleRegistrationEvent>()
            .add_event::<ResetParticleChildrenEvent>()
            .add_event::<ResetParticleEvent>()
            .add_event::<RemoveParticleEvent>()
            .add_systems(
                PreUpdate,
                handle_new_particles.before(ParticleSimulationSet),
            )
            .add_systems(Update, (handle_new_particle_types, ev_reset_particle))
            .add_observer(on_reset_particle_children);
    }
}

/// A trait for RNG utilities used in particle systems.
pub trait ParticleRng: Component {
    /// The type of the internal RNG (must implement the required methods).
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

/// Unique identifer for a particle type. No two particle types with the same name can exist.
#[derive(
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Default,
    Component,
    Reflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component)]
pub struct ParticleType {
    /// The particle type's name.
    pub name: String,
}

impl ParticleType {
    /// Initialize a new `ParticleType`
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

/// Maps each [`ParticleType`] to their corresponding entity in the ECS world.
#[derive(Resource, Clone, Default, Debug)]
pub struct ParticleTypeMap {
    map: HashMap<String, Entity>,
}

impl ParticleTypeMap {
    /// Returns true if the designated key exists in the map.
    pub fn contains(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    /// Iterate over key value pairs in the map.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Entity)> {
        self.map.iter()
    }

    /// Iterate over keys in the map.
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.map.keys()
    }

    /// Insert a new particle type entity.
    pub fn insert(&mut self, name: String, entity: Entity) -> Option<Entity> {
        self.map.insert(name, entity)
    }

    /// Get a particle type from the map if it exists.
    #[must_use]
    pub fn get(&self, name: &String) -> Option<&Entity> {
        self.map.get(name)
    }
}

/// Marker component for a Particle entity.
#[derive(Component, Clone, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Particle {
    /// The name of the particle, which corresponds to its [`ParticleType`] and can be used as an
    /// index in the  [`ParticleTypeMap`] resource.
    pub name: String,
}

impl Particle {
    /// Initialize a new `Particle`
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

/// Holds the position of a particle in the simulation
#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect, Serialize, Deserialize,
)]
#[reflect(Component)]
pub struct ParticlePosition(pub IVec2);

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

/// Event to send each tiem a Particle is removed from the simulation.
#[derive(Event)]
pub struct RemoveParticleEvent {
    /// The position of the particle.
    pub position: IVec2,
    /// Should the underlying entity be despawned
    pub despawn: bool,
}

#[derive(Event)]
/// Triggers a [`ParticleType`] to reset all of its children's data.
pub struct ResetParticleChildrenEvent {
    /// The particle type entity to reset children for.
    pub entity: Entity,
}

#[derive(Event)]
/// Triggers a particle to reset itself to its parent's blueprint data.
pub struct ResetParticleEvent {
    /// The entity to reset particle blueprint data for.
    pub entity: Entity,
}

#[allow(clippy::needless_pass_by_value)]
fn condition_ev_simulation_step_received(
    mut ev_simulation_step: EventReader<SimulationStepEvent>,
) -> bool {
    for _ in ev_simulation_step.read() {
        return true;
    }
    false
}

/// Handles new particle types as they are added to the world. Particle types with existing names
/// will overwrite the previous entry.
pub fn handle_new_particle_types(
    mut commands: Commands,
    particle_type_query: Query<(Entity, &ParticleType), Changed<ParticleType>>,
    mut type_map: ResMut<ParticleTypeMap>,
) {
    particle_type_query
        .iter()
        .for_each(|(entity, particle_type)| {
            commands
                .entity(entity)
                .insert((Transform::default(), Visibility::default()));
            type_map.insert(particle_type.name.clone(), entity);
        });
}

/// Handles new particles as they are added to the world. If a new particle is being added at the same
/// coordinate of an existing entity, the new particle is despawned.
#[allow(clippy::needless_pass_by_value)]
fn handle_new_particles(
    mut commands: Commands,
    parent_query: Query<Entity, With<ParticleType>>,
    particle_query: Query<(&Particle, &Transform, Entity), Changed<Particle>>,
    mut map: ResMut<ParticleMap>,
    type_map: Res<ParticleTypeMap>,
    mut ev_particle_registered: EventWriter<ParticleRegistrationEvent>,
) {
    let mut entities: Vec<Entity> = vec![];

    for (particle_type, transform, entity) in particle_query.iter() {
        let coordinates = IVec2::new(
            transform.translation.x as i32,
            transform.translation.y as i32,
        );

        if let Some(chunk) = map.chunk_mut(&coordinates) {
            if chunk.entry(coordinates).or_insert(entity) != &entity {
                commands.entity(entity).despawn();
                continue;
            }
        } else {
            // If the chunk is out of bounds, also despawn the entity
            commands.entity(entity).despawn();
            continue;
        }

        if let Some(parent_handle) = type_map.get(&particle_type.name) {
            if let Ok(parent_entity) = parent_query.get(*parent_handle) {
                entities.push(entity);
                commands.entity(parent_entity).add_child(entity);
                commands
                    .entity(entity)
                    .insert((ParticlePosition(coordinates),));
            }
        } else {
            warn!(
                "No parent entity found for particle type {:?}",
                particle_type
            );
        }
    }

    ev_particle_registered.write(ParticleRegistrationEvent { entities });
}

/// Observer which listens for [`ResetParticleEvent`] and subsequently triggers the associated
/// [`Particle`] component of an entity (if it exists) for change detection
/// ([`Changed`](https://docs.rs/bevy/latest/bevy/ecs/prelude/struct.Changed.html)). Systems which
/// act on [`Particle`] change detection can then perform corresponding actions, such as resetting
/// blueprint data.
///
/// For example, [`handle_new_particles`] utilizes [`Particle`] component change detection to send a
/// `ParticleRegistrationEvent`, which subcrates of *Bevy Falling Sand* rely on to insert or remove
/// Particle components based on their parent's blueprint data.
#[allow(clippy::needless_pass_by_value)]
fn ev_reset_particle(
    mut ev_reset_particle: EventReader<ResetParticleEvent>,
    mut particle_query: Query<&mut Particle>,
) {
    ev_reset_particle.read().for_each(|ev| {
        if let Ok(particle) = particle_query.get_mut(ev.entity) {
            particle.into_inner();
        } else {
            warn!("No Particle component found for entity {:?}", ev.entity);
        }
    })
}

/// Observer which listens for [`ResetParticleChildrenEvent`] and subsequently triggers a reset for
/// all children of a particle type entity. This is useful for resetting all particles of a
/// specified type to their parent's blueprint data, allowing for batch resets of particle.
#[allow(clippy::needless_pass_by_value)]
fn on_reset_particle_children(
    trigger: Trigger<ResetParticleChildrenEvent>,
    mut commands: Commands,
    particle_type_query: Query<Option<&Children>, With<ParticleType>>,
) {
    if let Ok(children) = particle_type_query.get(trigger.event().entity) {
        if let Some(children) = children {
            children.iter().for_each(|child| {
                commands.trigger(ResetParticleEvent { entity: child });
            });
        }
    }
}
