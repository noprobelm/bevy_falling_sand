//! Particle mapping behaviors.
use crate::*;
use bevy_turborand::prelude::{DelegatedRng, GlobalRng};

/// Observer for disassociating a particle from its parent, despawning it, and removing it from the ChunkMap if a
/// RemoveParticle event is triggered.
pub fn on_remove_particle(
    trigger: Trigger<RemoveParticleEvent>,
    mut commands: Commands,
    mut map: ResMut<ChunkMap>,
) {
    if let Some(entity) = map.remove(&trigger.event().coordinates) {
        if trigger.event().despawn == true {
            commands.entity(entity).remove_parent().despawn();
        } else {
            commands.entity(entity).remove_parent();
        }
    }
}

/// Observer for clearing all particles from the world as soon as a ClearChunkMap event is triggered.
pub fn on_clear_chunk_map(
    _trigger: Trigger<ClearChunkMapEvent>,
    mut commands: Commands,
    particle_parent_map: Res<ParticleTypeMap>,
    mut map: ResMut<ChunkMap>,
) {
    particle_parent_map.iter().for_each(|(_, entity)| {
        commands.entity(*entity).despawn_descendants();
    });

    map.clear();
}

/// Map all particles to their respective parent when added/changed within the simulation
pub fn handle_new_particles(
    mut commands: Commands,
    parent_query: Query<
        (
            Entity,
            Option<&Density>,
            Option<&MovementPriority>,
            Option<&Velocity>,
            &ParticleColors,
            Option<&Momentum>,
            Option<&Anchored>,
            Option<&Fire>,
            Option<&Burns>,
            Option<&Burning>,
        ),
        With<ParticleType>,
    >,
    particle_query: Query<(&Particle, &Transform, Entity), Changed<Particle>>,
    mut rng: ResMut<GlobalRng>,
    mut map: ResMut<ChunkMap>,
    type_map: Res<ParticleTypeMap>,
) {
    let rng = rng.get_mut();
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
            if let Ok((
                parent_entity,
                density,
                movement_priority,
                velocity,
                colors,
                momentum,
                anchored,
                fire,
                burns,
                burning,
            )) = parent_query.get(*parent_entity)
            {
                commands.entity(entity).insert((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::srgba(0., 0., 0., 0.),
                            ..default()
                        },
                        transform: *transform,
                        ..default()
                    },
                    Coordinates(coordinates),
                    ParticleColor(colors.random(rng)),
                    PhysicsRng::default(),
                    ColorRng::default(),
                ));

                if let Some(density) = density {
                    commands.entity(entity).insert(density.clone());
                } else {
                    commands.entity(entity).remove::<Density>();
                }

                if let Some(velocity) = velocity {
                    commands.entity(entity).insert(velocity.clone());
                } else {
                    commands.entity(entity).remove::<Velocity>();
                }

                if let Some(movement_priority) = movement_priority {
                    commands.entity(entity).insert(movement_priority.clone());
                } else {
                    commands.entity(entity).remove::<MovementPriority>();
                }

                if momentum.is_some() {
                    commands.entity(entity).insert(Momentum(IVec2::ZERO));
                } else {
                    commands.entity(entity).remove::<Momentum>();
                }

                if anchored.is_some() {
                    commands.entity(entity).insert(Anchored);
                } else {
                    commands.entity(entity).remove::<Anchored>();
                }

                if let Some(fire) = fire {
                    commands.entity(entity).insert(fire.clone());
                } else {
                    commands.entity(entity).remove::<Fire>();
                }

                if let Some(burns) = burns {
                    commands.entity(entity).insert(burns.clone());
                } else {
                    commands.entity(entity).remove::<Burns>();
                }

                if let Some(_) = burning {
                    commands.entity(entity).insert(Burning);
                }

                commands.entity(parent_entity).add_child(entity);
            }
        } else {
            panic!(
                "No parent entity found for particle type {:?}",
                particle_type
            );
        }
    }
}

/// Map all particles to their respective parent when added/changed within the simulation
pub fn handle_new_particle_types(
    particle_type_query: Query<(Entity, &ParticleType), Changed<ParticleType>>,
    mut type_map: ResMut<ParticleTypeMap>,
) {
    particle_type_query
        .iter()
        .for_each(|(entity, particle_type)| {
            type_map.insert(particle_type.name.clone(), entity);
        });
}
