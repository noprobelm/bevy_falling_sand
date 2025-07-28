//! Provides spatial mapping functionality for particles.
use bevy::platform::collections::{hash_map::Entry, HashMap};
use bevy::platform::hash::FixedHasher;
use bevy::prelude::*;

use crate::{
    Particle, ParticleInstances, ParticlePosition, ParticleSimulationSet, ParticleType,
    ParticleTypeMap,
};

/// Adds Bevy plugin elements for particle mapping functionality.
pub(super) struct ParticleMapPlugin;

impl Plugin for ParticleMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ClearParticleMapEvent>()
            .add_event::<ClearParticleTypeChildrenEvent>()
            .add_event::<DespawnParticleEvent>()
            .add_systems(Startup, setup_particle_map)
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
/// allowing for the definition of dirty_rects, and eventually parallelized operations on chunks.
///
/// Preliminary testing suggests the ideal map and chunk size are both '32'. Initializing with
/// "default" will ensure optimal values are used.
#[derive(Clone, Eq, PartialEq, Debug, Resource)]
pub struct ParticleMap {
    /// The x/y size of the particle map.
    pub size: usize,
    /// The x/y number of particles assigned per chunk.
    pub particles_per_chunk: usize,
    /// The chunks, stored as a flat map
    chunks: Vec<Chunk>,
    /// The offset value to use when finding the index of a chunk.
    flat_map_offset_value: usize,
    /// Bitwise right shift operand to use when finding the index of a chunk.
    chunk_shift: u32,
}

impl Default for ParticleMap {
    fn default() -> Self {
        const MAP_SIZE: usize = 32;
        const CHUNK_SIZE: usize = 32;
        ParticleMap::new(MAP_SIZE, CHUNK_SIZE)
    }
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
            flat_map_offset_value: grid_offset,
            chunk_shift: chunk_size.trailing_zeros(),
        }
    }

    const fn index(&self, position: IVec2) -> Option<usize> {
        let col = ((position.x + self.flat_map_offset_value as i32) >> self.chunk_shift) as isize;
        let row = ((self.flat_map_offset_value as i32 - position.y) >> self.chunk_shift) as isize;

        if col < 0 || col >= self.size as isize || row < 0 || row >= self.size as isize {
            None
        } else {
            Some((row as usize) * self.size + (col as usize))
        }
    }

    /// Get a chunk if the position falls anywhere within its bounds.
    #[must_use]
    pub fn chunk(&self, position: &IVec2) -> Option<&Chunk> {
        self.index(*position).and_then(|i| self.chunks.get(i))
    }

    /// Get a mutable chunk if the position falls anywhere within its bounds.
    pub fn chunk_mut(&mut self, position: &IVec2) -> Option<&mut Chunk> {
        self.index(*position)
            .and_then(move |index| self.chunks.get_mut(index))
    }

    /// Get the entity at position.
    #[must_use]
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
        let col = ((position.x + self.flat_map_offset_value as i32) >> self.chunk_shift) as usize;
        let row = ((self.flat_map_offset_value as i32 - position.y) >> self.chunk_shift) as usize;
        row * self.size + col
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
    pub fn swap(&mut self, first: IVec2, second: IVec2) -> Result<(), SwapError> {
        let first_index = self
            .index(first)
            .ok_or(SwapError::PositionOutOfBounds { position: first })?;
        let second_index = self
            .index(second)
            .ok_or(SwapError::PositionOutOfBounds { position: second })?;

        if first_index == second_index {
            let chunk = self
                .chunks
                .get_mut(first_index)
                .ok_or(SwapError::ChunkOutOfBounds { index: first_index })?;

            let entity_first = chunk
                .remove(&first)
                .ok_or(SwapError::PositionNotFound { position: first })?;

            if let Some(entity_second) = chunk.remove(&second) {
                chunk.insert(first, entity_second);
            }
            chunk.insert(second, entity_first);

            return Ok(());
        }

        let (chunk_a, chunk_b) = if first_index < second_index {
            let (left, right) = self.chunks.split_at_mut(second_index);
            (left.get_mut(first_index), right.get_mut(0))
        } else {
            let (left, right) = self.chunks.split_at_mut(first_index);
            (right.get_mut(0), left.get_mut(second_index))
        };

        let (Some(chunk_first), Some(chunk_second)) = (chunk_a, chunk_b) else {
            return Err(SwapError::ChunkOutOfBounds {
                index: first_index.max(second_index),
            });
        };

        let entity_first = chunk_first
            .remove(&first)
            .ok_or(SwapError::PositionNotFound { position: first })?;

        if let Some(entity_second) = chunk_second.remove(&second) {
            chunk_first.insert(first, entity_second);
        }
        chunk_second.insert(second, entity_first);

        Ok(())
    }

    fn reset_chunks(&mut self) {
        let map_size = self.size as isize;
        let chunk_ptr = self.chunks.as_mut_ptr();

        let mut pending_updates = Vec::with_capacity(self.chunks.len()); // Preallocate memory for updates

        for index in 0..self.chunks.len() {
            let chunk = unsafe { &mut *chunk_ptr.add(index) };

            if let Some(dirty_rect) = chunk.next_dirty_rect.take() {
                let inflated_rect = dirty_rect.inflate(1);
                chunk.dirty_rect = Some(inflated_rect.intersect(chunk.region));
                let expanded = dirty_rect.inflate(2);

                let chunk_row = index as isize / map_size;
                let chunk_col = index as isize % map_size;

                let neighbors = [
                    (chunk_row, chunk_col - 1),     // Left
                    (chunk_row, chunk_col + 1),     // Right
                    (chunk_row - 1, chunk_col),     // Up
                    (chunk_row + 1, chunk_col),     // Down
                    (chunk_row - 1, chunk_col - 1), // Up-Left
                    (chunk_row - 1, chunk_col + 1), // Up-Right
                    (chunk_row + 1, chunk_col - 1), // Down-Left
                    (chunk_row + 1, chunk_col + 1), // Down-Right
                ];

                for &(n_row, n_col) in &neighbors {
                    if n_row >= 0 && n_row < map_size && n_col >= 0 && n_col < map_size {
                        let n_index = (n_row * map_size + n_col) as usize;
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

        // Second pass: apply neighbor updates
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
                (diff.x * diff.x + diff.y * diff.y) as f32 <= radius_squared
            })
    }

    /// Find all particles within a rectangular area
    pub fn within_rect(&self, rect: IRect) -> impl Iterator<Item = (IVec2, &Entity)> {
        self.within_rect_impl(rect.min, rect.max)
    }

    fn within_rect_impl(
        &self,
        min_pos: IVec2,
        max_pos: IVec2,
    ) -> impl Iterator<Item = (IVec2, &Entity)> {
        let min_chunk_x =
            ((min_pos.x + self.flat_map_offset_value as i32) >> self.chunk_shift).max(0) as usize;
        let max_chunk_x = ((max_pos.x + self.flat_map_offset_value as i32) >> self.chunk_shift)
            .min(self.size as i32 - 1) as usize;
        let min_chunk_y =
            ((self.flat_map_offset_value as i32 - max_pos.y) >> self.chunk_shift).max(0) as usize;
        let max_chunk_y = ((self.flat_map_offset_value as i32 - min_pos.y) >> self.chunk_shift)
            .min(self.size as i32 - 1) as usize;

        (min_chunk_y..=max_chunk_y)
            .flat_map(move |chunk_row| {
                (min_chunk_x..=max_chunk_x).map(move |chunk_col| chunk_row * self.size + chunk_col)
            })
            .filter_map(move |chunk_index| self.chunks.get(chunk_index))
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

    fn dirty_rect_union_point(&mut self, position: IVec2) {
        if let Some(dirty_rect) = self.next_dirty_rect {
            self.next_dirty_rect = Some(dirty_rect.union_point(position));
        } else {
            self.next_dirty_rect = Some(IRect::from_center_size(position, IVec2::ONE));
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
    pub fn get(&self, position: &IVec2) -> Option<&Entity> {
        self.map.get(position)
    }

    /// Insert an entity at position.
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

    /// Iterate through all entities in the chunk.
    pub fn iter(&self) -> impl Iterator<Item = (&IVec2, &Entity)> {
        self.map.iter()
    }
}

#[derive(Clone, Event)]
/// Event used to trigger the removal of all particles in the [`ParticleMap`] resource.
pub struct ClearParticleMapEvent;

#[derive(Clone, Event)]
/// Event used to trigger the removal of all children under a specified [`ParticleType`].
pub struct ClearParticleTypeChildrenEvent(pub String);

/// Event to send each tiem a Particle is removed from the simulation.
#[derive(Event)]
pub struct DespawnParticleEvent {
    /// Type of particle remove event
    ev_type: DespawnParticleEventType,
}

impl DespawnParticleEvent {
    /// Build event from particle position.
    pub fn from_position(position: IVec2) -> Self {
        Self {
            ev_type: DespawnParticleEventType::Position(position),
        }
    }

    /// Build event from particle entity.
    pub fn from_entity(entity: Entity) -> Self {
        Self {
            ev_type: DespawnParticleEventType::Entity(entity),
        }
    }
}

enum DespawnParticleEventType {
    Position(IVec2),
    Entity(Entity),
}

fn setup_particle_map(mut commands: Commands) {
    commands.insert_resource(ParticleMap::default());
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
                info!(
                    "Attempted to despawn particle from position where none exists: {:?}",
                    position
                );
            }
        }
        DespawnParticleEventType::Entity(entity) => {
            if particle_query.contains(*entity) {
                commands.entity(*entity).despawn();
            } else {
                info!(
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
    mut particle_type_query: Query<&mut ParticleInstances, With<ParticleType>>,
    particle_parent_map: Res<ParticleTypeMap>,
) {
    ev_clear_particle_type_children.read().for_each(|ev| {
        let particle_type = &ev.0;
        if let Some(parent_entity) = particle_parent_map.get(particle_type) {
            if let Ok(mut particle_instances) = particle_type_query.get_mut(*parent_entity) {
                for child_entity in particle_instances.iter() {
                    if let Ok(position) = particle_query.get(*child_entity) {
                        map.remove(&position.0);
                    } else {
                        panic!("No child entity found for particle type '{particle_type}' while removing child from particle map!")
                    }
                    commands.entity(*child_entity).despawn();
                }
                particle_instances.clear();
            }
        } else {
            warn!("Ignoring particle type '{particle_type}': not found in particle type map.");
        }
    });
}
