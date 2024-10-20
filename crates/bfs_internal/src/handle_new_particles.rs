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
