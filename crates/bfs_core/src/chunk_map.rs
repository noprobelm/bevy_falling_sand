use ahash::{HashMap, HashMapExt};
use bevy::prelude::*;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;

use crate::{
    Coordinates, Particle, ParticleSimulationSet, ParticleType, ParticleTypeMap,
    RemoveParticleEvent, SimulationRun,
};

pub struct ChunkMapPlugin;

impl Plugin for ChunkMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkMap>()
            .add_event::<ClearMapEvent>()
            .add_event::<ClearParticleTypeChildrenEvent>()
            .add_systems(
                Update,
                reset_chunks
                    .after(ParticleSimulationSet)
                    .run_if(resource_exists::<SimulationRun>),
            )
            .add_observer(on_remove_particle)
            .add_observer(on_clear_chunk_map)
            .add_observer(on_clear_particle_type_children);
    }
}

#[derive(Resource, Debug, Clone)]
pub struct ChunkMap {
    pub chunks: Vec<Chunk>,
}

impl Default for ChunkMap {
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
    fn chunk_index(&self, coord: &IVec2) -> usize {
        const OFFSET: i32 = 512;
        const GRID_WIDTH: usize = 32;

        let col = ((coord.x + OFFSET) >> 5) as usize;
        let row = ((OFFSET - coord.y) >> 5) as usize;

        row * GRID_WIDTH + col
    }

    pub fn chunk(&self, coord: &IVec2) -> Option<&Chunk> {
        let index = self.chunk_index(coord);
        self.chunks.get(index)
    }

    fn chunk_mut(&mut self, coord: &IVec2) -> Option<&mut Chunk> {
        let index = self.chunk_index(coord);
        self.chunks.get_mut(index)
    }
}

impl ChunkMap {
    pub fn clear(&mut self) {
        for map in &mut self.chunks {
            map.clear();
        }
    }

    pub fn remove(&mut self, coords: &IVec2) -> Option<Entity> {
        self.chunk_mut(&coords).unwrap().remove(coords)
    }
}

impl ChunkMap {
    pub fn iter_chunks(&self) -> impl Iterator<Item = &Chunk> {
        self.chunks.iter()
    }

    pub fn iter_chunks_mut(&mut self) -> impl Iterator<Item = &mut Chunk> {
        self.chunks.iter_mut()
    }
}

impl ChunkMap {
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
    pub fn insert_no_overwrite(&mut self, coords: IVec2, entity: Entity) -> &mut Entity {
        let chunk = self.chunk_mut(&coords).unwrap();
        chunk.insert_no_overwrite(coords, entity)
    }

    pub fn insert_overwrite(&mut self, coords: IVec2, entity: Entity) -> Option<Entity> {
        let chunk = self.chunk_mut(&coords).unwrap();
        chunk.insert_overwrite(coords, entity)
    }

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

    pub fn entity(&self, coords: &IVec2) -> Option<&Entity> {
        self.chunk(&coords).unwrap().get(coords)
    }

    #[allow(unused)]
    pub fn iter(&self) -> impl Iterator<Item = (&IVec2, &Entity)> {
        self.chunks.iter().flat_map(|chunk| chunk.iter())
    }

    pub fn par_iter(&self) -> impl IntoParallelIterator<Item = (&IVec2, &Entity)> {
        self.chunks.par_iter().flat_map(|chunk| chunk.par_iter())
    }

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

#[derive(Debug, Clone)]
pub struct Chunk {
    chunk: HashMap<IVec2, Entity>,
    region: IRect,
    dirty_rect: Option<IRect>,
    prev_dirty_rect: Option<IRect>,
    should_process_next_frame: bool,
    hibernating: bool,
}

impl Chunk {
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
    pub fn min(&self) -> &IVec2 {
        &self.region.min
    }

    pub fn max(&self) -> &IVec2 {
        &self.region.max
    }
}

impl Chunk {
    pub fn hibernating(&self) -> bool {
        self.hibernating
    }

    pub fn should_process_next_frame(&self) -> bool {
        self.should_process_next_frame
    }
}

impl Chunk {
    pub fn get(&self, coords: &IVec2) -> Option<&Entity> {
        self.chunk.get(coords)
    }
}

impl Chunk {
    pub fn iter(&self) -> impl Iterator<Item = (&IVec2, &Entity)> {
        self.chunk.iter()
    }

    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.chunk.values()
    }

    pub fn par_iter(&self) -> impl IntoParallelIterator<Item = (&IVec2, &Entity)> {
        self.chunk.par_iter()
    }
}

impl Chunk {
    pub fn clear(&mut self) {
        self.chunk.clear();
    }

    pub fn remove(&mut self, coords: &IVec2) -> Option<Entity> {
        self.should_process_next_frame = true;
        self.chunk.remove(coords)
    }

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
    pub fn dirty_rect(&self) -> Option<IRect> {
        self.dirty_rect
    }

    pub fn prev_dirty_rect(&self) -> Option<IRect> {
        self.prev_dirty_rect
    }

    pub fn empty(&self) -> bool {
        self.chunk.len() == 0
    }
}

pub fn reset_chunks(mut map: ResMut<ChunkMap>) {
    map.reset_chunks();
}

#[derive(Event)]
pub struct ClearMapEvent;

#[derive(Event)]
pub struct ClearParticleTypeChildrenEvent(pub String);

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
