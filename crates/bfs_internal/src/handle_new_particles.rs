//! Handle newly added or changed particles within the simulation.

use bevy::prelude::*;
use bfs_color::*;
use bfs_core::*;
use bfs_movement::*;
use bfs_reactions::*;

/// Plugin for handling newly added or changed particles within the simulation.
pub struct SimulationManagementPlugin;

impl Plugin for SimulationManagementPlugin {
    fn build(&self, app: &mut App) {
	app.add_systems(Update, handle_new_particles.before(ParticleSimulationSet));
        app.observe(on_remove_particle)
            .observe(on_clear_chunk_map);
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
