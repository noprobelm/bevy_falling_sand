use bevy::prelude::*;
use bfs_color::*;
use bfs_core::*;
use bfs_movement::*;
use bfs_reactions::*;

pub struct ParticleManagementPlugin;

impl Plugin for ParticleManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mutate_particle_type).add_systems(
            PreUpdate,
            handle_new_particles.before(ParticleSimulationSet),
        );
    }
}

pub fn handle_new_particles(
    mut commands: Commands,
    parent_query: Query<Entity, With<ParticleType>>,
    particle_query: Query<(&Particle, &Transform, Entity), Changed<Particle>>,
    mut map: ResMut<ChunkMap>,
    type_map: Res<ParticleTypeMap>,
    mut ev_particle_registered: EventWriter<ParticleRegistrationEvent>,
) {
    let mut entities: Vec<Entity> = vec![];
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
                entities.push(entity);
                commands.entity(parent_entity).add_child(entity);
                commands.entity(entity).insert((
                    Sprite {
                        color: Color::srgba(0., 0., 0., 0.),
                        ..default()
                    },
                    Coordinates(coordinates),
                    ColorRng::default(),
                    ReactionRng::default(),
                ));
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
    ev_particle_registered.send(ParticleRegistrationEvent { entities });
}

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
            Changed<Burning>,
        )>,
    >,
) {
    particle_type_query.iter().for_each(|(_, children)| {
        children.iter().for_each(|entity| {
            commands.trigger(ResetParticleEvent { entity: *entity });
        });
    });
}
