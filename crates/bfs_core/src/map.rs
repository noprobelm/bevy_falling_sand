//! Resources providing mapping functionality to particle positions and types.
use ahash::{HashMap, HashMapExt};
use bevy::prelude::*;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{MutateParticleEvent, Particle};

/// Plugin for mapping particles to coordinate space.
pub struct ChunkMapPlugin;

impl Plugin for ChunkMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (reset_chunks, on_change_particle))
            .add_event::<ClearMapEvent>()
            .init_resource::<ChunkMap>()
            .register_type::<Hibernating>();
    }
}

/// Triggers [on_clear_chunk_map](crate::on_clear_chunk_map) to remove a particle from the simulation.
#[derive(Event)]
pub struct ClearMapEvent;

/// Provides a flag for indicating whether an entity is in a hibernating state. Entities with the Hibernating component
/// can be used with bevy query filters to manage which particles are actually being simulated.
/// Marker component for entities that act as a central reference for particle type information.
#[derive(
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Default,
    Component,
    Reflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component)]
pub struct Hibernating;

/// Chunk map for segmenting collections of entities into coordinate-based chunks.
#[derive(Resource, Debug, Clone)]
pub struct ChunkMap {
    /// The entity chunk maps
    chunks: Vec<Chunk>,
}

impl Default for ChunkMap {
    /// Gets a default ChunkMap
    fn default() -> ChunkMap {
        let chunks: Vec<Chunk> = (0..32_i32.pow(2))
            .map(|i| {
                let x = (i % 32) * 32 - 512;
                let y = 512 - (i / 32) * 32;
                let upper_left = IVec2::new(x, y - 31);
                let lower_right = IVec2::new(x + 31, y);
                Chunk::new(upper_left, lower_right)
            })
            .collect();

        ChunkMap { chunks }
    }
}

impl ChunkMap {
    /// Gets the index of the corresponding chunk
    fn chunk_index(&self, coord: &IVec2) -> usize {
        const OFFSET: i32 = 512;
        const GRID_WIDTH: usize = 32;

        let col = ((coord.x + OFFSET) >> 5) as usize;
        let row = ((OFFSET - coord.y) >> 5) as usize;

        row * GRID_WIDTH + col
    }

    /// Gets an immutable reference to a chunk
    fn chunk(&self, coord: &IVec2) -> Option<&Chunk> {
        let index = self.chunk_index(coord);
        self.chunks.get(index)
    }

    /// Gets a mutable reference to a chunk
    fn chunk_mut(&mut self, coord: &IVec2) -> Option<&mut Chunk> {
        let index = self.chunk_index(coord);
        self.chunks.get_mut(index)
    }
}

impl ChunkMap {
    /// Clear all existing key, value pairs from all chunks.
    /// > **⚠️ Warning:**
    /// > Calling this method will cause major breakage to the simulation if entities are not despawned before another
    /// system attempts to access them.
    pub fn clear(&mut self) {
        for map in &mut self.chunks {
            map.clear();
        }
    }

    /// Remove a particle from the map.
    /// > **⚠️ Warning:**
    /// > Calling this method will cause major breakage to the simulation if the target entity is not despawned before
    /// another system attempts to access it.
    pub fn remove(&mut self, coords: &IVec2) -> Option<Entity> {
        self.chunk_mut(&coords).unwrap().remove(coords)
    }
}

impl ChunkMap {
    /// Immutable iterator over all chunks.
    pub fn iter_chunks(&self) -> impl Iterator<Item = &Chunk> {
        self.chunks.iter()
    }
}

impl ChunkMap {
    /// Checks each chunk for activity in the current frame. This method is meant to be called after all
    /// movement logic has occurred for this frame.
    ///
    /// If a chunk was active and is currently hibernating, wake it up and remove the Hibernating marker
    /// component from its entity.
    ///
    /// If a chunk was not activated and is currently awake, put it to sleep and add the Hibernating
    /// component to its entity.
    pub fn reset_chunks(&mut self, mut commands: Commands) {
        for chunk in &mut self.chunks {
            match (chunk.should_process_next_frame, chunk.hibernating) {
                (true, true) => {
                    // Wake up the chunk
                    for entity in chunk.entities() {
                        commands.entity(*entity).remove::<Hibernating>();
                    }
                    chunk.hibernating = false;
                }
                (false, false) => {
                    // Put the chunk to sleep
                    for entity in chunk.entities() {
                        commands.entity(*entity).insert(Hibernating);
                    }
                    chunk.hibernating = true;
                }
                _ => {} // No state change needed
            }
            // Reset processing flag for the next frame
            chunk.should_process_next_frame = false;
        }
    }

    /// Checks if a coordinate lies on the border of a neighboring chunk and activates it if true.
    fn activate_neighbor_chunks(&mut self, coord: &IVec2, chunk_idx: usize) {
        let chunk = &self.chunks[chunk_idx];
        let neighbors = [
            (coord.x == chunk.min().x, chunk_idx - 1),  // Left neighbor
            (coord.x == chunk.max().x, chunk_idx + 1),  // Right neighbor
            (coord.y == chunk.min().y, chunk_idx + 32), // Bottom neighbor
            (coord.y == chunk.max().y, chunk_idx - 32), // Top neighbor
        ];

        for (condition, neighbor_idx) in neighbors.iter() {
            if *condition {
                self.chunks[*neighbor_idx].should_process_next_frame = true;
            }
        }
    }
}

impl ChunkMap {
    /// Inserts a new particle at a given coordinate if it is not already occupied. Calls to this method will
    /// wake up the subject chunk.
    pub fn insert_no_overwrite(&mut self, coords: IVec2, entity: Entity) -> &mut Entity {
        let chunk = self.chunk_mut(&coords).unwrap();
        chunk.insert_no_overwrite(coords, entity)
    }

    /// Inserts a new particle at a given coordinate irrespective of its occupied state. Calls to this method will
    /// wake up the subject chunk.
    pub fn insert_overwrite(&mut self, coords: IVec2, entity: Entity) -> Option<Entity> {
        let chunk = self.chunk_mut(&coords).unwrap();
        chunk.insert_overwrite(coords, entity)
    }

    /// Swaps two entities in the ChunkMap. This method is the preferred interface when carrying out component-based
    /// interactions between entities due to the facilities this provides for waking up neighboring chunks.
    /// 'insert_overwrite' and 'insert_no_overwrite' will wake up the subject chunk, but they will NOT wake up
    /// neighboring chunks.
    pub fn swap(&mut self, first: IVec2, second: IVec2) {
        let first_chunk_idx = self.chunk_index(&first);
        let second_chunk_idx = self.chunk_index(&second);

        // Short-circuit if both positions are in the same chunk
        if first_chunk_idx == second_chunk_idx {
            let chunk = &mut self.chunks[first_chunk_idx];

            let entity_first = chunk.remove(&first).unwrap();
            if let Some(entity_second) = chunk.remove(&second) {
                chunk.insert_overwrite(first, entity_second);
                chunk.insert_overwrite(second, entity_first);
            } else {
                chunk.insert_overwrite(second, entity_first);
            }
        } else {
            // Handle when the positions are in different chunks
            let entity_first = self.chunks[first_chunk_idx].remove(&first).unwrap();
            if let Some(entity_second) = self.chunks[second_chunk_idx].remove(&second) {
                self.chunks[first_chunk_idx].insert_overwrite(first, entity_second);
                self.chunks[second_chunk_idx].insert_overwrite(second, entity_first);
            } else {
                self.chunks[second_chunk_idx].insert_overwrite(second, entity_first);
            }
        }

        self.activate_neighbor_chunks(&first, first_chunk_idx);
        self.activate_neighbor_chunks(&second, second_chunk_idx);
    }

    /// Get an immutable reference to an entity, if it exists.
    pub fn entity(&self, coords: &IVec2) -> Option<&Entity> {
        self.chunk(&coords).unwrap().get(coords)
    }

    /// Iterator through a flattened map of all the particles in the ChunkMap
    #[allow(unused)]
    pub fn iter(&self) -> impl Iterator<Item = (&IVec2, &Entity)> {
        self.chunks.iter().flat_map(|chunk| chunk.iter())
    }

    /// Parallel iterator through a flattened map of all the particles in the ChunkMap
    pub fn par_iter(&self) -> impl IntoParallelIterator<Item = (&IVec2, &Entity)> {
        self.chunks.par_iter().flat_map(|chunk| chunk.par_iter())
    }
}

/// A chunk which stores location information for entities.
#[derive(Debug, Clone)]
pub struct Chunk {
    /// The chunk containing the particle data
    chunk: HashMap<IVec2, Entity>,
    /// The area of the chunk
    irect: IRect,
    /// Flag indicating whether the chunk should be processed in the next frame
    should_process_next_frame: bool,
    /// Flag indicating whether the chunk should be processed this frame
    hibernating: bool,
}

impl Chunk {
    /// Creates a new Chunk instance
    pub fn new(upper_left: IVec2, lower_right: IVec2) -> Chunk {
        Chunk {
            chunk: HashMap::with_capacity(1024),
            irect: IRect::from_corners(upper_left, lower_right),
            should_process_next_frame: false,
            hibernating: false,
        }
    }
}

impl Chunk {
    /// The minimum (upper left) point of the chunk's area
    pub fn min(&self) -> &IVec2 {
        &self.irect.min
    }

    /// The maximum (lower right) point of the chunk's area
    pub fn max(&self) -> &IVec2 {
        &self.irect.max
    }
}

impl Chunk {
    /// The chunk should be processed in the current frame
    pub fn hibernating(&self) -> bool {
        self.hibernating
    }

    /// The chunk should be processed in the next frame
    pub fn should_process_next_frame(&self) -> bool {
        self.should_process_next_frame
    }
}

impl Chunk {
    /// Get an immutable reference to the corresponding entity, if it exists.
    pub fn get(&self, coords: &IVec2) -> Option<&Entity> {
        self.chunk.get(coords)
    }
}

impl Chunk {
    /// Iterate through all key, value instances of the entity map
    pub fn iter(&self) -> impl Iterator<Item = (&IVec2, &Entity)> {
        self.chunk.iter()
    }

    /// Iterate through all entities in the chunk
    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.chunk.values()
    }

    /// Parallel iter through all the key, value instances of the entity map
    pub fn par_iter(&self) -> impl IntoParallelIterator<Item = (&IVec2, &Entity)> {
        self.chunk.par_iter()
    }
}

impl Chunk {
    /// Clear all existing entities from the map
    /// > **⚠️ Warning:**
    /// > Calling this method will cause major breakage to the simulation if entities are not
    /// simultaneously cleared within the same system from which this method was called.
    pub fn clear(&mut self) {
        self.chunk.clear();
    }

    /// Remove a entity from the map
    /// > **⚠️ Warning:**
    /// > Calling this method will cause major breakage to the simulation if entities are not
    /// simultaneously cleared within the same system from which this method was called.
    pub fn remove(&mut self, coords: &IVec2) -> Option<Entity> {
        self.should_process_next_frame = true;
        self.chunk.remove(coords)
    }

    /// Inserts a new entity at a given coordinate if it is not already occupied. Calls to this method will
    /// wake up the subject chunk.
    pub fn insert_no_overwrite(&mut self, coords: IVec2, entity: Entity) -> &mut Entity {
        self.should_process_next_frame = true;
        self.chunk.entry(coords).or_insert(entity)
    }

    /// Inserts a new entity at a given coordinate irrespective of its occupied state. Calls to this method will
    /// wake up the subject chunk.
    pub fn insert_overwrite(&mut self, coords: IVec2, entity: Entity) -> Option<Entity> {
        self.should_process_next_frame = true;
        self.chunk.insert(coords, entity)
    }
}

/// Resets all chunks in preparation for the next frame
///
/// When this system runs, all chunks are checked to see if they should be awakened in preparation for the next frame
/// (see field `should_process_this_frame`). After this, their 'activated' status is reset (see field
/// `should_process_next_frame`)
pub fn reset_chunks(commands: Commands, mut map: ResMut<ChunkMap>) {
    map.reset_chunks(commands);
}

/// Event reader for particle type updates
pub fn on_change_particle(
    mut ev_change_particle: EventReader<MutateParticleEvent>,
    mut particle_query: Query<&mut Particle>
) {
    for ev in ev_change_particle.read() {
	let mut particle = particle_query.get_mut(ev.entity).unwrap();
	particle.name  = ev.particle.name.clone();
    }
}