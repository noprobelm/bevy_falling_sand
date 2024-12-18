//! Resources providing mapping functionality to particle positions and types.
use ahash::{HashMap, HashMapExt};
use bevy::prelude::*;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;

use crate::{
    Coordinates, Particle, ParticleSimulationSet, ParticleType, ParticleTypeMap,
    RemoveParticleEvent, SimulationRun,
};

/// Plugin for mapping particles to coordinate space.
pub struct ChunkMapPlugin;

impl Plugin for ChunkMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            reset_chunks
                .after(ParticleSimulationSet)
                .run_if(resource_exists::<SimulationRun>),
        )
        .add_event::<ClearMapEvent>()
        .add_event::<ClearParticleTypeChildrenEvent>()
        .init_resource::<ChunkMap>()
        .observe(on_remove_particle)
        .observe(on_clear_chunk_map)
        .observe(on_clear_particle_type_children);
    }
}

/// Chunk map for segmenting collections of entities into coordinate-based chunks.
#[derive(Resource, Debug, Clone)]
pub struct ChunkMap {
    /// The entity chunk maps
    pub chunks: Vec<Chunk>,
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
    pub fn chunk(&self, coord: &IVec2) -> Option<&Chunk> {
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

    /// Immutable iterator over all chunks.
    pub fn iter_chunks_mut(&mut self) -> impl Iterator<Item = &mut Chunk> {
        self.chunks.iter_mut()
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
    pub fn reset_chunks(&mut self) {
        for chunk in &mut self.chunks {
            chunk.prev_dirty_rect = chunk.dirty_rect;
            chunk.dirty_rect = None;

            match (chunk.should_process_next_frame, chunk.hibernating) {
                (true, true) => {
                    chunk.hibernating = false;
                }
                (false, false) => {
                    chunk.hibernating = true;
                }
                _ => {}
            }

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

        // Short-circuit if both positions are in the same chunk to save ourselves a hashmap lookup.
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

    /// Should we process the entity this frame
    pub fn should_process_this_frame(&self, coords: &IVec2) -> bool {
        if let Some(chunk) = self.chunk(coords) {
            if chunk.hibernating() == true {
                return false;
            } else if let Some(dirty_rect) = chunk.prev_dirty_rect() {
                return dirty_rect.contains(*coords);
            }
        }
        false
    }
}

/// A chunk which stores location information for entities.
#[derive(Debug, Clone)]
pub struct Chunk {
    /// The chunk containing the particle data
    chunk: HashMap<IVec2, Entity>,
    /// The region of the chunk
    region: IRect,
    /// A dirty rect for all particles that have moved in the current frame
    dirty_rect: Option<IRect>,
    /// A dirty rect for all particles that moved in the previous frame
    prev_dirty_rect: Option<IRect>,
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
            region: IRect::from_corners(upper_left, lower_right),
            dirty_rect: None,
            prev_dirty_rect: None,
            should_process_next_frame: false,
            hibernating: false,
        }
    }
}

impl Chunk {
    /// The minimum (upper left) point of the chunk's area
    pub fn min(&self) -> &IVec2 {
        &self.region.min
    }

    /// The maximum (lower right) point of the chunk's area
    pub fn max(&self) -> &IVec2 {
        &self.region.max
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
        // Extend the dirty rect to include the newly added particle
        self.should_process_next_frame = true;
        if let Some(dirty_rect) = self.dirty_rect {
            self.dirty_rect = Some(dirty_rect.union_point(coords));
        } else {
            self.dirty_rect = Some(IRect::from_center_size(coords, IVec2::ONE));
        }

        self.chunk.entry(coords).or_insert(entity)
    }

    /// Inserts a new entity at a given coordinate irrespective of its occupied state. Calls to this method will
    /// wake up the subject chunk.
    pub fn insert_overwrite(&mut self, coords: IVec2, entity: Entity) -> Option<Entity> {
        self.should_process_next_frame = true;
        // Extend the dirty rect to include the newly added particle
        if let Some(dirty_rect) = self.dirty_rect {
            self.dirty_rect = Some(dirty_rect.union_point(coords));
        } else {
            self.dirty_rect = Some(IRect::from_center_size(coords, IVec2::ONE));
        }

        self.chunk.insert(coords, entity)
    }
}

impl Chunk {
    /// Gets the dirty rect from the chunk
    pub fn dirty_rect(&self) -> Option<IRect> {
        self.dirty_rect
    }

    /// Gets the previous dirty rect from the chunk
    pub fn prev_dirty_rect(&self) -> Option<IRect> {
        self.prev_dirty_rect
    }

    /// Is the chunk empty
    pub fn empty(&self) -> bool {
        self.chunk.len() == 0
    }
}

/// Resets all chunks in preparation for the next frame
pub fn reset_chunks(mut map: ResMut<ChunkMap>) {
    map.reset_chunks();
}

/// Remove all particles from the simulation.
#[derive(Event)]
pub struct ClearMapEvent;

/// Remove all child particles of a specified type from the simulation.
#[derive(Event)]
pub struct ClearParticleTypeChildrenEvent(pub String);

/// RemoveParticle event is triggered.
pub fn on_remove_particle(
    trigger: Trigger<RemoveParticleEvent>,
    mut commands: Commands,
    mut map: ResMut<ChunkMap>,
) {
    if let Some(entity) = map.remove(&trigger.event().coordinates) {
        if trigger.event().despawn == true {
            commands.entity(entity).remove_parent().despawn();
        } else {
            commands.entity(entity).remove_parent();
        }
    }
}

/// Observer for clearing all particles from the world as soon as a ClearMapEvent is triggered.
pub fn on_clear_chunk_map(
    _trigger: Trigger<ClearMapEvent>,
    mut commands: Commands,
    particle_parent_map: Res<ParticleTypeMap>,
    mut map: ResMut<ChunkMap>,
) {
    particle_parent_map.iter().for_each(|(_, entity)| {
        commands.entity(*entity).despawn_descendants();
    });

    map.clear();
}

/// Observer for clearing dynamic particles
pub fn on_clear_particle_type_children(
    trigger: Trigger<ClearParticleTypeChildrenEvent>,
    mut commands: Commands,
    particle_query: Query<&Coordinates, With<Particle>>,
    parent_query: Query<&Children, With<ParticleType>>,

    particle_parent_map: Res<ParticleTypeMap>,
    mut map: ResMut<ChunkMap>,
) {
    let particle_type = trigger.event().0.clone();
    if let Some(parent_entity) = particle_parent_map.get(&particle_type) {
        if let Ok(children) = parent_query.get(*parent_entity) {
            children.iter().for_each(|child_entity| {
                if let Ok(coordinates) = particle_query.get(*child_entity) {
                    map.remove(&coordinates.0);
                } else {
                    // If this happens, something is seriously amiss.
                    error!("No child entity found for particle type '{particle_type}' while removing child from chunk map.")
                }
            });
            commands.entity(*parent_entity).despawn_descendants();
        }
    } else {
        warn!("Ignoring particle type '{particle_type}': not found in particle type map.");
    }
}
