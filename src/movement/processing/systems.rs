use crate::movement::{
    AirResistance, Density, Momentum, Movement, MovementRng, ParticleResistor, Speed,
};
use std::mem;
use std::sync::Mutex;

use super::MovementState;
use crate::core::{
    ChunkCoord, ChunkDirtyState, ChunkIndex, ChunkRegion, GridPosition, ParticleMap, ParticleRng,
};
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy::tasks::ComputeTaskPool;
use bevy_turborand::prelude::*;

type ObstructedDirections = [bool; 9];

struct SendPtr<T>(*mut T);

unsafe impl<T> Send for SendPtr<T> {}

impl<T> Copy for SendPtr<T> {}

impl<T> Clone for SendPtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> SendPtr<T> {
    const unsafe fn new(ptr: *mut T) -> Self {
        Self(ptr)
    }

    const fn get(&self) -> *mut T {
        self.0
    }
}

/// Maps a direction vector to an index in a 3x3 grid (0-8).
/// Layout: [(-1,-1), (0,-1), (1,-1), (-1,0), (0,0), (1,0), (-1,1), (0,1), (1,1)]
///         [   0   ,   1  ,   2   ,   3  ,   4  ,   5  ,   6   ,   7  ,   8   ]
#[inline(always)]
const fn get_direction_index(dir: IVec2) -> usize {
    let sx = dir.x.signum() + 1;
    let sy = dir.y.signum() + 1;
    (sy * 3 + sx) as usize
}

type ParticleMovementByChunksQuery<'a> = (
    &'a mut GridPosition,
    &'a mut MovementRng,
    &'a mut Speed,
    Option<&'a mut Momentum>,
    &'a Movement,
    &'a AirResistance,
    Option<&'a ParticleResistor>,
);

type ParticleMovementByParticlesQuery<'a> = (
    Entity,
    &'a mut GridPosition,
    &'a mut MovementRng,
    &'a mut Speed,
    Option<&'a mut Momentum>,
    &'a Movement,
    &'a AirResistance,
    Option<&'a ParticleResistor>,
);

#[allow(unused_mut, clippy::too_many_lines)]
#[allow(clippy::needless_pass_by_value)]
pub(super) fn par_handle_movement_by_chunks(
    mut particle_query: Query<ParticleMovementByChunksQuery>,
    mut density_query: Query<&Density>,
    mut map: ResMut<ParticleMap>,
    mut movement_state: ResMut<MovementState>,
    mut global_rng: ResMut<GlobalRng>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<(&ChunkRegion, &mut ChunkDirtyState)>,
) {
    movement_state.visited_entities.clear();

    let dirty_positions = Mutex::new(Vec::<IVec2>::new());

    let mut groups = chunk_index.partition_chunks_by_group();

    let group_order: [usize; 4] = {
        let mut order = [0, 1, 2, 3];
        global_rng.shuffle(&mut order);
        order
    };

    let seed = global_rng.u64(..);
    let mut all_group_visited: Vec<HashSet<Entity>> = Vec::with_capacity(4);

    unsafe {
        let map_ptr = &raw mut *map;
        let query_ptr = (&raw mut particle_query).cast::<Query<ParticleMovementByChunksQuery>>();
        let density_query_ptr = (&raw mut density_query).cast::<Query<&Density>>();
        let dirty_ptr = SendPtr::new((&raw const dirty_positions).cast_mut());

        for group_idx in group_order {
            let chunk_coords = &mut groups[group_idx];

            global_rng.shuffle(chunk_coords);

            let visited_entities_by_chunk = Mutex::new(Vec::<HashSet<Entity>>::new());

            let map_ptr_send = SendPtr::new(map_ptr);
            let query_ptr_send = SendPtr::new(query_ptr);
            let density_ptr_send = SendPtr::new(density_query_ptr);
            let visited_ptr = SendPtr::new((&raw const visited_entities_by_chunk).cast_mut());

            let chunks_to_process: Vec<(ChunkCoord, Option<IRect>, Vec<IVec2>)> = chunk_coords
                .iter()
                .filter_map(|&coord| {
                    let chunk_entity = chunk_index.get(coord)?;
                    let (region, dirty_state) = chunk_query.get(chunk_entity).ok()?;
                    if !dirty_state.is_dirty() {
                        return None;
                    }
                    let border_positions = dirty_state.border_strip_positions(region.region());
                    Some((coord, dirty_state.current, border_positions))
                })
                .collect();

            ComputeTaskPool::get().scope(|scope| {
                for (_coord, dirty_rect, border_positions) in chunks_to_process {
                    let map_ptr_copy = map_ptr_send;
                    let query_ptr_copy = query_ptr_send;
                    let density_ptr_copy = density_ptr_send;
                    let dirty_ptr_copy = dirty_ptr;
                    let prev_groups_ref = &all_group_visited;

                    scope.spawn(async move {
                        let mut chunk_rng = Rng::with_seed(seed);

                        let rect_capacity = dirty_rect.map_or(0, |r| {
                            ((r.max.x - r.min.x + 1) * (r.max.y - r.min.y + 1)) as usize
                        });
                        let capacity = rect_capacity + border_positions.len();

                        let mut positions: Vec<IVec2> = Vec::with_capacity(capacity);
                        if let Some(dirty_rect) = dirty_rect {
                            for y in dirty_rect.min.y..=dirty_rect.max.y {
                                for x in dirty_rect.min.x..=dirty_rect.max.x {
                                    positions.push(IVec2::new(x, y));
                                }
                            }
                        }
                        positions.extend(border_positions);
                        chunk_rng.shuffle(&mut positions);

                        let mut local_visited = HashSet::<Entity>::with_capacity(capacity / 10 + 1);
                        let mut local_dirty = Vec::<IVec2>::new();

                        for position in positions {
                            let Ok(Some(entity)) = (*map_ptr_copy.get()).get_copied(position)
                            else {
                                continue;
                            };

                            if local_visited.contains(&entity)
                                || prev_groups_ref.iter().any(|s| s.contains(&entity))
                            {
                                continue;
                            }

                            let Ok(density) = (*density_ptr_copy.get()).get(entity) else {
                                continue;
                            };

                            if let Ok((
                                mut position,
                                mut rng,
                                mut speed,
                                mut momentum,
                                movement_priority,
                                air_resistance,
                                _resistor,
                            )) = (*query_ptr_copy.get()).get_unchecked(entity)
                            {
                                if speed.current() == 0 {
                                    speed.set_speed(1);
                                }

                                let mut moved = false;
                                let mut obstructed: ObstructedDirections = [false; 9];

                                'speed_loop: for _ in 0..speed.current() {
                                    let momentum_ref = momentum.as_deref().copied();

                                    for (tier, relative_position) in movement_priority
                                        .iter_candidates(&mut rng, momentum_ref.as_ref())
                                        .map(|(tier, pos)| (tier, *pos))
                                    {
                                        let neighbor_position = position.0 + relative_position;
                                        let obstruct_idx = get_direction_index(relative_position);

                                        if obstructed[obstruct_idx] {
                                            continue;
                                        }

                                        let neighbor_entity =
                                            (*map_ptr_copy.get()).get_copied(neighbor_position);

                                        if neighbor_entity.is_err() {
                                            obstructed[obstruct_idx] = true;
                                            continue;
                                        }

                                        if let Ok(Some(neighbor_entity)) = neighbor_entity {
                                            let Ok(neighbor_density) =
                                                (*density_ptr_copy.get()).get(neighbor_entity)
                                            else {
                                                obstructed[obstruct_idx] = true;
                                                continue;
                                            };

                                            if density == neighbor_density {
                                                continue;
                                            }
                                            if density < neighbor_density {
                                                obstructed[obstruct_idx] = true;
                                                continue;
                                            }

                                            if let Ok((
                                                mut neighbor_position,
                                                mut neighbor_rng,
                                                _,
                                                _,
                                                _,
                                                _,
                                                neighbor_resistor,
                                            )) = (*query_ptr_copy.get())
                                                .get_unchecked(neighbor_entity)
                                            {
                                                if let Some(resistor) = neighbor_resistor {
                                                    if neighbor_rng.chance(resistor.0) {
                                                        local_dirty.push(position.0);
                                                        continue;
                                                    }
                                                }
                                                let swap_ok = (*map_ptr_copy.get())
                                                    .swap(neighbor_position.0, position.0);

                                                if swap_ok.is_ok() {
                                                    local_dirty.push(position.0);
                                                    local_dirty.push(neighbor_position.0);

                                                    swap_particle_positions(
                                                        &mut position,
                                                        &mut neighbor_position,
                                                    );

                                                    if let Some(ref mut m) = momentum {
                                                        m.0 = IVec2::ZERO;
                                                    }

                                                    speed.decrement();
                                                    moved = true;
                                                    break 'speed_loop;
                                                }
                                            } else {
                                                obstructed[obstruct_idx] = true;
                                            }
                                        } else {
                                            if let Some(r) = air_resistance.get(tier) {
                                                if r > 0.0 && chunk_rng.chance(r) {
                                                    local_dirty.push(position.0);
                                                    continue;
                                                }
                                            }
                                            let swap_ok = (*map_ptr_copy.get())
                                                .swap(position.0, neighbor_position);

                                            if swap_ok.is_ok() {
                                                local_dirty.push(position.0);
                                                local_dirty.push(neighbor_position);

                                                position.0 = neighbor_position;
                                                if let Some(ref mut m) = momentum {
                                                    m.0 = relative_position;
                                                }
                                                speed.increment();
                                                moved = true;
                                                continue 'speed_loop;
                                            }
                                            obstructed[obstruct_idx] = true;
                                        }
                                    }
                                    if !moved {
                                        break 'speed_loop;
                                    }
                                }

                                if moved {
                                    local_visited.insert(entity);
                                } else {
                                    if let Some(ref mut m) = momentum {
                                        m.0 = IVec2::ZERO;
                                    }
                                    speed.decrement();
                                }
                            }
                        }
                        if !local_visited.is_empty() {
                            (*visited_ptr.get()).lock().unwrap().push(local_visited);
                        }
                        if !local_dirty.is_empty() {
                            (*dirty_ptr_copy.get()).lock().unwrap().extend(local_dirty);
                        }
                    });
                }
            });

            let mut group_set = HashSet::new();
            for chunk_visited in visited_entities_by_chunk.into_inner().unwrap() {
                group_set.extend(chunk_visited);
            }
            all_group_visited.push(group_set);
        }
    }

    for set in all_group_visited {
        movement_state.visited_entities.extend(set);
    }

    for pos in dirty_positions.into_inner().unwrap() {
        let coord = chunk_index.world_to_chunk_coord(pos);
        if let Some(chunk_entity) = chunk_index.get(coord) {
            if let Ok((_, mut dirty_state)) = chunk_query.get_mut(chunk_entity) {
                dirty_state.mark_dirty(pos);
            }
        }
    }
}

#[allow(
    unused_mut,
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    clippy::needless_pass_by_value
)]
pub(super) fn serial_handle_movement_by_chunks(
    mut particle_query: Query<ParticleMovementByChunksQuery>,
    density_query: Query<&Density>,
    mut map: ResMut<ParticleMap>,
    mut movement_state: ResMut<MovementState>,
    mut global_rng: ResMut<GlobalRng>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<(&ChunkRegion, &mut ChunkDirtyState)>,
) {
    movement_state.visited_entities.clear();

    let mut dirty_positions = Vec::<IVec2>::new();

    let seed = global_rng.u64(..);

    let mut chunks_data: Vec<(Option<IRect>, Vec<IVec2>)> = Vec::with_capacity(chunk_index.len());

    for (_coord, chunk_entity) in chunk_index.iter() {
        if let Ok((region, dirty_state)) = chunk_query.get(chunk_entity) {
            if dirty_state.is_dirty() {
                let border_positions = dirty_state.border_strip_positions(region.region());
                chunks_data.push((dirty_state.current, border_positions));
            }
        }
    }

    global_rng.shuffle(&mut chunks_data);

    unsafe {
        let mut chunk_rng = Rng::with_seed(seed);

        for (dirty_rect, border_positions) in chunks_data {
            let rect_capacity = dirty_rect.map_or(0, |r| {
                ((r.max.x - r.min.x + 1) * (r.max.y - r.min.y + 1)) as usize
            });
            let capacity = rect_capacity + border_positions.len();

            let mut positions: Vec<IVec2> = Vec::with_capacity(capacity);
            if let Some(dirty_rect) = dirty_rect {
                for y in dirty_rect.min.y..=dirty_rect.max.y {
                    for x in dirty_rect.min.x..=dirty_rect.max.x {
                        positions.push(IVec2::new(x, y));
                    }
                }
            }
            positions.extend(border_positions);

            chunk_rng.shuffle(&mut positions);

            for position in positions {
                let Ok(Some(entity)) = map.get_copied(position) else {
                    continue;
                };

                if movement_state.visited_entities.contains(&entity) {
                    continue;
                }

                let Ok(density) = density_query.get(entity) else {
                    continue;
                };

                if let Ok((
                    mut position,
                    mut rng,
                    mut speed,
                    mut momentum,
                    movement_priority,
                    air_resistance,
                    _resistor,
                )) = particle_query.get_unchecked(entity)
                {
                    if speed.current() == 0 {
                        speed.set_speed(1);
                    }

                    let mut moved = false;
                    let mut obstructed: ObstructedDirections = [false; 9];

                    'speed_loop: for _ in 0..speed.current() {
                        let momentum_ref = momentum.as_deref().copied();

                        for (tier, relative_position) in movement_priority
                            .iter_candidates(&mut rng, momentum_ref.as_ref())
                            .map(|(tier, pos)| (tier, *pos))
                        {
                            let neighbor_position = position.0 + relative_position;
                            let obstruct_idx = get_direction_index(relative_position);

                            if obstructed[obstruct_idx] {
                                continue;
                            }

                            let neighbor_result = map.get_copied(neighbor_position);
                            if neighbor_result.is_err() {
                                obstructed[obstruct_idx] = true;
                                continue;
                            }

                            if let Ok(Some(neighbor_entity)) = neighbor_result {
                                let Ok(neighbor_density) = density_query.get(neighbor_entity)
                                else {
                                    obstructed[obstruct_idx] = true;
                                    continue;
                                };

                                if density == neighbor_density {
                                    continue;
                                }
                                if density < neighbor_density {
                                    obstructed[obstruct_idx] = true;
                                    continue;
                                }

                                if let Ok((
                                    mut neighbor_position,
                                    mut neighbor_rng,
                                    _,
                                    _,
                                    _,
                                    _,
                                    neighbor_resistor,
                                )) = particle_query.get_unchecked(neighbor_entity)
                                {
                                    if let Some(resistor) = neighbor_resistor {
                                        if neighbor_rng.chance(resistor.0) {
                                            dirty_positions.push(position.0);
                                            continue;
                                        }
                                    }
                                    if map.swap(neighbor_position.0, position.0).is_ok() {
                                        dirty_positions.push(position.0);
                                        dirty_positions.push(neighbor_position.0);

                                        swap_particle_positions(
                                            &mut position,
                                            &mut neighbor_position,
                                        );

                                        if let Some(ref mut m) = momentum {
                                            m.0 = IVec2::ZERO;
                                        }

                                        speed.decrement();
                                        moved = true;
                                        break 'speed_loop;
                                    }
                                } else {
                                    obstructed[obstruct_idx] = true;
                                }
                            } else {
                                if let Some(r) = air_resistance.get(tier) {
                                    if r > 0.0 && chunk_rng.chance(r) {
                                        dirty_positions.push(position.0);
                                        continue;
                                    }
                                }
                                if map.swap(position.0, neighbor_position).is_ok() {
                                    dirty_positions.push(position.0);
                                    dirty_positions.push(neighbor_position);

                                    position.0 = neighbor_position;
                                    if let Some(ref mut m) = momentum {
                                        m.0 = relative_position;
                                    }
                                    speed.increment();
                                    moved = true;
                                    continue 'speed_loop;
                                }
                                obstructed[obstruct_idx] = true;
                            }
                        }
                        if !moved {
                            break 'speed_loop;
                        }
                    }

                    if moved {
                        movement_state.visited_entities.insert(entity);
                    } else {
                        if let Some(ref mut m) = momentum {
                            m.0 = IVec2::ZERO;
                        }
                        speed.decrement();
                    }
                }
            }
        }
    }

    for pos in dirty_positions {
        let coord = chunk_index.world_to_chunk_coord(pos);
        if let Some(chunk_entity) = chunk_index.get(coord) {
            if let Ok((_, mut dirty_state)) = chunk_query.get_mut(chunk_entity) {
                dirty_state.mark_dirty(pos);
            }
        }
    }
}

#[allow(unused_mut, clippy::too_many_lines)]
#[allow(clippy::needless_pass_by_value)]
pub(super) fn handle_movement_by_particles(
    mut particle_query: Query<ParticleMovementByParticlesQuery>,
    density_query: Query<&Density>,
    mut map: ResMut<ParticleMap>,
    mut movement_state: ResMut<MovementState>,
    mut global_rng: ResMut<GlobalRng>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<(&ChunkRegion, &mut ChunkDirtyState)>,
) {
    movement_state.visited_positions.clear();

    let mut dirty_positions = Vec::<IVec2>::new();

    unsafe {
        particle_query.iter_unsafe().for_each(
            |(
                entity,
                mut position,
                mut rng,
                mut speed,
                mut momentum,
                movement_priority,
                air_resistance,
                _resistor,
            )| {
                if speed.current() == 0 {
                    speed.set_speed(1);
                }

                let pos = position.0;
                let my_chunk_coord = chunk_index.world_to_chunk_coord(pos);

                let in_dirty_region = chunk_index
                    .get(my_chunk_coord)
                    .and_then(|entity| chunk_query.get(entity).ok())
                    .is_some_and(|(region, dirty_state)| {
                        dirty_state.current.is_some_and(|rect| rect.contains(pos))
                            || dirty_state.is_in_border_strip(pos, region.region())
                    });

                if !in_dirty_region {
                    return;
                }

                let Ok(density) = density_query.get(entity) else {
                    return;
                };

                let mut moved = false;
                let mut obstructed: ObstructedDirections = [false; 9];

                'speed_loop: for _ in 0..speed.current() {
                    let momentum_ref = momentum.as_deref().copied();

                    for (tier, relative_position) in movement_priority
                        .iter_candidates(&mut rng, momentum_ref.as_ref())
                        .map(|(tier, pos)| (tier, *pos))
                    {
                        let neighbor_position = position.0 + relative_position;
                        let obstruct_idx = get_direction_index(relative_position);

                        if movement_state
                            .visited_positions
                            .contains(&neighbor_position)
                            || obstructed[obstruct_idx]
                        {
                            continue;
                        }

                        let neighbor_result = map.get_copied(neighbor_position);
                        if neighbor_result.is_err() {
                            obstructed[obstruct_idx] = true;
                            continue;
                        }

                        if let Ok(Some(neighbor_entity)) = neighbor_result {
                            let Ok(neighbor_density) = density_query.get(neighbor_entity) else {
                                obstructed[obstruct_idx] = true;
                                continue;
                            };

                            if density == neighbor_density {
                                continue;
                            }
                            if density < neighbor_density {
                                obstructed[obstruct_idx] = true;
                                continue;
                            }

                            if let Ok((
                                _,
                                mut neighbor_position,
                                mut neighbor_rng,
                                _,
                                _,
                                _,
                                _,
                                neighbor_resistor,
                            )) = particle_query.get_unchecked(neighbor_entity)
                            {
                                if let Some(resistor) = neighbor_resistor {
                                    if neighbor_rng.chance(resistor.0) {
                                        dirty_positions.push(position.0);
                                        continue;
                                    }
                                }
                                if map.swap(neighbor_position.0, position.0).is_ok() {
                                    {
                                        dirty_positions.push(position.0);
                                        dirty_positions.push(neighbor_position.0);
                                    }

                                    swap_particle_positions(&mut position, &mut neighbor_position);
                                    if let Some(ref mut momentum) = momentum {
                                        momentum.0 = IVec2::ZERO;
                                    }
                                    speed.decrement();
                                    moved = true;
                                    break 'speed_loop;
                                }
                            } else {
                                obstructed[obstruct_idx] = true;
                            }
                        } else {
                            if let Some(r) = air_resistance.get(tier) {
                                if r > 0.0 && global_rng.chance(r) {
                                    dirty_positions.push(position.0);
                                    continue;
                                }
                            }
                            let old_position = position.0;
                            if map.swap(position.0, neighbor_position).is_ok() {
                                {
                                    dirty_positions.push(old_position);
                                    dirty_positions.push(neighbor_position);
                                }

                                position.0 = neighbor_position;
                                if let Some(ref mut momentum) = momentum {
                                    momentum.0 = relative_position;
                                }
                                speed.increment();
                                moved = true;
                                continue 'speed_loop;
                            }
                            obstructed[obstruct_idx] = true;
                        }
                    }
                }

                if moved {
                    movement_state.visited_positions.insert(position.0);
                } else {
                    if let Some(ref mut momentum) = momentum {
                        momentum.0 = IVec2::ZERO;
                    }
                    speed.decrement();
                }
            },
        );
    }

    for pos in dirty_positions {
        let coord = chunk_index.world_to_chunk_coord(pos);
        if let Some(chunk_entity) = chunk_index.get(coord) {
            if let Ok((_, mut dirty_state)) = chunk_query.get_mut(chunk_entity) {
                dirty_state.mark_dirty(pos);
            }
        }
    }
}

#[inline(always)]
const fn swap_particle_positions(
    first_position: &mut GridPosition,
    second_position: &mut GridPosition,
) {
    mem::swap(&mut first_position.0, &mut second_position.0);
}
