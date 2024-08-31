use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_spatial::SpatialAccess;

use crate::{
    Burning, Burns, Coordinates, Fire, Particle, ParticleTree, PhysicsRng, RandomColors,
    RemoveParticleEvent,
};

/// Burns particles within a radius of entities that posses the `Fire` component.
pub fn handle_fire(
    mut commands: Commands,
    mut fire_query: Query<(&Fire, &Coordinates, &mut PhysicsRng)>,
    burns_query: Query<(Entity, &Burns), (With<Particle>, Without<Burning>)>,
    particle_tree: Res<ParticleTree>,
) {
    fire_query
        .iter_mut()
        .for_each(|(fire, coordinates, mut rng)| {
            let mut destroy_fire: bool = false;
            if !rng.chance(fire.chance_to_spread) {
                return;
            }
            particle_tree
                .within_distance(coordinates.0.as_vec2(), fire.burn_radius)
                .iter()
                .for_each(|(_, entity)| {
                    if let Ok((entity, burns)) = burns_query.get(entity.unwrap()) {
                        commands.entity(entity).insert(Burning);
                        if let Some(colors) = &burns.colors {
                            commands.entity(entity).insert(colors.clone());
                        }
                        if let Some(fire) = &burns.spreads {
                            commands.entity(entity).insert(fire.clone());
                        }
                        if fire.destroys_on_ignition {
                            destroy_fire = true;
                        }
                    }
                });
            if destroy_fire {
                commands.trigger(RemoveParticleEvent {
                    coordinates: coordinates.0,
                    despawn: true,
                });
            }
        });
}

/// Handles all burning particles for the frame.
pub fn handle_burning(
    mut commands: Commands,
    mut burning_query: Query<
        (
            Entity,
            &mut Particle,
            &mut Burns,
            &mut PhysicsRng,
            &Coordinates,
        ),
        (With<Burning>, With<Particle>),
    >,
    time: Res<Time>,
) {
    burning_query
        .iter_mut()
        .for_each(|(entity, mut particle, mut burns, mut rng, coordinates)| {
            if burns.timer.tick(time.delta()).finished() {
                if let Some(produces) = &burns.produces_on_completion {
                    particle.name = produces.name.clone();
                }
                if burns.destroy {
                    commands.trigger(RemoveParticleEvent {
                        coordinates: coordinates.0,
                        despawn: true,
                    })
                } else {
                    commands.entity(entity).remove::<Burning>();
                    commands.entity(entity).remove::<RandomColors>();
                    burns.reset();
                }
                return;
            }
            if burns.tick_timer.tick(time.delta()).finished() {
                if let Some(ref mut reaction) = &mut burns.reaction {
                    if reaction.chance(&mut rng) {
                        reaction.produce(&mut commands, coordinates);
                    }
                }
                if rng.chance(burns.chance_destroy_per_tick) {
                    commands.trigger(RemoveParticleEvent {
                        coordinates: coordinates.0,
                        despawn: burns.destroy,
                    })
                }
            }
        });
}
