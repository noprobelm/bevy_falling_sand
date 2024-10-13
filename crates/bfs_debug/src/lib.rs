use bevy::prelude::*;

use bfs_core::{ChunkMap, Particle};
use bfs_movement::Wall;

/// Plugin for Bevy Falling Sand debugging functionality.
pub struct FallingSandDebugPlugin;

impl Plugin for FallingSandDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<DebugGizmos>()
            .init_resource::<DynamicParticleCount>()
            .init_resource::<TotalParticleCount>()
            .add_systems(
                Update,
                (color_chunks, count_dynamic_particles, count_total_particles)
                    .run_if(resource_exists::<DebugParticles>),
            );
    }
}

/// Debug gizmos.
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct DebugGizmos;

/// Indicates whether built-in debugging should be enabled.
#[derive(Default, Resource)]
pub struct DebugParticles;

/// The total number of dynamic particles in the simulation.
#[derive(Default, Resource)]
pub struct DynamicParticleCount(pub u64);

/// The total number of particles in the simulation.
#[derive(Default, Resource)]
pub struct TotalParticleCount(pub u64);

/// Provides gizmos rendering for visualizing dead/alive chunks
pub fn color_chunks(map: Res<ChunkMap>, mut chunk_gizmos: Gizmos<DebugGizmos>) {
    map.iter_chunks().for_each(|chunk| {
        let rect = Rect::from_corners(chunk.min().as_vec2(), chunk.max().as_vec2());
        if chunk.hibernating() == true {
            chunk_gizmos.rect_2d(
                rect.center(),
                0.,
                rect.size() + Vec2::splat(1.),
                Color::srgba(0.67, 0.21, 0.24, 1.),
            );
        }
    });

    map.iter_chunks().for_each(|chunk| {
        let rect = Rect::from_corners(chunk.min().as_vec2(), chunk.max().as_vec2());
        if chunk.hibernating() == false {
            chunk_gizmos.rect_2d(
                rect.center(),
                0.,
                rect.size() + Vec2::splat(1.),
                Color::srgba(0.52, 0.80, 0.51, 1.0),
            );
        }
    });
}

/// Counts the total number of dynamic particles in the simulation.
pub fn count_dynamic_particles(
    mut dynamic_particle_count: ResMut<DynamicParticleCount>,
    particle_query: Query<&Particle, Without<Wall>>,
) {
    dynamic_particle_count.0 = particle_query.iter().fold(0, |acc, _| acc + 1);
}

/// Counts the total number of particles in the simulation.
pub fn count_total_particles(
    mut total_particle_count: ResMut<TotalParticleCount>,
    particle_query: Query<&Particle>,
) {
    total_particle_count.0 = particle_query.iter().fold(0, |acc, _| acc + 1);
}
