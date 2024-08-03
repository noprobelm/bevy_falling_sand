use crate::*;

/// Provides gizmos rendering for visualizing dead/alive chunks
pub fn color_chunks(map: Res<ChunkMap>, mut chunk_gizmos: Gizmos<DebugGizmos>) {
    map.iter_chunks().for_each(|chunk| {
        let rect = Rect::from_corners(chunk.min().as_vec2(), chunk.max().as_vec2());
        if chunk.hibernating() == true {
            chunk_gizmos.rect_2d(rect.center(), 0., rect.size() + Vec2::splat(1.), Color::srgba(0.67, 0.21, 0.24, 1.));
        }
    });

    map.iter_chunks().for_each(|chunk| {
        let rect = Rect::from_corners(chunk.min().as_vec2(), chunk.max().as_vec2());
        if chunk.hibernating() == false {
            chunk_gizmos.rect_2d(
                rect.center(),
                0.,
                rect.size() + Vec2::splat(1.),
                Color::srgba(0.52,0.80,0.51, 1.0),
            );
        }
    });
}

pub fn count_dynamic_particles(mut dynamic_particle_count: ResMut<DynamicParticleCount>, particle_query: Query<&Particle, Without<Anchored>>) {
    dynamic_particle_count.0 = particle_query.iter().fold(0, |acc, _| acc + 1);
}

pub fn count_total_particles(mut total_particle_count: ResMut<TotalParticleCount>, particle_query: Query<&Particle>) {
    total_particle_count.0 = particle_query.iter().fold(0, |acc, _| acc + 1);
}
