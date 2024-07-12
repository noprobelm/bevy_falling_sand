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
	Option<&Anchored>,
        Entity,
    )>,
    particle_query: Query<
        (&ParticleType, &Transform, Entity),
        (Changed<ParticleType>, Without<ParticleParent>),
    >,
    mut rng: ResMut<GlobalRng>,
    mut map: ResMut<ChunkMap>,
    type_map: Res<ParticleParentMap>,
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
            if let Ok((_parent_type, _parent, velocity, momentum, colors, anchored, parent_entity)) =
                parent_query.get(*parent_entity)
            {
                commands.entity(entity).insert((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgba(0., 0., 0., 0.),
                            ..default()
                        },
                        transform: *transform,
                        ..default()
                    },
                    Particle,
                    Coordinates(coordinates),
                    Velocity::new(velocity.val, velocity.max),
                    ParticleColor(colors.random(rng)),
                    PhysicsRng::default(),
                ));

		if momentum.is_some() {
                    commands.entity(entity).insert(Momentum(IVec2::ZERO));
                } else {
		    commands.entity(entity).remove::<Momentum>();
		}

		if anchored.is_some() {
		    commands.entity(entity).insert(Anchored);
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

/// Setup all particle parent types on startup
pub fn setup_particles(mut commands: Commands, mut type_map: ResMut<ParticleParentMap>) {
    let id = commands
        .spawn((
            ParticleType::Water,
            ParticleParent,
            WaterBundle::default(),
            SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        ))
        .id();

    type_map.insert(ParticleType::Water, id);

    let id = commands
        .spawn((
            ParticleType::Oil,
            ParticleParent,
            OilBundle::default(),
            SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        ))
        .id();

    type_map.insert(ParticleType::Oil, id);

    let id = commands
        .spawn((
            ParticleType::Whiskey,
            ParticleParent,
            WhiskeyBundle::default(),
            SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        ))
        .id();

    type_map.insert(ParticleType::Whiskey, id);

    let id = commands
        .spawn((
            ParticleType::Sand,
            ParticleParent,
            SandBundle::default(),
            SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        ))
        .id();

    type_map.insert(ParticleType::Sand, id);

    let id = commands
        .spawn((
            ParticleType::Wall,
            ParticleParent,
            WallBundle::default(),
            SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        ))
        .id();

    type_map.insert(ParticleType::Wall, id);

    let id = commands
        .spawn((
            ParticleType::DirtWall,
            ParticleParent,
            DirtWallBundle::default(),
            SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        ))
        .id();

    type_map.insert(ParticleType::DirtWall, id);

    let id = commands
        .spawn((
            ParticleType::GrassWall,
            ParticleParent,
            GrassWallBundle::default(),
            SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        ))
        .id();

    type_map.insert(ParticleType::GrassWall, id);

    let id = commands
        .spawn((
            ParticleType::RockWall,
            ParticleParent,
            RockWallBundle::default(),
            SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        ))
        .id();

    type_map.insert(ParticleType::RockWall, id);

    let id = commands
        .spawn((
            ParticleType::DenseRockWall,
            ParticleParent,
            DenseRockWallBundle::default(),
            SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        ))
        .id();

    type_map.insert(ParticleType::DenseRockWall, id);

    let id = commands
        .spawn((
            ParticleType::Steam,
            ParticleParent,
            SteamBundle::default(),
            SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        ))
        .id();

    type_map.insert(ParticleType::Steam, id);
}
