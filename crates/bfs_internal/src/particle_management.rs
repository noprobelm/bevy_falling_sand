//! Handle newly added or changed particles within the simulation.
//!
//! Some dependencies of this crate (e.g., bfs_movement; bfs_color) have modules titled
//! `particle_definitions`. These modules contain comopnents that act as extensions to
//! particle types, as well as events and observers which allow systems to trigger resets
//! of a particle's component data to its parent's blueprint.
//!
//! This module makes heavy use of this pattern. By triggering the aforementioned events
//! each time a particle is added or changed, we can essentially "reset" a particle to its
//! parent's blueprint at will.

use bevy::prelude::*;
use bfs_color::*;
use bfs_core::*;
use bfs_movement::*;
use bfs_reactions::*;

/// Plugin for handling newly added or changed particles within the simulation.
pub struct ParticleManagementPlugin;

impl Plugin for ParticleManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mutate_particle_type).add_systems(
            PreUpdate,
            handle_new_particles.before(ParticleSimulationSet),
        );
    }
}

/// Map all particles to their location and parent when added/changed within the simulation
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

/// Manage mutated particle types.
pub fn mutate_particle_type(
    mut commands: Commands,
    particle_type_query: Query<
        (&ParticleType, &Children),
        Or<(
            Changed<Density>,
            Changed<ParticleColor>,
            Changed<MovementPriority>,
            Changed<RandomizesColor>,
            Changed<FlowsColor>,
	    Changed<Momentum>,
	    Changed<Fire>,
	    Changed<Burns>,
	    Changed<Burning>
        )>,
    >,
) {
    particle_type_query.iter().for_each(|(_, children)| {
        children.iter().for_each(|entity| {
            commands.trigger(ResetParticleEvent { entity: *entity });
        });
    });
}
