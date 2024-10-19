//! Handle newly added or changed particles within the simulation.

use bevy::prelude::*;
use bevy_turborand::prelude::{DelegatedRng, GlobalRng};
use bfs_color::*;
use bfs_core::*;
use bfs_movement::{material::Material, *};
use bfs_reactions::*;

/// Plugin for handling newly added or changed particles within the simulation.
pub struct SimulationManagementPlugin;

impl Plugin for SimulationManagementPlugin {
    fn build(&self, app: &mut App) {
	app.add_systems(Update, handle_new_particles);
        app.observe(on_remove_particle)
            .observe(on_clear_chunk_map)
            // Particle state change observers.
            .observe(on_solid_added)
            .observe(on_movable_solid_added)
            .observe(on_liquid_added)
            .observe(on_gas_added)
            // Particle component reset observers.
            .observe(on_reset_particle)
            .observe(on_reset_density)
            .observe(on_reset_movement_priority)
            .observe(on_reset_velocity)
            .observe(on_reset_particle_color)
            .observe(on_reset_momentum)
            .observe(on_reset_fire)
            .observe(on_reset_burns)
            .observe(on_reset_burning)
            .observe(on_reset_randomizes_color)
            .observe(on_reset_flows_color);
    }
}

/// Map all particles to their respective parent when added/changed within the simulation
pub fn handle_new_particles(
    mut commands: Commands,
    parent_query: Query<Entity, With<ParticleType>>,
    particle_query: Query<(&Particle, &Transform, Entity), Changed<Particle>>,
    mut map: ResMut<ChunkMap>,
    type_map: Res<ParticleTypeMap>,
) {
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
                commands.entity(parent_entity).add_child(entity);
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
                    PhysicsRng::default(),
                    ColorRng::default(),
                    ReactionRng::default(),
                ));
                commands.trigger(ResetDensityEvent { entity });
                commands.trigger(ResetMovementPriorityEvent { entity });
                commands.trigger(ResetVelocityEvent { entity });
                commands.trigger(ResetParticleColorEvent { entity });
                commands.trigger(ResetRandomizesColorEvent { entity });
                commands.trigger(ResetFlowsColorEvent { entity });
                commands.trigger(ResetMomentumEvent { entity });
                commands.trigger(ResetFireEvent { entity });
                commands.trigger(ResetBurnsEvent { entity });
                commands.trigger(ResetBurningEvent { entity });
            }
        } else {
            panic!(
                "No parent entity found for particle type {:?}",
                particle_type
            );
        }
    }
}

/// Observer for resetting all of a particle's data. This system simply marks the Particle as changed so it gets picked
/// up by `handle_new_particles` the next frame.
pub fn on_reset_particle(
    trigger: Trigger<ResetParticleEvent>,
    mut particle_query: Query<&mut Particle>,
) {
    particle_query
        .get_mut(trigger.event().entity)
        .unwrap()
        .into_inner();
}

/// Observer for resetting a particle's Density information to its parent's.
pub fn on_reset_density(
    trigger: Trigger<ResetDensityEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&Density>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(density) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(density.clone());
        } else {
            commands.entity(trigger.event().entity).remove::<Density>();
        }
    }
}

/// Observer for resetting a particle's MovementPriority information to its parent's.
pub fn on_reset_movement_priority(
    trigger: Trigger<ResetMovementPriorityEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&MovementPriority>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(movement_priority) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(movement_priority.clone());
        } else {
            commands
                .entity(trigger.event().entity)
                .remove::<MovementPriority>();
        }
    }
}

/// Observer for resetting a particle's Velocity information to its parent's.
pub fn on_reset_velocity(
    trigger: Trigger<ResetVelocityEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&Velocity>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(velocity) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(velocity.clone());
        } else {
            commands.entity(trigger.event().entity).remove::<Velocity>();
        }
    }
}

/// Observer for resetting a particle's Velocity information to its parent's.
pub fn on_reset_particle_color(
    trigger: Trigger<ResetParticleColorEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&ParticleColor>, With<ParticleType>>,
    mut rng: ResMut<GlobalRng>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        let rng = rng.get_mut();
        if let Some(particle_color) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(particle_color.new_with_random(rng));
        } else {
            commands
                .entity(trigger.event().entity)
                .remove::<ParticleColor>();
        }
    }
}

/// Observer for resetting a particle's Momentum information to its parent's.
pub fn on_reset_momentum(
    trigger: Trigger<ResetMomentumEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&Momentum>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(momentum) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(momentum.clone());
        } else {
            commands.entity(trigger.event().entity).remove::<Momentum>();
        }
    }
}

/// Observer for resetting a particle's Fire information to its parent's.
pub fn on_reset_fire(
    trigger: Trigger<ResetFireEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&Fire>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(fire) = parent_query.get(parent.get()).unwrap() {
            commands.entity(trigger.event().entity).insert(fire.clone());
        } else {
            commands.entity(trigger.event().entity).remove::<Fire>();
        }
    }
}

/// Observer for resetting a particle's Burns information to its parent's.
pub fn on_reset_burns(
    trigger: Trigger<ResetBurnsEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&Burns>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(burns) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(burns.clone());
        } else {
            commands.entity(trigger.event().entity).remove::<Burns>();
        }
    }
}

/// Observer for resetting a particle's Burning information to its parent's.
pub fn on_reset_burning(
    trigger: Trigger<ResetBurningEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&Burning>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(burning) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(burning.clone());
        } else {
            commands.entity(trigger.event().entity).remove::<Burning>();
        }
    }
}

/// Observer for resetting a particle's RandomizesColor information to its parent's.
pub fn on_reset_randomizes_color(
    trigger: Trigger<ResetRandomizesColorEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&RandomizesColor>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(color) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(color.clone());
        } else {
            commands
                .entity(trigger.event().entity)
                .remove::<RandomizesColor>();
        }
    }
}

/// Observer for resetting a particle's FlowsColor information to its parent's.
pub fn on_reset_flows_color(
    trigger: Trigger<ResetFlowsColorEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&FlowsColor>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(color) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(color.clone());
        } else {
            commands
                .entity(trigger.event().entity)
                .remove::<FlowsColor>();
        }
    }
}

// /// Observer for resetting a particle's FlowsColor information to its parent's.
// pub fn on_reset_reacts(
//     trigger: Trigger<ResetReactsEvent>,
//     mut commands: Commands,
//     particle_query: Query<&Parent, With<Particle>>,
//     parent_query: Query<Option<&Reacts>, With<ParticleType>>,
// ) {
//     if let Ok(parent) = particle_query.get(trigger.event().entity) {
//         if let Some(reacts) = parent_query.get(parent.get()).unwrap() {
//             commands
//                 .entity(trigger.event().entity)
//                 .insert(reacts.clone());
//         } else {
//             commands.entity(trigger.event().entity).remove::<Reacts>();
//         }
//     }
// }

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

/// Observer for clearing all particles from the world as soon as a ClearMapEvent is triggered.
pub fn on_clear_chunk_map(
    _trigger: Trigger<ClearMapEvent>,
    mut commands: Commands,
    particle_parent_map: Res<ParticleTypeMap>,
    mut map: ResMut<ChunkMap>,
) {
    particle_parent_map.iter().for_each(|(_, entity)| {
        commands.entity(*entity).despawn_descendants();
    });

    map.clear();
}

/// Observer for adding movement priority when a particle is given a new state of matter.
pub fn on_solid_added(
    trigger: Trigger<OnAdd, Solid>,
    mut commands: Commands,
    particle_query: Query<&Solid, With<ParticleType>>,
) {
    let entity = trigger.entity();
    if let Ok(solid) = particle_query.get(entity) {
        commands
            .entity(entity)
            .insert(solid.into_movement_priority());
    }
}

/// Observer for adding movement priority when a particle is given a new state of matter.
pub fn on_movable_solid_added(
    trigger: Trigger<OnAdd, MovableSolid>,
    mut commands: Commands,
    particle_query: Query<&MovableSolid, With<ParticleType>>,
) {
    let entity = trigger.entity();
    if let Ok(movable_solid) = particle_query.get(entity) {
        commands
            .entity(entity)
            .insert(movable_solid.into_movement_priority());
    }
}

/// Observer for adding movement priority when a particle is given a new state of matter.
pub fn on_liquid_added(
    trigger: Trigger<OnAdd, Liquid>,
    mut commands: Commands,
    particle_query: Query<&Liquid, With<ParticleType>>,
) {
    let entity = trigger.entity();
    if let Ok(liquid) = particle_query.get(entity) {
        commands
            .entity(entity)
            .insert(liquid.into_movement_priority());
    }
}

/// Observer for adding movement priority when a particle is given a new state of matter.
pub fn on_gas_added(
    trigger: Trigger<OnAdd, Gas>,
    mut commands: Commands,
    particle_query: Query<&Gas, With<ParticleType>>,
) {
    let entity = trigger.entity();
    if let Ok(gas) = particle_query.get(entity) {
        commands.entity(entity).insert(gas.into_movement_priority());
    }
}

/// Observer for adding movement priority when a particle is given a new state of matter.
pub fn on_wall_added(
    trigger: Trigger<OnAdd, Wall>,
    mut commands: Commands,
    particle_query: Query<&Wall, With<ParticleType>>,
) {
    let entity = trigger.entity();
    if let Ok(gas) = particle_query.get(entity) {
        commands.entity(entity).insert(gas.into_movement_priority());
    }
}
