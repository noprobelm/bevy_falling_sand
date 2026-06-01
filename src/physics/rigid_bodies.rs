//! Provides rigid body integration with particle movement systems

use avian2d::prelude::{ColliderAabb, SpatialQuery, SpatialQueryFilter};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;

use crate::{ChunkCoord, ChunkDirtyState, ChunkIndex, ParticleMovementSystems};

pub(super) struct RigidBodiesPlugin;

impl Plugin for RigidBodiesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RigidBodyParticleOccupancy>()
            .add_systems(
                PostUpdate,
                update_rigid_body_particle_occupancy.before(ParticleMovementSystems),
            );
    }
}

/// Marker component which can be added to rigid body colliders in order to include their boundaries
/// for evaluation in particle movement systems.
#[derive(Component)]
pub struct ParticleCollider;

/// Source particle cells for a [`ParticleCollider`].
///
/// When present, rigid body occupancy can be rebuilt from this cell map instead of issuing
/// per-grid-cell point queries into the physics world.
#[derive(Component, Clone)]
pub struct ParticleColliderCells {
    cells: HashSet<IVec2>,
    grid_from_local_translation: Vec2,
}

impl ParticleColliderCells {
    /// Creates source cell metadata for a [`ParticleCollider`].
    ///
    /// `grid_from_local_translation` converts collider-local coordinates back to the original
    /// particle grid coordinate space.
    #[must_use]
    pub fn new<I>(cells: I, grid_from_local_translation: Vec2) -> Self
    where
        I: IntoIterator<Item = IVec2>,
    {
        Self {
            cells: cells.into_iter().collect(),
            grid_from_local_translation,
        }
    }

    #[inline]
    fn contains_local_point(&self, local_point: Vec2) -> bool {
        let grid_point = local_point + self.grid_from_local_translation;
        let cell = grid_point.floor().as_ivec2();
        self.cells.contains(&cell)
    }
}

/// Grid cells currently occupied by rigid bodies marked with [`ParticleCollider`].
#[derive(Resource, Default)]
pub struct RigidBodyParticleOccupancy {
    cells_by_chunk: HashMap<ChunkCoord, Vec<u64>>,
    chunk_size: usize,
    chunk_word_len: usize,
    chunk_shift: u32,
}

impl RigidBodyParticleOccupancy {
    /// Returns whether a particle-grid cell overlaps a [`ParticleCollider`].
    #[inline]
    #[must_use]
    pub fn contains(&self, position: IVec2) -> bool {
        if self.chunk_size == 0 {
            return false;
        }

        let coord = self.world_to_chunk_coord(position);
        let Some(cells) = self.cells_by_chunk.get(&coord) else {
            return false;
        };

        let local = self.local_position(coord, position);
        let idx = local.y as usize * self.chunk_size + local.x as usize;
        test_bit(cells, idx)
    }

    fn clear_chunk(&mut self, coord: ChunkCoord) {
        self.cells_by_chunk.remove(&coord);
    }

    fn insert(&mut self, coord: ChunkCoord, position: IVec2) {
        let local = self.local_position(coord, position);
        let idx = local.y as usize * self.chunk_size + local.x as usize;
        let cells = self
            .cells_by_chunk
            .entry(coord)
            .or_insert_with(|| vec![0; self.chunk_word_len]);
        set_bit(cells, idx);
    }

    #[inline]
    fn contains_in_chunk(&self, coord: ChunkCoord, position: IVec2) -> bool {
        let Some(cells) = self.cells_by_chunk.get(&coord) else {
            return false;
        };

        let local = self.local_position(coord, position);
        let idx = local.y as usize * self.chunk_size + local.x as usize;
        test_bit(cells, idx)
    }

    fn set_chunk_layout(&mut self, chunk_size: usize) {
        if self.chunk_size == chunk_size {
            return;
        }

        self.cells_by_chunk.clear();
        self.chunk_size = chunk_size;
        self.chunk_word_len = (chunk_size * chunk_size).div_ceil(u64::BITS as usize);
        self.chunk_shift = chunk_size.trailing_zeros();
    }

    #[inline]
    const fn world_to_chunk_coord(&self, position: IVec2) -> ChunkCoord {
        ChunkCoord::new(
            position.x >> self.chunk_shift,
            position.y >> self.chunk_shift,
        )
    }

    #[inline]
    const fn local_position(&self, coord: ChunkCoord, position: IVec2) -> IVec2 {
        IVec2::new(
            position.x - (coord.x() << self.chunk_shift),
            position.y - (coord.y() << self.chunk_shift),
        )
    }
}

#[allow(clippy::needless_pass_by_value)]
fn update_rigid_body_particle_occupancy(
    mut occupancy: ResMut<RigidBodyParticleOccupancy>,
    spatial_query: SpatialQuery,
    chunk_index: Res<ChunkIndex>,
    chunk_query: Query<&ChunkDirtyState>,
    bodies: Query<
        (
            Entity,
            &ColliderAabb,
            Option<&ParticleColliderCells>,
            Option<&GlobalTransform>,
        ),
        With<ParticleCollider>,
    >,
) {
    occupancy.set_chunk_layout(chunk_index.chunk_size() as usize);

    let mut rebuild_chunks = HashSet::<ChunkCoord>::default();

    for (coord, chunk_entity) in chunk_index.iter() {
        let Ok(dirty_state) = chunk_query.get(chunk_entity) else {
            continue;
        };

        if !dirty_state.is_dirty() {
            continue;
        }

        rebuild_chunks.insert(coord);
        for neighbor in coord.neighbors() {
            if chunk_index.contains(neighbor) {
                rebuild_chunks.insert(neighbor);
            }
        }
    }

    if rebuild_chunks.is_empty() {
        return;
    }

    for &coord in &rebuild_chunks {
        occupancy.clear_chunk(coord);
    }

    let mut fallback_colliders = HashSet::<Entity>::default();
    let mut has_cached_colliders = false;

    for (entity, _, collider_cells, transform) in &bodies {
        if collider_cells.is_some() && transform.is_some() {
            has_cached_colliders = true;
        } else {
            fallback_colliders.insert(entity);
        }
    }

    if !has_cached_colliders && fallback_colliders.is_empty() {
        return;
    }

    let filter = SpatialQueryFilter::default();

    for (entity, aabb, collider_cells, transform) in &bodies {
        if !aabb.min.is_finite() || !aabb.max.is_finite() {
            continue;
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
                if !rebuild_chunks.contains(&coord) {
                    continue;
                }

                let chunk_region = chunk_index.chunk_coord_to_chunk_region(coord);
                let Some(scan_rect) = intersect_rects(body_rect, chunk_region) else {
                    continue;
                };

                if let (Some(collider_cells), Some(transform)) = (collider_cells, transform) {
                    scan_cached_collider_cells(
                        &mut occupancy,
                        coord,
                        scan_rect,
                        collider_cells,
                        transform,
                    );
                } else if fallback_colliders.contains(&entity) {
                    scan_occupied_cells(
                        &mut occupancy,
                        coord,
                        scan_rect,
                        &spatial_query,
                        &filter,
                        &fallback_colliders,
                    );
                }
            }
        }
    }
}

fn scan_cached_collider_cells(
    occupancy: &mut RigidBodyParticleOccupancy,
    coord: ChunkCoord,
    scan_rect: IRect,
    collider_cells: &ParticleColliderCells,
    transform: &GlobalTransform,
) {
    let inverse_transform = transform.affine().inverse();

    for y in scan_rect.min.y..=scan_rect.max.y {
        for x in scan_rect.min.x..=scan_rect.max.x {
            let position = IVec2::new(x, y);
            if occupancy.contains_in_chunk(coord, position) {
                continue;
            }

            let world_center = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0);
            let local_center = inverse_transform.transform_point3(world_center).truncate();
            if collider_cells.contains_local_point(local_center) {
                occupancy.insert(coord, position);
            }
        }
    }
}

fn scan_occupied_cells(
    occupancy: &mut RigidBodyParticleOccupancy,
    coord: ChunkCoord,
    scan_rect: IRect,
    spatial_query: &SpatialQuery,
    filter: &SpatialQueryFilter,
    colliders: &HashSet<Entity>,
) {
    for y in scan_rect.min.y..=scan_rect.max.y {
        for x in scan_rect.min.x..=scan_rect.max.x {
            let position = IVec2::new(x, y);
            if occupancy.contains_in_chunk(coord, position) {
                continue;
            }

            let center = position.as_vec2() + Vec2::splat(0.5);
            if spatial_query
                .point_intersections(center, filter)
                .iter()
                .any(|entity| colliders.contains(entity))
            {
                occupancy.insert(coord, position);
            }
        }
    }
}

#[inline]
fn test_bit(words: &[u64], idx: usize) -> bool {
    let word_idx = idx / u64::BITS as usize;
    let bit_idx = idx % u64::BITS as usize;
    words
        .get(word_idx)
        .is_some_and(|word| (word & (1 << bit_idx)) != 0)
}

#[inline]
fn set_bit(words: &mut [u64], idx: usize) {
    let word_idx = idx / u64::BITS as usize;
    let bit_idx = idx % u64::BITS as usize;
    words[word_idx] |= 1 << bit_idx;
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
