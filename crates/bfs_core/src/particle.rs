//! Provides all core functionality for particles, including registration, mutation, removal, and
//! even extension through external plugins if desired.
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ParticleMap;

/// Adds Bevy plugin elements for core particle functionality.
pub struct ParticleCorePlugin;

impl Plugin for ParticleCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticleSimulationRun>()
            .configure_sets(
                Update,
                ParticleSimulationSet.run_if(resource_exists::<ParticleSimulationRun>),
            )
            .init_resource::<ParticleTypeMap>()
            .add_event::<ParticleRegistrationEvent>()
            .add_event::<ResetParticleEvent>()
            .add_event::<RemoveParticleEvent>()
            .add_systems(
                PreUpdate,
                handle_new_particles.before(ParticleSimulationSet),
            )
            .add_systems(Update, handle_new_particle_types)
            .add_observer(on_reset_particle);
    }
}

/// Designates a component as a blueprint for some particle data. Types implmeneting this
/// trait can use its underlying methods to access component data for a particle, which can be
/// useful for resetting particle data back to the blueprint its parent holds.
#[doc(hidden)]
pub trait ParticleComponent: Component {
    /// The data held by the blueprint
    type Data: Component;

    /// The immutable component
    fn component(&self) -> &Self::Data;
    /// The mutable component
    fn component_mut(&mut self) -> &mut Self::Data;
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_particle_blueprint {
    ($struct_name:ident, $data_type:ty) => {
        impl ParticleComponent for $struct_name {
            type Data = $data_type;

            fn component(&self) -> &Self::Data {
                &self.0
            }

            fn component_mut(&mut self) -> &mut Self::Data {
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
    Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Component, Serialize, Deserialize,
)]
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
#[derive(Component, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Component, Serialize, Deserialize)]
pub struct ParticlePosition(pub IVec2);

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
/// Triggers a particle to reset itself to its parent's blueprint data.
pub struct ResetParticleEvent {
    pub entity: Entity,
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
pub fn handle_new_particles(
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

        if map
            .chunk_mut(&coordinates)
            .unwrap()
            .entry(coordinates)
            .or_insert(entity)
            != &entity
        {
            commands.entity(entity).despawn();
            continue;
        }

        if let Some(parent_entity) = type_map.get(&particle_type.name) {
            if let Ok(parent_entity) = parent_query.get(*parent_entity) {
                entities.push(entity);
                commands.entity(parent_entity).add_child(entity);
                commands
                    .entity(entity)
                    .insert((ParticlePosition(coordinates),));
            }
        } else {
            panic!(
                "No parent entity found for particle type {:?}",
                particle_type
            );
        }
    }
    ev_particle_registered.write(ParticleRegistrationEvent { entities });
}

/// Observer which listens for [`ResetParticleEvent`] and subsequently triggers the associated
/// [`Particle`] component of an entity (if it exists) as 
/// [`Changed`](https://docs.rs/bevy/latest/bevy/ecs/prelude/struct.Changed.html). Systems which 
/// act on [`Particle`] change detection can then perform their actions accordingly.
///
/// For example, [`handle_new_particles`] utilizes [`Particle`] component change detection to send a
/// `ParticleRegistrationEvent`, which subcrates of *Bevy Falling Sand* rely on to insert or remove
/// Particle components based on their parent's blueprint data.
pub fn on_reset_particle(
    trigger: Trigger<ResetParticleEvent>,
    mut particle_query: Query<&mut Particle>,
) {
    particle_query
        .get_mut(trigger.event().entity)
        .unwrap()
        .into_inner();
}
