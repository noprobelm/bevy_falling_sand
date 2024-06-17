use bevy::prelude::*;
use bevy_turborand::prelude::{DelegatedRng, GlobalRng};
use crate::*;

/// Handle all particles that have either been added to the simulation or changed state.
pub fn handle_new_particles(
    mut commands: Commands,
    parent_query: Query<(
        &ParticleType,
        &ParticleParent,
        &Velocity,
        Option<&Momentum>,
        &ParticleColors,
        Entity,
    )>,
    particle_query: Query<
        (&ParticleType, &Transform, Entity),
        (Changed<ParticleType>, Without<ParticleParent>),
    >,
    mut map: ResMut<ParticleMap>,
    type_map: Res<ParentParticleMap>,
    mut rng: ResMut<GlobalRng>
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

        if let Some(parent_entity) = type_map.get(particle_type) {
            if let Ok((_parent_type, _parent, velocity, momentum, colors, parent_entity)) =
                parent_query.get(*parent_entity)
            {
                commands.entity(entity).insert((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgba(0., 0., 0., 0.),
                            ..default()
                        },
                        ..default()
                    },
                    Particle,
                    Coordinates(coordinates),
                    LastMoved::default(),
                    Velocity::new(velocity.val, velocity.max),
                    ParticleColor(colors.random(rng)),
                ));

		if momentum.is_some() {
                    commands.entity(entity).insert(Momentum(IVec2::ZERO));
                } else {
		    commands.entity(entity).remove::<Momentum>();
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
