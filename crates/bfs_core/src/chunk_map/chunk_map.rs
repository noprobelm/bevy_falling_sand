//! Resources providing mapping functionality to particle positions and types.
use ahash::{HashMap, HashMapExt};
use bevy::prelude::*;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;

use crate::{ParticleSimulationSet, ParticleTypeMap, RemoveParticleEvent, SimulationRun};

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
        .init_resource::<ChunkMap>()
        .observe(on_remove_particle)
        .observe(on_clear_chunk_map);
    }
}

/// The selected optimization strategy influences the way we simulate particle movement and ChunkMap
/// behaviors.
#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum OptimizationStrategy {
    /// `Hibernation` mode implements an algorithm that will *only* process the particles within 
    /// a chunk if they need to be. This mode will reliably evaluate all particles requiring 
    /// movement while maintaining the benefit of potentially simulating only a small number of
    /// particles compared to the total number in the simulation.
    ///
    /// This mode works by adding the `Hibernating` component for particles, as well as the
    /// `Chunk.should_process_next_frame` and `Chunk.hibernating` fields for flagging whether the
    /// particles within a chunk should be processed in the following frame, or should be
    /// processed in the current frame (respectively).
    ///
    /// Each frame, the system that handles particle movement will filter out particles
    /// with the `Hibernating` component, iterating only through "awakened" particles.
    /// If a particle moves anywhere within a chunk's region during a frame, or a particle is
    /// inserted or removed from the chunk, its `Chunk.should_process_next_frame` is set to `true.`
    ///
    /// If a particle moves along the bounds of a chunk's region, the chunk's neighbor's
    /// `should_process_next_frame` field is also set to `true.`
    ///
    /// At the end of each frame, we iterate through each `Chunk` in the `ChunkMap`.
    ///   - If a `should_process_next_frame` == `true` and `hibernating` == `true`:
    ///       - Remove the `Hibernating` component from all particle entities
    ///       - Set `Chunk.hibernating` to `false`
    ///   - Else if `should_process_next_frame` == `false` and `hibernating` == `false`:
    ///       - insert the `Hibernating` component to all particle entities
    ///       - Set `Chunk.hibernating` to `true`
    ///   - Reset `Chunk.should_process_next_frame` to `false` for all chunks.
    Hibernation,
    /// `DirtyRect` mode is moderately faster than `Hibernation`, but as a tradeoff some particles
    /// that could br processed may be excluded from processing for a frame.
    ///
    /// This mode works by adding `Chunk.dirty_rect` and `Chunk.prev_dirty_rect` fields, each of
    /// which are `Option<IRect>`. A dirty rect is simply the smallest possible bounding box around
    /// all particles within a chunk that have moved for a given frame.
    ///
    /// As we iterate through each particle, we check to see if it is contained within the region of
    /// `Chunk.prev_dirty_rect`:
    ///   - If yes, we will process this particle.
    ///   - If no, there is a chance we will skip the particle.
    ///
    /// Even when setting the chance to skip as very high (>= 90%), particle movement looks natural.
    /// If you set this chance to high, it will become increasingly evident that particles are
    /// floating mid air. If set to 100%, particles will remain suspended until another particle
    /// attempts to change positions with it.
    ///
    /// Each time a particle moves, `Chunk.dirty_rect` is created if it didn't exist. Otherwise,
    /// a new IRect is created as a union point between the existing IRect and the particle's
    /// coordinate.
    ///
    /// At the end of each frame, `Chunk.prev_dirty_rect` is cloned from `Chunk.dirty_rect`, and
    /// `Chunk.dirty_rect` is reset to its original `None` state.
    #[default]
    DirtyRect,
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
    pub fn reset_chunks(&mut self) {
        for chunk in &mut self.chunks {
            chunk.prev_dirty_rect = chunk.dirty_rect;
            chunk.dirty_rect = None;
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
    pub fn should_process(&self, coords: &IVec2) -> bool {
        if let Some(dirty_rect) = self.chunk(coords).unwrap().prev_dirty_rect {
            return dirty_rect.contains(*coords);
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
}

impl Chunk {
    /// Creates a new Chunk instance
    pub fn new(upper_left: IVec2, lower_right: IVec2) -> Chunk {
        Chunk {
            chunk: HashMap::with_capacity(1024),
            region: IRect::from_corners(upper_left, lower_right),
            dirty_rect: None,
            prev_dirty_rect: None,
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
        self.chunk.remove(coords)
    }

    /// Inserts a new entity at a given coordinate if it is not already occupied. Calls to this method will
    /// wake up the subject chunk.
    pub fn insert_no_overwrite(&mut self, coords: IVec2, entity: Entity) -> &mut Entity {
        // Extend the dirty rect to include the newly added particle
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
