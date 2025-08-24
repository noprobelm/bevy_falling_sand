//! Provides spatial mapping functionality for particles.
use bevy::platform::collections::{hash_map::Entry, HashMap};
use bevy::platform::hash::FixedHasher;
use bevy::prelude::*;

use crate::{
    Particle, ParticleInstances, ParticlePosition, ParticleSimulationSet, ParticleType,
    ParticleTypeMap,
};

/// Adds Bevy plugin elements for particle mapping functionality.
pub(super) struct ParticleSpatialPlugin {
    pub map_size: usize,
    pub chunk_size: usize,
}

impl Plugin for ParticleSpatialPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ParticleMap::new(self.map_size, self.chunk_size))
            .add_event::<ClearParticleMapEvent>()
            .add_event::<ClearParticleTypeChildrenEvent>()
            .add_event::<DespawnParticleEvent>()
            .add_systems(PostUpdate, reset_chunks.in_set(ParticleSimulationSet))
            .add_systems(
                PreUpdate,
                (
                    ev_clear_particle_type_children,
                    ev_clear_particle_map,
                    ev_despawn_particle,
                ),
            );
    }
}

/// Error for particle map indexing.
#[derive(Debug)]
pub enum SwapError {
    /// The chunk index is out of bounds.
    ChunkOutOfBounds {
        /// The out of bounds index.
        index: usize,
    },
    /// No entity exists at the specified position.
    PositionNotFound {
        /// The invalid position.
        position: IVec2,
    },
    /// The position is out of bounds of the particle map.
    PositionOutOfBounds {
        /// The out of bounds position.
        position: IVec2,
    },
}

/// Maps spatial positions to Particle entities, which can then be cross referenced to a Particle
/// query.
///
/// The map is segmented into a series of chunks, which assists in particle movement systems by
/// allowing for the definition of dirty rects, and eventually parallelized operations on chunks.
#[derive(Clone, Eq, PartialEq, Debug, Resource)]
pub struct ParticleMap {
    /// The x/y size of the particle map.
    pub size: usize,
    /// The x/y number of particles assigned per chunk.
    pub particles_per_chunk: usize,
    /// The chunks, stored as a flat map
    chunks: Vec<Chunk>,
    /// The offset value to use when finding the index of a chunk.
    flat_map_offset_value: i32,
    /// Bitwise right shift operand to use when finding the index of a chunk.
    chunk_shift: u32,
    /// Bitwise right shift for map size operations.
    map_shift: u32,
}

impl ParticleMap {
    /// Initialize a new [`ParticleMap`] using custom values for the map and chunk size.
    ///
    /// # Panics
    ///
    /// The returned [`ParticleMap`] will panic if the `map_size` or `chunk_size` is not a power of
    /// two. The internals of this struct rely on this property for efficient indexing.
    #[must_use]
    pub fn new(map_size: usize, chunk_size: usize) -> Self {
        assert!(
            map_size.is_power_of_two(),
            "Particle map size must be a power of 2"
        );
        assert!(
            chunk_size.is_power_of_two(),
            "Chunk size must be a power of 2"
        );

        let map_shift = map_size.trailing_zeros();
        let chunk_shift = chunk_size.trailing_zeros();
        let num_chunks = map_size.pow(2);
        let grid_offset = num_chunks / 2;
        let mut chunks = Vec::with_capacity(num_chunks);

        let map_size_i32: i32 = map_size.try_into().expect("map_size exceeds i32::MAX");
        let chunk_size_i32: i32 = chunk_size.try_into().expect("chunk_size exceeds i32::MAX");
        let grid_offset_i32: i32 = grid_offset
            .try_into()
            .expect("grid_offset exceeds i32::MAX");

        for i in 0..num_chunks {
            let i_i32: i32 = i.try_into().expect("num_chunks exceeds i32::MAX");
            let row = i_i32 / map_size_i32;
            let col = i_i32 % map_size_i32;

            let x = col * chunk_size_i32 - grid_offset_i32;
            let y = grid_offset_i32 - row * chunk_size_i32;
            let upper_left = IVec2::new(x, y - (chunk_size_i32 - 1));
            let lower_right = IVec2::new(x + (chunk_size_i32 - 1), y);

            let chunk = Chunk::new(upper_left, lower_right, map_size);
            chunks.push(chunk);
        }

        Self {
            chunks,
            size: map_size,
            particles_per_chunk: chunk_size.pow(2),
            flat_map_offset_value: grid_offset
                .try_into()
                .expect("grid_offset exceeds i32::MAX"),
            chunk_shift,
            map_shift,
        }
    }

    #[inline(always)]
    #[allow(clippy::cast_sign_loss)]
    const fn index(&self, position: IVec2) -> Option<usize> {
        let offset = self.flat_map_offset_value;
        let col = ((position.x + offset) >> self.chunk_shift) as isize;
        let row = ((offset - position.y) >> self.chunk_shift) as isize;
        let size = self.size as isize;

        if col < 0 || col >= size || row < 0 || row >= size {
            None
        } else {
            Some(((row as usize) << self.map_shift) + (col as usize)) // row * size using bit shift
        }
    }

    /// Get a chunk if the position falls anywhere within its bounds.
    #[must_use]
    #[inline(always)]
    pub fn chunk(&self, position: &IVec2) -> Option<&Chunk> {
        self.index(*position).and_then(|i| self.chunks.get(i))
    }

    /// Get a mutable chunk if the position falls anywhere within its bounds.
    #[inline(always)]
    pub fn chunk_mut(&mut self, position: &IVec2) -> Option<&mut Chunk> {
        self.index(*position)
            .and_then(move |index| self.chunks.get_mut(index))
    }

    /// Get the entity at position.
    #[must_use]
    #[inline(always)]
    pub fn get(&self, position: &IVec2) -> Option<&Entity> {
        self.chunk(position)?.get(position)
    }

    /// Remove the entity at position.
    pub fn remove(&mut self, position: &IVec2) -> Option<Entity> {
        self.chunk_mut(position)?.remove(position)
    }

    /// # Safety
    /// Caller must ensure that the position lies within the bounds of the particle map.
    #[must_use]
    pub const unsafe fn index_unchecked(&self, position: IVec2) -> usize {
        let col = ((position.x + self.flat_map_offset_value) >> self.chunk_shift) as usize;
        let row = ((self.flat_map_offset_value - position.y) >> self.chunk_shift) as usize;
        (row << self.map_shift) + col // row * size using bit shift
    }

    /// # Safety
    /// Caller must ensure that the position lies within the bounds of the particle map.
    #[must_use]
    pub unsafe fn chunk_unchecked(&self, position: &IVec2) -> &Chunk {
        let index = self.index_unchecked(*position);
        self.chunks.get_unchecked(index)
    }

    /// # Safety
    /// Caller must ensure that the position lies within the bounds of the particle map.
    pub unsafe fn chunk_unchecked_mut(&mut self, position: &IVec2) -> &mut Chunk {
        let index = self.index_unchecked(*position);
        self.chunks.get_unchecked_mut(index)
    }

    /// # Safety
    /// Caller must ensure that the position lies within bounds, and the chunk contains the entity.
    #[must_use]
    pub unsafe fn get_unchecked(&self, position: &IVec2) -> Option<&Entity> {
        self.chunk_unchecked(position).get(position)
    }

    /// # Safety
    /// Caller must ensure that the position lies within bounds, and the chunk contains the entity.
    pub unsafe fn remove_unchecked(&mut self, position: &IVec2) -> Option<Entity> {
        self.chunk_unchecked_mut(position).remove(position)
    }

    /// Iterate through all particle entities in the [`ParticleMap`]
    pub fn iter_particles(&self) -> impl Iterator<Item = &Entity> {
        self.chunks.iter().flat_map(|chunk| chunk.map.values())
    }

    /// Iterate through all chunks in the [`ParticleMap`]
    pub fn iter_chunks(&self) -> impl Iterator<Item = &Chunk> {
        self.chunks.iter()
    }

    /// Iterate through all mutable chunks in the [`ParticleMap`]
    pub fn iter_chunks_mut(&mut self) -> impl Iterator<Item = &mut Chunk> {
        self.chunks.iter_mut()
    }

    /// Swap the entities between the first and second positions.
    ///
    /// # Errors
    ///
    /// Returns `Err(SwapError)` if any position is invalid.
    #[inline(always)]
    pub fn swap(&mut self, first: IVec2, second: IVec2) -> Result<(), SwapError> {
        let first_index = self
            .index(first)
            .ok_or(SwapError::PositionOutOfBounds { position: first })?;
        let second_index = self
            .index(second)
            .ok_or(SwapError::PositionOutOfBounds { position: second })?;

        if first_index == second_index {
            let chunk = unsafe { self.chunks.get_unchecked_mut(first_index) };

            match (chunk.map.remove(&first), chunk.map.remove(&second)) {
                (Some(entity_first), Some(entity_second)) => {
                    chunk.dirty_rect_union_point(first);
                    chunk.dirty_rect_union_point(second);
                    chunk.map.insert(first, entity_second);
                    chunk.map.insert(second, entity_first);
                    Ok(())
                }
                (Some(entity_first), None) => {
                    chunk.dirty_rect_union_point(first);
                    chunk.dirty_rect_union_point(second);
                    chunk.map.insert(second, entity_first);
                    Ok(())
                }
                (None, _) => Err(SwapError::PositionNotFound { position: first }),
            }
        } else {
            let chunks_ptr = self.chunks.as_mut_ptr();
            let chunk_first = unsafe { &mut *chunks_ptr.add(first_index) };
            let chunk_second = unsafe { &mut *chunks_ptr.add(second_index) };

            match (
                chunk_first.map.remove(&first),
                chunk_second.map.remove(&second),
            ) {
                (Some(entity_first), Some(entity_second)) => {
                    chunk_first.dirty_rect_union_point(first);
                    chunk_second.dirty_rect_union_point(second);
                    chunk_first.map.insert(first, entity_second);
                    chunk_second.map.insert(second, entity_first);
                    Ok(())
                }
                (Some(entity_first), None) => {
                    chunk_first.dirty_rect_union_point(first);
                    chunk_second.dirty_rect_union_point(second);
                    chunk_second.map.insert(second, entity_first);
                    Ok(())
                }
                (None, _) => Err(SwapError::PositionNotFound { position: first }),
            }
        }
    }

    #[allow(clippy::cast_sign_loss, clippy::too_many_lines)]
    fn reset_chunks(&mut self) {
        let map_size = self.size;
        let map_size_minus_1 = map_size - 1;
        let chunk_ptr = self.chunks.as_mut_ptr();
        let mut pending_updates = Vec::with_capacity(256);

        // First pass: process dirty rects and collect neighbor updates
        for index in 0..self.chunks.len() {
            let chunk = unsafe { &mut *chunk_ptr.add(index) };

            if let Some(dirty_rect) = chunk.next_dirty_rect.take() {
                let inflated = dirty_rect.inflate(1);
                chunk.dirty_rect = Some(inflated.intersect(chunk.region));
                let expanded = dirty_rect.inflate(2);

                let chunk_row = index >> self.map_shift;
                let chunk_col = index & ((1 << self.map_shift) - 1);

                // Left
                if chunk_col > 0 {
                    let n_index = index - 1;
                    let neighbor_region = unsafe { (*chunk_ptr.add(n_index)).region };
                    let intersection = neighbor_region.intersect(expanded);
                    if !intersection.is_empty() {
                        pending_updates.push((n_index, intersection));
                    }
                }
                // Right
                if chunk_col < map_size_minus_1 {
                    let n_index = index + 1;
                    let neighbor_region = unsafe { (*chunk_ptr.add(n_index)).region };
                    let intersection = neighbor_region.intersect(expanded);
                    if !intersection.is_empty() {
                        pending_updates.push((n_index, intersection));
                    }
                }
                // Up
                if chunk_row > 0 {
                    let n_index = index - map_size;
                    let neighbor_region = unsafe { (*chunk_ptr.add(n_index)).region };
                    let intersection = neighbor_region.intersect(expanded);
                    if !intersection.is_empty() {
                        pending_updates.push((n_index, intersection));
                    }
                }
                // Down
                if chunk_row < map_size_minus_1 {
                    let n_index = index + map_size;
                    let neighbor_region = unsafe { (*chunk_ptr.add(n_index)).region };
                    let intersection = neighbor_region.intersect(expanded);
                    if !intersection.is_empty() {
                        pending_updates.push((n_index, intersection));
                    }
                }
                // Diagonals
                if chunk_row > 0 {
                    // Up-Left
                    if chunk_col > 0 {
                        let n_index = index - map_size - 1;
                        let neighbor_region = unsafe { (*chunk_ptr.add(n_index)).region };
                        let intersection = neighbor_region.intersect(expanded);
                        if !intersection.is_empty() {
                            pending_updates.push((n_index, intersection));
                        }
                    }
                    // Up-Right
                    if chunk_col < map_size_minus_1 {
                        let n_index = index - map_size + 1;
                        let neighbor_region = unsafe { (*chunk_ptr.add(n_index)).region };
                        let intersection = neighbor_region.intersect(expanded);
                        if !intersection.is_empty() {
                            pending_updates.push((n_index, intersection));
                        }
                    }
                }
                if chunk_row < map_size_minus_1 {
                    // Down-Left
                    if chunk_col > 0 {
                        let n_index = index + map_size - 1;
                        let neighbor_region = unsafe { (*chunk_ptr.add(n_index)).region };
                        let intersection = neighbor_region.intersect(expanded);
                        if !intersection.is_empty() {
                            pending_updates.push((n_index, intersection));
                        }
                    }
                    // Down-Right
                    if chunk_col < map_size_minus_1 {
                        let n_index = index + map_size + 1;
                        let neighbor_region = unsafe { (*chunk_ptr.add(n_index)).region };
                        let intersection = neighbor_region.intersect(expanded);
                        if !intersection.is_empty() {
                            pending_updates.push((n_index, intersection));
                        }
                    }
                }
            } else {
                chunk.dirty_rect = None;
            }
        }

        // Second pass: batch apply neighbor updates
        for (n_index, intersection) in pending_updates {
            let neighbor_chunk = unsafe { &mut *chunk_ptr.add(n_index) };
            match &mut neighbor_chunk.dirty_rect {
                Some(existing) => {
                    *existing = existing
                        .union(intersection)
                        .intersect(neighbor_chunk.region);
                }
                None => {
                    neighbor_chunk.dirty_rect = Some(intersection);
                }
            }
        }
    }

    /// Clear the particle map of all entities
    pub fn clear(&mut self) {
        self.chunks.iter_mut().for_each(|chunk| {
            chunk.clear();
            chunk.next_dirty_rect = None;
        });
    }

    /// Find all particles within a circular radius of a center position
    #[inline(always)]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn within_radius(
        &self,
        center: IVec2,
        radius: f32,
    ) -> impl Iterator<Item = (IVec2, &Entity)> {
        let radius_i32 = radius.ceil() as i32;
        let min_pos = center - IVec2::splat(radius_i32);
        let max_pos = center + IVec2::splat(radius_i32);
        let radius_squared = radius * radius;

        self.within_rect_impl(min_pos, max_pos)
            .filter(move |(pos, _)| {
                let diff = *pos - center;
                let dist_sq = (diff.x * diff.x + diff.y * diff.y) as f32;
                dist_sq <= radius_squared
            })
    }

    /// Find all particles within a rectangular area
    #[inline(always)]
    pub fn within_rect(&self, rect: IRect) -> impl Iterator<Item = (IVec2, &Entity)> {
        self.within_rect_impl(rect.min, rect.max)
    }

    #[inline(always)]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn within_rect_impl(
        &self,
        min_pos: IVec2,
        max_pos: IVec2,
    ) -> impl Iterator<Item = (IVec2, &Entity)> {
        let offset = self.flat_map_offset_value;
        let shift = self.chunk_shift;
        let size_minus_1 = self.size as i32 - 1;

        let min_chunk_x = ((min_pos.x + offset) >> shift).max(0) as usize;
        let max_chunk_x = ((max_pos.x + offset) >> shift).min(size_minus_1) as usize;
        let min_chunk_y = ((offset - max_pos.y) >> shift).max(0) as usize;
        let max_chunk_y = ((offset - min_pos.y) >> shift).min(size_minus_1) as usize;

        let chunks = &self.chunks;
        let map_shift = self.map_shift;

        (min_chunk_y..=max_chunk_y)
            .flat_map(move |chunk_row| {
                (min_chunk_x..=max_chunk_x)
                    .map(move |chunk_col| (chunk_row << map_shift) + chunk_col)
            })
            .filter_map(move |chunk_index| chunks.get(chunk_index))
            .flat_map(move |chunk| {
                chunk
                    .iter()
                    .filter(move |(pos, _)| {
                        pos.x >= min_pos.x
                            && pos.x <= max_pos.x
                            && pos.y >= min_pos.y
                            && pos.y <= max_pos.y
                    })
                    .map(|(pos, entity)| (*pos, entity))
            })
    }
}

/// A chunk, used to map positions to entities
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chunk {
    map: HashMap<IVec2, Entity>,
    region: IRect,
    next_dirty_rect: Option<IRect>,
    dirty_rect: Option<IRect>,
}

impl Chunk {
    /// Initialize a new Chunk.
    #[must_use]
    pub fn new(upper_left: IVec2, lower_right: IVec2, size: usize) -> Self {
        Self {
            map: HashMap::with_capacity(size.pow(2)),
            region: IRect::from_corners(upper_left, lower_right),
            next_dirty_rect: None,
            dirty_rect: None,
        }
    }

    #[inline(always)]
    fn dirty_rect_union_point(&mut self, position: IVec2) {
        match &mut self.next_dirty_rect {
            Some(rect) => *rect = rect.union_point(position),
            None => self.next_dirty_rect = Some(IRect::from_center_size(position, IVec2::ONE)),
        }
    }
}

impl Chunk {
    /// Get the region a chunk covers.
    #[must_use]
    pub const fn region(&self) -> IRect {
        self.region
    }

    /// Get the entity at position.
    #[must_use]
    #[inline(always)]
    pub fn get(&self, position: &IVec2) -> Option<&Entity> {
        self.map.get(position)
    }

    /// Insert an entity at position.
    #[inline(always)]
    pub fn insert(&mut self, position: IVec2, item: Entity) -> Option<Entity> {
        self.dirty_rect_union_point(position);
        self.map.insert(position, item)
    }

    /// Get the [`bevy::platform::collections::hash_map::Entry`]
    /// at position.
    pub fn entry(&mut self, position: IVec2) -> Entry<'_, IVec2, Entity, FixedHasher> {
        self.dirty_rect_union_point(position);
        self.map.entry(position)
    }

    /// Remove the entity at position.
    #[inline(always)]
    pub fn remove(&mut self, position: &IVec2) -> Option<Entity> {
        self.dirty_rect_union_point(*position);
        self.map.remove(position)
    }

    fn clear(&mut self) {
        self.map.clear();
        self.next_dirty_rect = None;
    }

    /// Get the dirty rect computed for the current frame.
    #[must_use]
    pub const fn next_dirty_rect(&self) -> Option<IRect> {
        self.next_dirty_rect
    }

    /// Get the dirty rect computed from the previous frame.
    #[must_use]
    pub const fn dirty_rect(&self) -> Option<IRect> {
        self.dirty_rect
    }

    /// Iterate through all positions and entities in the chunk.
    pub fn iter(&self) -> impl Iterator<Item = (&IVec2, &Entity)> {
        self.map.iter()
    }

    /// Iterate through all entities in the chunk.
    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.map.values()
    }

    /// Iterate through all positions in the chunk.
    pub fn positions(&self) -> impl Iterator<Item = &IVec2> {
        self.map.keys()
    }
}

#[derive(Clone, Event)]
/// Event used to trigger the removal of all particles in the [`ParticleMap`] resource.
pub struct ClearParticleMapEvent;

#[derive(Clone, Event)]
/// Event used to trigger the removal of all children under a specified [`ParticleType`].
pub struct ClearParticleTypeChildrenEvent(pub String);

/// Event to send each time a Particle is removed from the simulation.
#[derive(Event)]
pub struct DespawnParticleEvent {
    /// Type of particle remove event
    ev_type: DespawnParticleEventType,
}

impl DespawnParticleEvent {
    /// Build event from particle position.
    #[must_use]
    pub const fn from_position(position: IVec2) -> Self {
        Self {
            ev_type: DespawnParticleEventType::Position(position),
        }
    }

    /// Build event from particle entity.
    #[must_use]
    pub const fn from_entity(entity: Entity) -> Self {
        Self {
            ev_type: DespawnParticleEventType::Entity(entity),
        }
    }
}

enum DespawnParticleEventType {
    Position(IVec2),
    Entity(Entity),
}

fn reset_chunks(mut map: ResMut<ParticleMap>) {
    map.reset_chunks();
}

#[allow(clippy::needless_pass_by_value)]
fn ev_despawn_particle(
    mut ev_remove_particle: EventReader<DespawnParticleEvent>,
    mut commands: Commands,
    mut map: ResMut<ParticleMap>,
    particle_query: Query<&Particle>,
) {
    ev_remove_particle.read().for_each(|ev| match &ev.ev_type {
        DespawnParticleEventType::Position(position) => {
            if let Some(entity) = map.remove(position) {
                commands.entity(entity).despawn();
            } else {
                debug!(
                    "Attempted to despawn particle from position where none exists: {:?}",
                    position
                );
            }
        }
        DespawnParticleEventType::Entity(entity) => {
            if particle_query.contains(*entity) {
                commands.entity(*entity).despawn();
            } else {
                debug!(
                    "Attempted to despawn non-particle entity using DespawnParticlEvent: {:?}",
                    entity
                );
            }
        }
    });
}

#[allow(clippy::needless_pass_by_value)]
fn ev_clear_particle_map(
    mut ev_clear_particle_map: EventReader<ClearParticleMapEvent>,
    mut commands: Commands,
    map: ResMut<ParticleMap>,
) {
    ev_clear_particle_map.read().for_each(|_| {
        map.iter_particles().for_each(|entity| {
            commands.entity(*entity).despawn();
        });
    });
}

#[allow(clippy::needless_pass_by_value)]
fn ev_clear_particle_type_children(
    mut ev_clear_particle_type_children: EventReader<ClearParticleTypeChildrenEvent>,
    mut commands: Commands,
    mut map: ResMut<ParticleMap>,
    particle_query: Query<&ParticlePosition, With<Particle>>,
    particle_type_query: Query<&ParticleInstances, With<ParticleType>>,
    particle_parent_map: Res<ParticleTypeMap>,
) {
    ev_clear_particle_type_children.read().for_each(|ev| {
        let particle_type = &ev.0;
        if let Some(parent_entity) = particle_parent_map.get(particle_type) {
            if let Ok(particle_instances) = particle_type_query.get(*parent_entity) {
                for child_entity in particle_instances.iter() {
                    if let Ok(position) = particle_query.get(child_entity) {
                        map.remove(&position.0);
                    } else {
                        panic!("No child entity found for particle type '{particle_type}' while removing child from particle map!")
                    }
                    commands.entity(child_entity).despawn();
                }
            }
        } else {
            warn!("Ignoring particle type '{particle_type}': not found in particle type map.");
        }
    });
}
