//! Provides spatial mapping functionality for particles.
use bevy::platform::collections::{hash_map::Entry, HashMap};
use bevy::platform::hash::FixedHasher;
use bevy::prelude::*;

use crate::{
    Particle, ParticleComponent, ParticlePosition, ParticleSimulationSet, ParticleType,
    ParticleTypeMap, RemoveParticleEvent,
};

/// Adds Bevy plugin elements for particle mapping functionality.
pub struct ParticleMapPlugin;

impl Plugin for ParticleMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ClearMapEvent>()
            .add_event::<ClearParticleTypeChildrenEvent>()
            .add_systems(Startup, setup_particle_map)
            .add_systems(Update, reset_chunks.after(ParticleSimulationSet))
            .add_observer(on_remove_particle)
            .add_observer(on_clear_particle_map)
            .add_observer(on_clear_particle_type_children);
    }
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

    fn index(&self, position: &IVec2) -> usize {
        let col = ((position.x + self.flat_map_offset_value as i32) >> self.chunk_shift) as usize;
        let row = ((self.flat_map_offset_value as i32 - position.y) >> self.chunk_shift) as usize;
        row * self.size + col
    }

    /// Gets a chunk if the position falls anywhere within its bounds.
    pub fn chunk(&self, position: &IVec2) -> Option<&Chunk> {
        let index = self.index(position);
        self.chunks.get(index)
    }

    /// Gets a mutable chunk if the position falls anywhere within its bounds.
    pub fn chunk_mut(&mut self, position: &IVec2) -> Option<&mut Chunk> {
        let index = self.index(position);
        self.chunks.get_mut(index)
    }

    /// Iterate through all chunks in the [`ParticleMap`]
    pub fn iter_chunks(&self) -> impl Iterator<Item = &Chunk> {
        self.chunks.iter()
    }

    /// Iterate through all mutable chunks in the [`ParticleMap`]
    pub fn iter_chunks_mut(&mut self) -> impl Iterator<Item = &mut Chunk> {
        self.chunks.iter_mut()
    }

    /// Get the entity at position.
    pub fn get(&self, position: &IVec2) -> Option<&Entity> {
        let index = self.index(position);
        if let Some(chunk) = self.chunks.get(index) {
            chunk.get(position)
        } else {
            None
        }
    }

    /// Remove the entity at position.
    pub fn remove(&mut self, position: &IVec2) -> Option<Entity> {
        let index = self.index(position); // Calculate index first
        if let Some(chunk) = self.chunks.get_mut(index) {
            chunk.remove(position)
        } else {
            None
        }
    }

    /// Swap the entities between the first and second positions.
    pub fn swap(&mut self, first: IVec2, second: IVec2) {
        let first_chunk_idx = self.index(&first);
        let second_chunk_idx = self.index(&second);

        // Short-circuit if both positions are in the same chunk
        if first_chunk_idx == second_chunk_idx {
            if let Some(chunk) = self.chunks.get_mut(first_chunk_idx) {
                let entity_first = chunk.remove(&first).unwrap();
                if let Some(entity_second) = chunk.remove(&second) {
                    chunk.insert(first, entity_second);
                    chunk.insert(second, entity_first);
                } else {
                    chunk.insert(second, entity_first);
                }
            }
        } else {
            let entity_first = self
                .chunks
                .get_mut(first_chunk_idx)
                .and_then(|chunk| chunk.remove(&first))
                .unwrap();
            if let Some(entity_second) = self
                .chunks
                .get_mut(second_chunk_idx)
                .and_then(|chunk| chunk.remove(&second))
            {
                self.chunks
                    .get_mut(first_chunk_idx)
                    .unwrap()
                    .insert(first, entity_second);
                self.chunks
                    .get_mut(second_chunk_idx)
                    .unwrap()
                    .insert(second, entity_first);
            } else {
                self.chunks
                    .get_mut(second_chunk_idx)
                    .unwrap()
                    .insert(second, entity_first);
            }
        }
    }

    fn reset_chunks(&mut self) {
        self.chunks.iter_mut().for_each(|chunk| {
            if let Some(dirty_rect) = chunk.next_dirty_rect {
                chunk.dirty_rect = Some(dirty_rect.inflate(5).intersect(chunk.region));
            } else {
                chunk.dirty_rect = None;
            }
            chunk.next_dirty_rect = None;
        })
    }

    /// Clear the particle map of all entities
    pub fn clear(&mut self) {
        self.chunks.iter_mut().for_each(|chunk| {
            chunk.clear();
            chunk.next_dirty_rect = None;
        })
    }
}

/// A chunk, used to map positions to entities
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chunk {
    chunk: HashMap<IVec2, Entity>,
    region: IRect,
    next_dirty_rect: Option<IRect>,
    dirty_rect: Option<IRect>,
}

impl Chunk {
    /// Initialize a new Chunk.
    pub fn new(upper_left: IVec2, lower_right: IVec2, size: usize) -> Chunk {
        Chunk {
            chunk: HashMap::with_capacity(size.pow(2)),
            region: IRect::from_corners(upper_left, lower_right),
            next_dirty_rect: None,
            dirty_rect: None,
        }
    }

    fn set_dirty_rect(&mut self, position: IVec2) {
        if let Some(dirty_rect) = self.next_dirty_rect {
            self.next_dirty_rect = Some(dirty_rect.union_point(position));
        } else {
            self.next_dirty_rect = Some(IRect::from_center_size(position, IVec2::ONE));
        }
    }
}

impl Chunk {
    /// Get the region a chunk covers.
    pub fn region(&self) -> IRect {
        self.region
    }

    /// Get the entity at position.
    pub fn get(&self, position: &IVec2) -> Option<&Entity> {
        self.chunk.get(position)
    }

    /// Insert an entity at position.
    pub fn insert(&mut self, position: IVec2, item: Entity) -> Option<Entity> {
        self.set_dirty_rect(position);
        self.chunk.insert(position, item)
    }

    /// Get the
    /// ['Entry'](https://docs.rs/bevy/latest/bevy/platform/collections/hash_map/type.Entry.html)
    /// at position.
    pub fn entry(&mut self, position: IVec2) -> Entry<'_, IVec2, Entity, FixedHasher> {
        self.set_dirty_rect(position);
        self.chunk.entry(position)
    }

    /// Remove the entity at position.
    pub fn remove(&mut self, position: &IVec2) -> Option<Entity> {
        self.set_dirty_rect(*position);
        self.chunk.remove(position)
    }

    fn clear(&mut self) {
        self.chunk.clear();
        self.next_dirty_rect = None;
    }

    /// Get the dirty rect computed for the current frame.
    pub fn next_dirty_rect(&self) -> Option<IRect> {
        self.next_dirty_rect
    }

    /// Get the dirty rect computed from the previous frame.
    pub fn dirty_rect(&self) -> Option<IRect> {
        self.dirty_rect
    }

    /// Iterate through all entities in the chunk.
    pub fn iter(&self) -> impl Iterator<Item = (&IVec2, &Entity)> {
        self.chunk.iter()
    }
}

#[derive(Event)]
pub struct ClearMapEvent;

#[derive(Event)]
pub struct ClearParticleTypeChildrenEvent(pub String);

pub fn on_remove_particle(
    trigger: Trigger<RemoveParticleEvent>,
    mut commands: Commands,
    mut map: ResMut<ParticleMap>,
) {
    if let Some(entity) = map.remove(&trigger.event().position) {
        if trigger.event().despawn {
            commands.entity(entity).remove::<ChildOf>().despawn();
        } else {
            commands.entity(entity).remove::<ChildOf>();
        }
    }
}

fn setup_particle_map(mut commands: Commands) {
    commands.insert_resource(ParticleMap::default());
}

fn reset_chunks(mut map: ResMut<ParticleMap>) {
    map.reset_chunks();
}

pub fn on_clear_particle_map(
    _trigger: Trigger<ClearMapEvent>,
    mut commands: Commands,
    mut map: ResMut<ParticleMap>,
    particle_parent_map: Res<ParticleTypeMap>,
) {
    particle_parent_map.iter().for_each(|(_, entity)| {
        commands.entity(*entity).despawn_related::<Children>();
    });
    map.clear();
}

pub fn on_clear_particle_type_children(
    trigger: Trigger<ClearParticleTypeChildrenEvent>,
    mut commands: Commands,
    particle_query: Query<&ParticlePosition, With<Particle>>,
    parent_query: Query<&Children, With<ParticleType>>,
    particle_parent_map: Res<ParticleTypeMap>,
    mut map: ResMut<ParticleMap>,
) {
    let particle_type = trigger.event().0.clone();
    if let Some(parent_entity) = particle_parent_map.get(&particle_type) {
        if let Ok(children) = parent_query.get(*parent_entity) {
            children.iter().for_each(|child_entity| {
                if let Ok(position) = particle_query.get(child_entity) {
                    map.remove(&position.0);
                } else {
                    // If this happens, something is seriously amiss.
                    error!("No child entity found for particle type '{particle_type}' while removing child from chunk map.")
                }
            });
            commands
                .entity(*parent_entity)
                .despawn_related::<Children>();
        }
    } else {
        warn!("Ignoring particle type '{particle_type}': not found in particle type map.");
    }
}
