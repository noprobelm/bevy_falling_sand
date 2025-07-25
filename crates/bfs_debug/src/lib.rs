#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(
    clippy::nursery,
    clippy::pedantic,
    nonstandard_style,
    rustdoc::broken_intra_doc_links,
    missing_docs
)]
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]
//! Provides debug functionality for the Falling Sand simulation. This includes visualizing
//! chunks, dirty rectangles, and more.
use bevy::prelude::*;

use bfs_core::{Particle, ParticleMap, ParticlePosition, ParticleSimulationSet};
use bfs_movement::ParticleMaterialsParam;

/// Adds the constructs and systems necessary for debugging the Falling Sand simulation.
pub struct FallingSandDebugPlugin;

impl Plugin for FallingSandDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DynamicParticleCount>()
            .init_resource::<WallParticleCount>()
            .init_resource::<TotalParticleCount>()
            .init_resource::<ActiveParticleCount>()
            .init_resource::<ActiveChunkColor>()
            .init_resource::<InactiveChunkColor>()
            .init_resource::<DirtyRectColor>()
            .init_resource::<DebugParticleCount>()
            .init_resource::<DebugParticleMap>()
            .init_resource::<DebugDirtyRects>()
            .init_gizmo_group::<DebugGizmos>()
            .add_systems(
                Update,
                (
                    color_active_chunks
                        .after(ParticleSimulationSet)
                        .run_if(resource_exists::<DebugParticleMap>),
                    color_dirty_rects
                        .after(ParticleSimulationSet)
                        .after(color_active_chunks)
                        .run_if(resource_exists::<DebugDirtyRects>),
                    (
                        count_dynamic_particles,
                        count_wall_particles,
                        count_total_particles,
                        count_active_particles,
                    )
                        .after(ParticleSimulationSet)
                        .run_if(resource_exists::<DebugParticleCount>),
                ),
            );
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct DebugGizmos;

/// Marker resource to indicate we should count the number of particles present.
#[derive(Default, Resource)]
pub struct DebugParticleCount;

#[derive(Default, Resource)]
/// Resource to control whether the [`ParticleMap`] should be visualized.
pub struct DebugParticleMap;

#[derive(Default, Resource)]
/// Resource to control whether dirty rectangles should be visualized.
pub struct DebugDirtyRects;

#[derive(Default, Resource)]
/// Resource to hold the number of dynamic particles in the simulation.
pub struct DynamicParticleCount(pub u64);

/// Resource to hold the number of wall particles in the simulation.
#[derive(Default, Resource)]
pub struct WallParticleCount(pub u64);

#[derive(Default, Resource)]
/// Resource to hold the total number of particles in the simulation.
pub struct TotalParticleCount(pub u64);

#[derive(Default, Resource)]
/// Resource to hold the number of active particles in the simulation.
pub struct ActiveParticleCount(pub u64);

#[derive(Resource)]
/// Resource to hold the color we render active chunks as.
pub struct ActiveChunkColor(pub Color);

impl Default for ActiveChunkColor {
    fn default() -> Self {
        Self(Color::srgba(0.52, 0.80, 0.51, 1.0))
    }
}

#[derive(Resource)]
/// Resource to hold the color we render inactive chunks as.
pub struct InactiveChunkColor(pub Color);

impl Default for InactiveChunkColor {
    fn default() -> Self {
        Self(Color::srgba(0.67, 0.21, 0.24, 1.0))
    }
}

#[derive(Resource)]
/// Resource to hold the color we render dirty rectangles as.
pub struct DirtyRectColor(pub Color);

impl Default for DirtyRectColor {
    fn default() -> Self {
        Self(Color::srgba(1., 1., 1., 1.))
    }
}

#[allow(clippy::needless_pass_by_value)]
fn color_dirty_rects(
    map: Res<ParticleMap>,
    dirty_rect_color: Res<DirtyRectColor>,
    mut chunk_gizmos: Gizmos<DebugGizmos>,
) {
    map.iter_chunks().for_each(|chunk| {
        if let Some(dirty_rect) = chunk.dirty_rect() {
            let rect = dirty_rect.as_rect();
            chunk_gizmos.rect_2d(
                rect.center(),
                rect.size() + Vec2::splat(1.0),
                dirty_rect_color.0,
            );
        }
    });
}

#[allow(clippy::needless_pass_by_value)]
fn color_active_chunks(
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

fn count_dynamic_particles(
    mut dynamic_particle_count: ResMut<DynamicParticleCount>,
    materials: ParticleMaterialsParam,
) {
    let mut num_dynamic = 0;
    num_dynamic += materials.num_solids();
    num_dynamic += materials.num_movable_solids();
    num_dynamic += materials.num_liquids();
    num_dynamic += materials.num_gases();
    num_dynamic += materials.num_other();

    dynamic_particle_count.0 = num_dynamic;
}

fn count_wall_particles(
    mut wall_particle_count: ResMut<WallParticleCount>,
    materials: ParticleMaterialsParam,
) {
    wall_particle_count.0 = materials.num_walls();
}

fn count_total_particles(
    mut total_particle_count: ResMut<TotalParticleCount>,
    particle_query: Query<&Particle>,
) {
    total_particle_count.0 = particle_query.iter().len() as u64;
}

fn count_active_particles(
    mut active_particle_count: ResMut<ActiveParticleCount>,
    particle_query: Query<&Particle, Changed<ParticlePosition>>,
) {
    active_particle_count.0 = particle_query.iter().fold(0, |acc, _| acc + 1);
}
