use bevy::prelude::*;

use bfs_core::{Chunk, ChunkMap, Particle};
use bfs_movement::Wall;

pub struct FallingSandDebugPlugin;

impl Plugin for FallingSandDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DynamicParticleCount>()
            .init_resource::<TotalParticleCount>()
            .init_gizmo_group::<DebugGizmos>()
            .add_systems(
                Update,
                (
                    color_hibernating_chunks.run_if(resource_exists::<DebugHibernatingChunks>),
                    color_dirty_rects.run_if(resource_exists::<DebugDirtyRects>),
                    (count_dynamic_particles, count_total_particles)
                        .run_if(resource_exists::<DebugParticleCount>),
                ),
            );
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct DebugGizmos;

#[derive(Default, Resource)]
pub struct DebugParticleCount;

#[derive(Default, Resource)]
pub struct DebugHibernatingChunks;

#[derive(Default, Resource)]
pub struct DebugDirtyRects;

#[derive(Default, Resource)]
pub struct DynamicParticleCount(pub u64);

#[derive(Default, Resource)]
pub struct TotalParticleCount(pub u64);

pub fn color_dirty_rects(
    map: Res<ChunkMap>,
    mut chunk_gizmos: Gizmos<DebugGizmos>,
    chunk_query: Query<&Chunk>,
) {
    map.iter_chunks().for_each(|entity| {
        let chunk = chunk_query.get(*entity).unwrap();
        if let Some(dirty_rect) = chunk.prev_dirty_rect() {
            chunk_gizmos.rect_2d(
                dirty_rect.center().as_vec2(),
                dirty_rect.size().as_vec2() + Vec2::splat(1.),
                Color::srgba(1., 1., 1., 1.),
            )
        }
    });
}

pub fn color_hibernating_chunks(
    map: Res<ChunkMap>,
    mut chunk_gizmos: Gizmos<DebugGizmos>,
    chunk_query: Query<&Chunk>,
) {
    map.iter_chunks().for_each(|entity| {
        let chunk = chunk_query.get(*entity).unwrap();
        let rect = Rect::from_corners(chunk.min().as_vec2(), chunk.max().as_vec2());
        if chunk.hibernating() == true {
            chunk_gizmos.rect_2d(
                rect.center(),
                rect.size() + Vec2::splat(1.),
                Color::srgba(0.67, 0.21, 0.24, 1.),
            );
        }
    });

    map.iter_chunks().for_each(|entity| {
        let chunk = chunk_query.get(*entity).unwrap();
        let rect = Rect::from_corners(chunk.min().as_vec2(), chunk.max().as_vec2());
        if chunk.hibernating() == false {
            chunk_gizmos.rect_2d(
                rect.center(),
                rect.size() + Vec2::splat(1.),
                Color::srgba(0.52, 0.80, 0.51, 1.0),
            );
        }
    });
}

pub fn count_dynamic_particles(
    mut dynamic_particle_count: ResMut<DynamicParticleCount>,
    particle_query: Query<&Particle, Without<Wall>>,
) {
    dynamic_particle_count.0 = particle_query.iter().fold(0, |acc, _| acc + 1);
}

pub fn count_total_particles(
    mut total_particle_count: ResMut<TotalParticleCount>,
    particle_query: Query<&Particle>,
) {
    total_particle_count.0 = particle_query.iter().fold(0, |acc, _| acc + 1);
}
