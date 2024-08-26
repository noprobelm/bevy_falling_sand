use bevy::prelude::*;
use bevy_spatial::SpatialAccess;

use crate::{
    Burning, Burns, Coordinates, Fire, Particle, ParticleTree, PhysicsRng, RemoveParticleEvent,
};

/// Burns particles within a radius of entities that posses the `Fire` component.
pub fn handle_fire(
    mut commands: Commands,
    fire_query: Query<(&Fire, &Coordinates)>,
    burns_query: Query<Entity, (With<Particle>, With<Burns>, Without<Burning>)>,
    particle_tree: Res<ParticleTree>,
) {
    fire_query.iter().for_each(|(fire, coordinates)| {
        particle_tree
            .within_distance(coordinates.0.as_vec2(), fire.burn_radius)
            .iter()
            .for_each(|(_, entity)| {
                if let Ok(entity) = burns_query.get(entity.unwrap()) {
                    commands.entity(entity).insert(Burning);
                }
            });
    });
}

/// Handles all burning particles for the frame.
pub fn handle_burning(
    mut commands: Commands,
    mut burns_query: Query<(Entity, &mut Burns, &mut PhysicsRng, &Coordinates), With<Burning>>,
    time: Res<Time>,
) {
    burns_query
        .iter_mut()
        .for_each(|(entity, mut burns, mut rng, coordinates)| {
            burns.tick(time.delta());
            if burns.timer.finished() {
                if burns.destroy {
		    commands.trigger(RemoveParticleEvent{coordinates: coordinates.0, despawn: true})
                }
                commands.entity(entity).remove::<Burning>();
                burns.reset();
                return;
            }
            burns.tick_timer.tick(time.delta());
            if burns.tick_timer.finished() {
                if let Some(ref mut reaction) = &mut burns.reaction {
                    if reaction.chance(&mut rng) {
                        reaction.produce(&mut commands, coordinates);
                    }
                }
            }
        });
}
