use bevy::prelude::*;

use bfs_core::{Particle, ParticleMap};
use bfs_movement::Wall;

pub struct FallingSandDebugPlugin;

impl Plugin for FallingSandDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DynamicParticleCount>()
            .init_resource::<TotalParticleCount>()
            .init_resource::<ActiveChunkColor>()
            .init_resource::<InactiveChunkColor>()
            .init_resource::<DirtyRectColor>()
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

#[derive(Resource)]
pub struct ActiveChunkColor(pub Color);

impl Default for ActiveChunkColor {
    fn default() -> Self {
        ActiveChunkColor(Color::srgba(0.52, 0.80, 0.51, 1.0))
    }
}

#[derive(Resource)]
pub struct InactiveChunkColor(pub Color);

impl Default for InactiveChunkColor {
    fn default() -> Self {
        InactiveChunkColor(Color::srgba(0.67, 0.21, 0.24, 1.0))
    }
}

#[derive(Resource)]
pub struct DirtyRectColor(pub Color);

impl Default for DirtyRectColor {
    fn default() -> Self {
        DirtyRectColor(Color::srgba(1., 1., 1., 1.))
    }
}

pub fn color_dirty_rects(
    map: Res<ParticleMap>,
    dirty_rect_color: Res<DirtyRectColor>,
    mut chunk_gizmos: Gizmos<DebugGizmos>,
) {
    map.iter_chunks().for_each(|chunk| {
        if let Some(dirty_rect) = chunk.dirty_rect() {
            chunk_gizmos.rect_2d(
                dirty_rect.center().as_vec2(),
                dirty_rect.size().as_vec2() + Vec2::splat(1.),
                dirty_rect_color.0,
            )
        }
    });
}

pub fn color_hibernating_chunks(
    map: Res<ParticleMap>,
    active_chunk_color: Res<ActiveChunkColor>,
    inactive_chunk_color: Res<InactiveChunkColor>,
    mut chunk_gizmos: Gizmos<DebugGizmos>,
) {
    map.iter_chunks().for_each(|chunk| {
        let rect = Rect::from_corners(chunk.region().min.as_vec2(), chunk.region().max.as_vec2());
        if chunk.dirty_rect().is_none() {
            chunk_gizmos.rect_2d(
                rect.center(),
                rect.size() + Vec2::splat(1.),
                inactive_chunk_color.0,
            );
        }
    });

    map.iter_chunks().for_each(|chunk| {
        let rect = Rect::from_corners(chunk.region().min.as_vec2(), chunk.region().max.as_vec2());
        if chunk.dirty_rect().is_some() {
            chunk_gizmos.rect_2d(
                rect.center(),
                rect.size() + Vec2::splat(1.),
                active_chunk_color.0,
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
