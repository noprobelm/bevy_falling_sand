use crate::*;

pub struct ParticleTypeSystemsPlugin;

impl Plugin for ParticleTypeSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_new_particle_types);
    }
}
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
                .insert((Transform::default(), Visibility::default()));
            type_map.insert(particle_type.name.clone(), entity);
        });
}
