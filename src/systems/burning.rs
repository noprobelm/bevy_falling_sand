use bevy::prelude::*;
use bevy_spatial::SpatialAccess;

use crate::*;

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
                        commands.entity(entity).insert(burns.to_burning());
                        if let Some(colors) = &burns.color {
                            commands.entity(entity).insert(colors.clone());
                            commands.entity(entity).insert(RandomizesColor::new(0.75));
                        }
                        if let Some(fire) = &burns.spreads {
                            commands.entity(entity).insert(fire.clone());
                        }
                        if fire.destroys_on_spread {
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
            &mut Burning,
            &mut PhysicsRng,
            &Coordinates,
        ),
    >,
    time: Res<Time>,
) {
    burning_query
        .iter_mut()
        .for_each(|(entity, particle, mut burns, mut burning, mut rng, coordinates)| {
            if burning.timer.tick(time.delta()).finished() {
                if burns.chance_destroy_per_tick.is_some() {
                    commands.trigger(RemoveParticleEvent {
                        coordinates: coordinates.0,
                        despawn: true,
                    })
                } else {
                    commands.entity(entity).remove::<Burning>();
		    // Causes the particle to resync with it's parent's data. This is a temporary solution
		    // until I've written events to handle resetting a particle's specific component data.
		    particle.into_inner();
                }
                return;
            }
            if burning.tick_timer.tick(time.delta()).finished() {
                if let Some(ref mut reaction) = &mut burns.reaction {
                    reaction.produce(&mut commands, &mut rng, coordinates);
                }
                if let Some(chance_destroy) = burns.chance_destroy_per_tick {
                    if rng.chance(chance_destroy) {
                        commands.trigger(RemoveParticleEvent {
                            coordinates: coordinates.0,
                            despawn: true,
                        })
                    }
                }
            }
        });
}
