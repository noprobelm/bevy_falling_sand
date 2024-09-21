use crate::{ChangeParticleEvent, Particle, ParticleTree, Reacts};
use bevy::prelude::*;
use bevy_spatial::SpatialAccess;

/// Manages particle reactions.
pub fn react_particles(
    mut ev_change_particle: EventWriter<ChangeParticleEvent>,
    reacts_query: Query<(Entity, &Transform, &Reacts, &Particle)>,
    particle_query: Query<&Particle>,
    particle_tree: Res<ParticleTree>,
) {
    reacts_query
        .iter()
        .for_each(|(entity, transform, reacts, _) | {
            let coords = Vec2::new(transform.translation.x, transform.translation.y);
            particle_tree
                .within_distance(coords, 2.)
                .iter()
                .for_each(|(_, other_entity)| {
                    if entity == other_entity.unwrap() {
                        return;
                    }
                    if let Ok(other_particle) = particle_query.get(other_entity.unwrap()) {
                        if reacts.other == *other_particle {
                            ev_change_particle.send(ChangeParticleEvent {
                                entity,
                                particle: reacts.into.clone(),
                            });
			    return;
                        }
                    }
                });
        });
}
