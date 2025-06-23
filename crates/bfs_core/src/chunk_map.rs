use bevy::platform::collections::{hash_map::Entry, HashMap};
use bevy::platform::hash::FixedHasher;
use bevy::prelude::*;

use crate::{
    Coordinates, Particle, ParticleSimulationSet, ParticleType, ParticleTypeMap,
    RemoveParticleEvent,
};

const OFFSET: i32 = 512;
const GRID_WIDTH: usize = 32;

pub struct ParticleMapPlugin;

impl Plugin for ParticleMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticleMap>()
            .add_event::<ClearMapEvent>()
            .add_event::<ClearParticleTypeChildrenEvent>()
            .add_systems(Update, reset_chunks.after(ParticleSimulationSet))
            .add_observer(on_remove_particle)
            .add_observer(on_clear_particle_map)
            .add_observer(on_clear_particle_type_children);
    }
}

#[derive(Resource, Debug, Clone)]
pub struct ParticleMap {
    chunks: Vec<Chunk>,
}

impl Default for ParticleMap {
    fn default() -> Self {
        const CHUNK_SIZE: i32 = 32;
        const GRID_SIZE: i32 = GRID_WIDTH as i32;
        const GRID_OFFSET: i32 = 512;

        let mut chunks = Vec::with_capacity((GRID_SIZE.pow(2)) as usize);
        for i in 0..GRID_SIZE.pow(2) {
            let row = i / GRID_SIZE;
            let col = i % GRID_SIZE;

            let x = col * CHUNK_SIZE - GRID_OFFSET;
            let y = GRID_OFFSET - row * CHUNK_SIZE;
            let upper_left = IVec2::new(x, y - (CHUNK_SIZE - 1));
            let lower_right = IVec2::new(x + (CHUNK_SIZE - 1), y);

            let chunk = Chunk::new(upper_left, lower_right);
            chunks.push(chunk);
        }

        ParticleMap { chunks }
    }
}

impl ParticleMap {
    fn index(&self, coord: &IVec2) -> usize {
        let col = ((coord.x + OFFSET) >> 5) as usize;
        let row = ((OFFSET - coord.y) >> 5) as usize;

        row * GRID_WIDTH + col
    }

    pub fn chunk(&self, coord: &IVec2) -> Option<&Chunk> {
        let index = self.index(coord);
        self.chunks.get(index)
    }

    pub fn chunk_mut(&mut self, coord: &IVec2) -> Option<&mut Chunk> {
        let index = self.index(coord);
        self.chunks.get_mut(index)
    }

    pub fn iter_chunks(&self) -> impl Iterator<Item = &Chunk> {
        self.chunks.iter()
    }

    pub fn iter_chunks_mut(&mut self) -> impl Iterator<Item = &mut Chunk> {
        self.chunks.iter_mut()
    }

    pub fn get(&self, coordinates: &IVec2) -> Option<&Entity> {
        let index = self.index(coordinates);
        if let Some(chunk) = self.chunks.get(index) {
            chunk.get(coordinates)
        } else {
            None
        }
    }

    pub fn remove(&mut self, coordinates: &IVec2) -> Option<Entity> {
        let index = self.index(coordinates); // Calculate index first
        if let Some(chunk) = self.chunks.get_mut(index) {
            chunk.remove(coordinates)
        } else {
            None
        }
    }

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
            if let Some(dirty_rect) = chunk.dirty_rect {
                chunk.prev_dirty_rect = Some(dirty_rect.inflate(5).intersect(chunk.region));
            } else {
                chunk.prev_dirty_rect = None;
            }
            chunk.dirty_rect = None;
        })
    }

    pub fn clear(&mut self) {
        self.chunks.iter_mut().for_each(|chunk| {
            chunk.clear();
            chunk.dirty_rect = None;
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ChunkGroup {
    One,
    Two,
    Three,
    Four,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chunk {
    chunk: HashMap<IVec2, Entity>,
    region: IRect,
    dirty_rect: Option<IRect>,
    prev_dirty_rect: Option<IRect>,
}

impl Chunk {
    pub fn new(upper_left: IVec2, lower_right: IVec2) -> Chunk {
        Chunk {
            chunk: HashMap::with_capacity(1024),
            region: IRect::from_corners(upper_left, lower_right),
            dirty_rect: None,
            prev_dirty_rect: None,
        }
    }

    fn set_dirty_rect(&mut self, coordinates: IVec2) {
        if let Some(dirty_rect) = self.dirty_rect {
            self.dirty_rect = Some(dirty_rect.union_point(coordinates));
        } else {
            self.dirty_rect = Some(IRect::from_center_size(coordinates, IVec2::ONE));
        }
    }
}

impl Chunk {
    pub fn region(&self) -> IRect {
        self.region
    }

    pub fn get(&self, coordinates: &IVec2) -> Option<&Entity> {
        self.chunk.get(coordinates)
    }

    pub fn insert(&mut self, coordinates: IVec2, item: Entity) -> Option<Entity> {
        self.set_dirty_rect(coordinates);
        self.chunk.insert(coordinates, item)
    }

    pub fn entry(&mut self, coordinates: IVec2) -> Entry<'_, IVec2, Entity, FixedHasher> {
        self.set_dirty_rect(coordinates);
        self.chunk.entry(coordinates)
    }

    pub fn remove(&mut self, coordinates: &IVec2) -> Option<Entity> {
        self.set_dirty_rect(*coordinates);
        self.chunk.remove(coordinates)
    }

    pub fn clear(&mut self) {
        self.chunk.clear();
        self.dirty_rect = None;
    }

    pub fn dirty_rect(&self) -> Option<IRect> {
        self.dirty_rect
    }

    pub fn prev_dirty_rect(&self) -> Option<IRect> {
        self.prev_dirty_rect
    }

    pub fn iter(&self) -> impl Iterator<Item = (&IVec2, &Entity)> {
        self.chunk.iter()
    }
}

fn reset_chunks(mut map: ResMut<ParticleMap>) {
    map.reset_chunks();
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
    if let Some(entity) = map.remove(&trigger.event().coordinates) {
        if trigger.event().despawn {
            commands.entity(entity).remove::<ChildOf>().despawn();
        } else {
            commands.entity(entity).remove::<ChildOf>();
        }
    }
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
    particle_query: Query<&Coordinates, With<Particle>>,
    parent_query: Query<&Children, With<ParticleType>>,
    particle_parent_map: Res<ParticleTypeMap>,
    mut map: ResMut<ParticleMap>,
) {
    let particle_type = trigger.event().0.clone();
    if let Some(parent_entity) = particle_parent_map.get(&particle_type) {
        if let Ok(children) = parent_query.get(*parent_entity) {
            children.iter().for_each(|child_entity| {
                if let Ok(coordinates) = particle_query.get(child_entity) {
                    map.remove(&coordinates.0);
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
