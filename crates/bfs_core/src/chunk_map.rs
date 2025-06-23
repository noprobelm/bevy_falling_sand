use bevy::platform::collections::{hash_map::Entry, HashMap};
use bevy::platform::hash::FixedHasher;
use bevy::prelude::*;

use crate::{
    Coordinates, Particle, ParticleSimulationSet, ParticleType, ParticleTypeMap,
    RemoveParticleEvent,
};

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

#[derive(Clone, Eq, PartialEq, Debug, Resource)]
pub struct ParticleMap {
    pub size: usize,
    pub particles_per_chunk: usize,
    chunks: Vec<Chunk>,
    flat_map_offset_value: usize,
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
    pub fn new(map_size: usize, chunk_size: usize) -> Self {
        if !map_size.is_power_of_two() {
            panic!("Particle map size must be a power of 2")
        }
        let num_chunks: usize = map_size.pow(2);
        let grid_offset: usize = map_size.pow(2) / 2;
        let mut chunks = Vec::with_capacity(num_chunks);
        for i in 0..(num_chunks as i32) {
            let row = i / map_size as i32;
            let col = i % map_size as i32;

            let x = col * chunk_size as i32 - grid_offset as i32;
            let y = grid_offset as i32 - row * chunk_size as i32;
            let upper_left = IVec2::new(x, y - (chunk_size as i32 - 1));
            let lower_right = IVec2::new(x + (chunk_size as i32 - 1), y);

            let chunk = Chunk::new(upper_left, lower_right, map_size);
            chunks.push(chunk);
        }
        ParticleMap {
            chunks,
            size: map_size,
            particles_per_chunk: chunk_size.pow(2),
            flat_map_offset_value: grid_offset,
            chunk_shift: chunk_size.trailing_zeros(),
        }
    }
    fn index(&self, coord: &IVec2) -> usize {
        let col = ((coord.x + self.flat_map_offset_value as i32) >> self.chunk_shift) as usize;
        let row = ((self.flat_map_offset_value as i32 - coord.y) >> self.chunk_shift) as usize;
        row * self.size + col
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
    pub fn new(upper_left: IVec2, lower_right: IVec2, size: usize) -> Chunk {
        Chunk {
            chunk: HashMap::with_capacity(size.pow(2)),
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
