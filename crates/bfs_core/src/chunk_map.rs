use bevy::platform::collections::{hash_map::Entry, HashMap};
use bevy::platform::hash::FixedHasher;
use bevy::prelude::*;

use crate::{
    ChunkRng, Coordinates, Particle, ParticleSimulationSet, ParticleType, ParticleTypeMap,
    RemoveParticleEvent, SimulationRun,
};

const OFFSET: i32 = 512;
const GRID_WIDTH: usize = 32;

pub struct ChunkMapPlugin;

impl Plugin for ChunkMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ClearMapEvent>()
            .add_event::<ClearParticleTypeChildrenEvent>()
            .add_systems(Startup, setup)
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
    pub chunks: Vec<Entity>,
}

impl ChunkMap {
    fn index(&self, coord: &IVec2) -> usize {
        let col = ((coord.x + OFFSET) >> 5) as usize;
        let row = ((OFFSET - coord.y) >> 5) as usize;

        row * GRID_WIDTH + col
    }

    pub fn chunk(&self, coord: &IVec2) -> Option<&Entity> {
        let index = self.index(coord);
        self.chunks.get(index)
    }
}

impl ChunkMap {
    pub fn iter_chunks(&self) -> impl Iterator<Item = &Entity> {
        self.chunks.iter()
    }

    pub fn remove(
        &mut self,
        coords: &IVec2,
        chunk_query: &mut Query<&mut Chunk>,
    ) -> Option<Entity> {
        chunk_query
            .get_mut(*self.chunk(coords).unwrap())
            .unwrap()
            .remove(coords)
    }

    pub fn swap(&mut self, first: IVec2, second: IVec2, chunk_query: &mut Query<&mut Chunk>) {
        let first_chunk_idx = self.index(&first);
        let second_chunk_idx = self.index(&second);

        // Short-circuit if both positions are in the same chunk to save ourselves a hashmap lookup.
        if first_chunk_idx == second_chunk_idx {
            let mut chunk = chunk_query.get_mut(*self.chunk(&first).unwrap()).unwrap();

            let entity_first = chunk.remove(&first).unwrap();
            if let Some(entity_second) = chunk.remove(&second) {
                chunk.insert(first, entity_second);
                chunk.insert(second, entity_first);
            } else {
                chunk.insert(second, entity_first);
            }
        } else {
            let entity_first = chunk_query
                .get_mut(*self.chunk(&first).unwrap())
                .unwrap()
                .remove(&first)
                .unwrap();
            if let Some(entity_second) = chunk_query
                .get_mut(*self.chunk(&second).unwrap())
                .unwrap()
                .remove(&second)
            {
                chunk_query
                    .get_mut(*self.chunk(&first).unwrap())
                    .unwrap()
                    .insert(first, entity_second);
                chunk_query
                    .get_mut(*self.chunk(&second).unwrap())
                    .unwrap()
                    .insert(second, entity_first);
            } else {
                chunk_query
                    .get_mut(*self.chunk(&second).unwrap())
                    .unwrap()
                    .insert(second, entity_first);
            }
        }
    }

    pub fn entity(&self, coords: &IVec2, chunk_query: &mut Query<&mut Chunk>) -> Option<Entity> {
        chunk_query
            .get(*self.chunk(coords).unwrap())
            .unwrap()
            .get(coords)
            .copied()
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Component)]
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
}

impl Chunk {
    pub fn min(&self) -> &IVec2 {
        &self.region.min
    }

    pub fn max(&self) -> &IVec2 {
        &self.region.max
    }

    pub fn region(&self) -> &IRect {
        &self.region
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

    pub fn coordinates(&self) -> impl Iterator<Item = &IVec2> {
        self.chunk.keys()
    }

    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.chunk.values()
    }
}

impl Chunk {
    fn set_dirty_rect(&mut self, coordinates: IVec2) {
        if let Some(dirty_rect) = self.dirty_rect {
            self.dirty_rect = Some(dirty_rect.union_point(coordinates));
        } else {
            self.dirty_rect = Some(IRect::from_center_size(coordinates, IVec2::ONE));
        }
    }

    pub fn clear(&mut self) {
        self.chunk.clear();
    }

    pub fn remove(&mut self, coordinates: &IVec2) -> Option<Entity> {
        self.set_dirty_rect(*coordinates);
        self.chunk.remove(coordinates)
    }

    pub fn entry(&mut self, coordinates: IVec2) -> Entry<'_, IVec2, Entity, FixedHasher> {
        self.set_dirty_rect(coordinates);
        self.chunk.entry(coordinates)
    }
    pub fn insert(&mut self, coordinates: IVec2, entity: Entity) -> Option<Entity> {
        // Extend the dirty rect to include the newly added particle
        self.set_dirty_rect(coordinates);
        self.chunk.insert(coordinates, entity)
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
        self.chunk.is_empty()
    }
}

fn setup(mut commands: Commands) {
    let mut map = ChunkMap { chunks: vec![] };

    for i in 0..32_i32.pow(2) {
        let x = (i % 32) * 32 - 512;
        let y = 512 - (i / 32) * 32;
        let upper_left = IVec2::new(x, y - 31);
        let lower_right = IVec2::new(x + 31, y);
        let chunk = Chunk::new(upper_left, lower_right);
        let id = commands.spawn((chunk, ChunkRng::default())).id();

        map.chunks.push(id);
    }
    commands.insert_resource(map);
}

fn reset_chunks(mut chunk_query: Query<&mut Chunk>) {
    chunk_query.iter_mut().for_each(|mut chunk| {
        if let Some(dirty_rect) = chunk.dirty_rect {
            chunk.prev_dirty_rect = Some(dirty_rect.inflate(5).intersect(chunk.region));
        } else {
            chunk.prev_dirty_rect = None;
        }
        chunk.dirty_rect = None;
    });
}

#[derive(Event)]
pub struct ClearMapEvent;

#[derive(Event)]
pub struct ClearParticleTypeChildrenEvent(pub String);

pub fn on_remove_particle(
    trigger: Trigger<RemoveParticleEvent>,
    mut commands: Commands,
    mut map: ResMut<ChunkMap>,
    mut chunk_query: Query<&mut Chunk>,
) {
    if let Some(entity) = map.remove(&trigger.event().coordinates, &mut chunk_query) {
        if trigger.event().despawn {
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
    mut chunk_query: Query<&mut Chunk>,
) {
    particle_parent_map.iter().for_each(|(_, entity)| {
        commands.entity(*entity).despawn_related::<Children>();
    });

    chunk_query.iter_mut().for_each(|mut chunk| chunk.clear());
}

pub fn on_clear_particle_type_children(
    trigger: Trigger<ClearParticleTypeChildrenEvent>,
    mut commands: Commands,
    particle_query: Query<&Coordinates, With<Particle>>,
    parent_query: Query<&Children, With<ParticleType>>,
    particle_parent_map: Res<ParticleTypeMap>,
    mut map: ResMut<ChunkMap>,
    mut chunk_query: Query<&mut Chunk>,
) {
    let particle_type = trigger.event().0.clone();
    if let Some(parent_entity) = particle_parent_map.get(&particle_type) {
        if let Ok(children) = parent_query.get(*parent_entity) {
            children.iter().for_each(|child_entity| {
                if let Ok(coordinates) = particle_query.get(child_entity) {
                    map.remove(&coordinates.0, &mut chunk_query);
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
