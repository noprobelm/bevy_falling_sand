use bevy::prelude::*;
use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};

use crate::{Chunk, ChunkMap, ParticleSimulationSet};

pub(super) struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ParticleType>()
            .register_type::<Particle>()
            .register_type::<Coordinates>()
            .init_resource::<ParticleTypeMap>()
            .add_event::<ParticleRegistrationEvent>()
            .add_event::<ResetParticleEvent>()
            .add_event::<RemoveParticleEvent>()
            .add_event::<MutateParticleEvent>()
            .add_systems(
                PreUpdate,
                handle_new_particles.before(ParticleSimulationSet),
            )
            .add_systems(Update, handle_new_particle_types)
            .add_systems(Update, ev_mutate_particle.in_set(ParticleSimulationSet))
            .add_observer(on_reset_particle);
    }
}

pub trait ParticleBlueprint: Component {
    type Data: Component;

    fn data(&self) -> &Self::Data;
    fn data_mut(&mut self) -> &mut Self::Data;
}

#[macro_export]
macro_rules! impl_particle_blueprint {
    ($struct_name:ident, $data_type:ty) => {
        impl ParticleBlueprint for $struct_name {
            type Data = $data_type;

            fn data(&self) -> &Self::Data {
                &self.0
            }

            fn data_mut(&mut self) -> &mut Self::Data {
                &mut self.0
            }
        }
    };
}

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
    pub name: String,
}

impl ParticleType {
    pub fn new(name: &str) -> ParticleType {
        ParticleType {
            name: name.to_string(),
        }
    }
}

#[derive(Resource, Clone, Default, Debug, Reflect)]
#[reflect(Resource)]
pub struct ParticleTypeMap {
    map: HashMap<String, Entity>,
}

impl ParticleTypeMap {
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Entity)> {
        self.map.iter()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.map.keys()
    }

    pub fn insert(&mut self, ptype: String, entity: Entity) -> &mut Entity {
        self.map.entry(ptype).or_insert(entity)
    }

    pub fn get(&self, ptype: &String) -> Option<&Entity> {
        self.map.get(ptype)
    }
}

#[derive(Component, Clone, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Particle {
    pub name: String,
}

impl Particle {
    pub fn new(name: &str) -> Particle {
        Particle {
            name: name.to_string(),
        }
    }
}

#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect, Serialize, Deserialize,
)]
#[reflect(Component)]
pub struct Coordinates(pub IVec2);

#[derive(Clone, Event, Hash, Debug, Eq, PartialEq, PartialOrd)]
pub struct ParticleRegistrationEvent {
    pub entities: Vec<Entity>,
}

#[derive(Event)]
pub struct MutateParticleEvent {
    pub entity: Entity,
    pub particle: Particle,
}

#[derive(Event)]
pub struct RemoveParticleEvent {
    pub coordinates: IVec2,
    pub despawn: bool,
}

#[derive(Event)]
pub struct ResetParticleEvent {
    pub entity: Entity,
}

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

pub fn handle_new_particles(
    mut commands: Commands,
    parent_query: Query<Entity, With<ParticleType>>,
    particle_query: Query<(&Particle, &Transform, Entity), Changed<Particle>>,
    map: Res<ChunkMap>,
    type_map: Res<ParticleTypeMap>,
    mut ev_particle_registered: EventWriter<ParticleRegistrationEvent>,
    mut chunk_query: Query<&mut Chunk>,
) {
    let mut entities: Vec<Entity> = vec![];
    for (particle_type, transform, entity) in particle_query.iter() {
        let coordinates = IVec2::new(
            transform.translation.x as i32,
            transform.translation.y as i32,
        );

        let new = map.insert_no_overwrite(coordinates, entity, &mut chunk_query.as_query_lens());
        if new != entity {
            commands.entity(entity).despawn();
            continue;
        }

        if let Some(parent_entity) = type_map.get(&particle_type.name) {
            if let Ok(parent_entity) = parent_query.get(*parent_entity) {
                entities.push(entity);
                commands.entity(parent_entity).add_child(entity);
                commands.entity(entity).insert((Coordinates(coordinates),));
            }
        } else {
            panic!(
                "No parent entity found for particle type {:?}",
                particle_type
            );
        }
    }
    ev_particle_registered.send(ParticleRegistrationEvent { entities });
}

pub fn ev_mutate_particle(
    mut ev_change_particle: EventReader<MutateParticleEvent>,
    mut particle_query: Query<&mut Particle>,
) {
    for ev in ev_change_particle.read() {
        let mut particle = particle_query.get_mut(ev.entity).unwrap();
        particle.name = ev.particle.name.clone();
    }
}

pub fn on_reset_particle(
    trigger: Trigger<ResetParticleEvent>,
    mut particle_query: Query<&mut Particle>,
) {
    particle_query
        .get_mut(trigger.event().entity)
        .unwrap()
        .into_inner();
}
