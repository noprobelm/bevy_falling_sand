//! Systems for moving particles.

use crate::*;

/// Plugin for systems that interact with particle types.
pub struct SystemsPlugin;

impl Plugin for SystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_new_particle_types);
    }
}
/// Map all particles to their respective parent when added/changed within the simulation.
pub fn handle_new_particle_types(
    mut commands: Commands,
    particle_type_query: Query<(Entity, &ParticleType), Changed<ParticleType>>,
    mut type_map: ResMut<ParticleTypeMap>,
) {
    particle_type_query
        .iter()
        .for_each(|(entity, particle_type)| {
            commands
                .entity(entity)
                .insert(SpatialBundle::from_transform(Transform::from_xyz(
                    0., 0., 0.,
                )));
            type_map.insert(particle_type.name.clone(), entity);
        });
}

