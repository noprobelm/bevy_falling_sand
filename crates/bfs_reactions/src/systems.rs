use super::ReactionRng;
use bevy::prelude::*;
use bfs_color::{ChangesColor, ResetParticleColorEvent};
use bfs_core::{Particle, ParticleMap, ParticlePosition, ParticleRng, ParticleSimulationSet};

use crate::{Burning, Burns, Fire};

pub(super) struct SystemsPlugin;

impl Plugin for SystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                handle_burning.in_set(ParticleSimulationSet),
                handle_fire.after(handle_burning),
            )
                .in_set(ParticleSimulationSet),
        );
    }
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
fn handle_fire(
    mut commands: Commands,
    mut fire_query: Query<(&Fire, &ParticlePosition, &mut ReactionRng)>,
    burns_query: Query<(Entity, &Burns), (With<Particle>, Without<Burning>)>,
    map: Res<ParticleMap>,
) {
    fire_query.iter_mut().for_each(|(fire, position, mut rng)| {
        let mut destroy_fire: bool = false;
        if !rng.chance(fire.chance_to_spread) {
            return;
        }
        map.within_radius(position.0, fire.burn_radius)
            .for_each(|(_, entity)| {
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
            });
        if destroy_fire {
            if let Some(entity) = map.get(&position.0) {
                commands.entity(*entity).despawn();
            }
        }
    });
}

#[allow(clippy::needless_pass_by_value)]
fn handle_burning(
    mut commands: Commands,
    mut burning_query: Query<(
        Entity,
        &mut Burns,
        &mut Burning,
        &mut ReactionRng,
        &ParticlePosition,
    )>,
    time: Res<Time>,
    mut ev_reset_particle_color: EventWriter<ResetParticleColorEvent>,
) {
    let mut entities: Vec<Entity> = vec![];
    burning_query
        .iter_mut()
        .for_each(|(entity, mut burns, mut burning, mut rng, position)| {
            if burning.timer.tick(time.delta()).finished() {
                if burns.chance_destroy_per_tick.is_some() {
                    commands.entity(entity).despawn();
                } else {
                    commands.entity(entity).remove::<Burning>();
                    entities.push(entity);
                }
                return;
            }
            if burning.tick_timer.tick(time.delta()).finished() {
                if let Some(ref mut reaction) = &mut burns.reaction {
                    reaction.produce(&mut commands, &mut rng, position);
                }
                if let Some(chance_destroy) = burns.chance_destroy_per_tick {
                    if rng.chance(chance_destroy) {
                        commands.entity(entity).despawn();
                    }
                }
            }
        });
    ev_reset_particle_color.write(ResetParticleColorEvent { entities });
}
