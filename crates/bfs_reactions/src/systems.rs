use super::ReactionRng;
use bevy::prelude::*;
use bevy_spatial::SpatialAccess;
use bfs_color::*;
use bfs_core::{Coordinates, Particle, ParticleSimulationSet, RemoveParticleEvent};
use bfs_spatial::ParticleTree;

use crate::{Burning, Burns, Fire};

pub struct SystemsPlugin;

impl Plugin for SystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_burning, handle_fire.after(handle_burning)).in_set(ParticleSimulationSet),
        );
    }
}

pub struct BurningPlugin;

impl Plugin for BurningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_fire, handle_burning).in_set(ParticleSimulationSet),
        );
    }
}

pub fn handle_fire(
    mut commands: Commands,
    mut fire_query: Query<(&Fire, &Coordinates, &mut ReactionRng)>,
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
                    if let Some(entity) = entity {
                        if let Ok((entity, burns)) = burns_query.get(*entity) {
                            commands.entity(entity).insert(burns.to_burning());
                            if let Some(colors) = &burns.color {
                                commands.entity(entity).insert(colors.clone());
                                commands.entity(entity).insert(ChangesColor::new(0.75));
                            }
                            if let Some(fire) = &burns.spreads {
                                commands.entity(entity).insert(*fire);
                            }
                            if fire.destroys_on_spread {
                                destroy_fire = true;
                            }
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

pub fn handle_burning(
    mut commands: Commands,
    mut burning_query: Query<(
        Entity,
        &mut Particle,
        &mut Burns,
        &mut Burning,
        &mut ReactionRng,
        &Coordinates,
    )>,
    time: Res<Time>,
    mut ev_reset_particle_color: EventWriter<ResetParticleColorEvent>,
) {
    let mut entities: Vec<Entity> = vec![];
    burning_query.iter_mut().for_each(
        |(entity, particle, mut burns, mut burning, mut rng, coordinates)| {
            if burning.timer.tick(time.delta()).finished() {
                if burns.chance_destroy_per_tick.is_some() {
                    commands.trigger(RemoveParticleEvent {
                        coordinates: coordinates.0,
                        despawn: true,
                    })
                } else {
                    commands.entity(entity).remove::<Burning>();
                    entities.push(entity);
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
        },
    );
    ev_reset_particle_color.write(ResetParticleColorEvent { entities });
}
