//! Provides rigid body integration with particle movement systems

use avian2d::prelude::{ColliderAabb, RigidBody, Sleeping};
use bevy::prelude::*;

use crate::{ChunkCoord, ChunkDirtyState, ChunkIndex, ChunkRegion, ParticleMovementSystems};

pub(super) struct RigidBodiesPlugin;

impl Plugin for RigidBodiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            expand_dirty_rects_for_active_bodies.before(ParticleMovementSystems),
        );
    }
}

/// Marker component which can be added to rigid body colliders in order to include their boundaries
/// for evaluation in particle movement systems.
#[derive(Component)]
pub struct ParticleCollider;

#[allow(clippy::needless_pass_by_value)]
pub(super) fn expand_dirty_rects_for_active_bodies(
    bodies: Query<&ColliderAabb, (With<RigidBody>, With<ParticleCollider>, Without<Sleeping>)>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<(&ChunkRegion, &mut ChunkDirtyState)>,
) {
    bodies.iter().for_each(|aabb| {
        if !aabb.min.is_finite() || !aabb.max.is_finite() {
            return;
        }

        let body_rect = IRect::new(
            aabb.min.x.floor() as i32,
            aabb.min.y.floor() as i32,
            aabb.max.x.ceil() as i32,
            aabb.max.y.ceil() as i32,
        );

        let min_coord = chunk_index.world_to_chunk_coord(body_rect.min);
        let max_coord = chunk_index.world_to_chunk_coord(body_rect.max);

        for chunk_y in min_coord.y()..=max_coord.y() {
            for chunk_x in min_coord.x()..=max_coord.x() {
                let coord = ChunkCoord::new(chunk_x, chunk_y);
                let Some(chunk_entity) = chunk_index.get(coord) else {
                    return;
                };
                let Ok((region, mut dirty_state)) = chunk_query.get_mut(chunk_entity) else {
                    continue;
                };
                let Some(dirty_rect) = intersect_rects(body_rect, region.region()) else {
                    continue;
                };

                dirty_state.current = Some(
                    dirty_state
                        .current
                        .map_or(dirty_rect, |current| current.union(dirty_rect)),
                );
                dirty_state.current_positions = None;
                dirty_state.mark_dirty_rect(dirty_rect);
            }
        }
    });
}

#[inline]
fn intersect_rects(a: IRect, b: IRect) -> Option<IRect> {
    let min_x = a.min.x.max(b.min.x);
    let min_y = a.min.y.max(b.min.y);
    let max_x = a.max.x.min(b.max.x);
    let max_y = a.max.y.min(b.max.y);

    if min_x <= max_x && min_y <= max_y {
        Some(IRect::new(min_x, min_y, max_x, max_y))
    } else {
        None
    }
}
