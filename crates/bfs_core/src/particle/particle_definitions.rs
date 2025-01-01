use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{ChunkMap, ParticleRegistrationEvent, ParticleSimulationSet, ParticleType, ParticleTypeMap};

pub struct ParticleDefinitionsPlugin;

impl Plugin for ParticleDefinitionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, handle_new_particles.before(ParticleSimulationSet));
        app.add_event::<MutateParticleEvent>()
            .register_type::<Coordinates>()
            .register_type::<Particle>()
            .add_event::<ResetParticleEvent>()
            .add_event::<RemoveParticleEvent>()
            .add_observer(on_reset_particle);
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

pub fn on_reset_particle(
    trigger: Trigger<ResetParticleEvent>,
    mut particle_query: Query<&mut Particle>,
) {
    particle_query
        .get_mut(trigger.event().entity)
        .unwrap()
        .into_inner();
}

pub fn handle_new_particles(
    mut commands: Commands,
    parent_query: Query<Entity, With<ParticleType>>,
    particle_query: Query<(&Particle, &Transform, Entity), Changed<Particle>>,
    mut map: ResMut<ChunkMap>,
    type_map: Res<ParticleTypeMap>,
    mut ev_particle_registered: EventWriter<ParticleRegistrationEvent>,
) {
    let mut entities: Vec<Entity> = vec![];
    for (particle_type, transform, entity) in particle_query.iter() {
        let coordinates = IVec2::new(
            transform.translation.x as i32,
            transform.translation.y as i32,
        );

        let new = map.insert_no_overwrite(coordinates, entity);
        if *new != entity {
            commands.entity(entity).despawn();
            continue;
        }

        if let Some(parent_entity) = type_map.get(&particle_type.name) {
            if let Ok(parent_entity) = parent_query.get(*parent_entity) {
                entities.push(entity);
                commands.entity(parent_entity).add_child(entity);
                commands.entity(entity).insert((
                    Coordinates(coordinates),
                ));
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
