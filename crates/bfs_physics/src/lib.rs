#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(
    clippy::nursery,
    clippy::pedantic,
    nonstandard_style,
    rustdoc::broken_intra_doc_links,
    missing_docs
)]
#![allow(
    clippy::default_trait_access,
    clippy::module_name_repetitions,
    clippy::inline_always,
    clippy::cast_possible_wrap
)]
//! Integrates avian2d physics with the Falling Sand simulation.
use avian2d::math::Vector;
pub use avian2d::prelude::*;

use bevy::prelude::*;
use bfs_core::{ParticleMap, ParticlePosition, ParticleSimulationSet};
use bfs_movement::{MovableSolid, Moved, Solid, Wall};

/// Provides the constructs and systems necessary to integrate avian2d in the Falling Sand simulation.
pub struct FallingSandPhysicsPlugin {
    /// The value for
    /// [`PhysicsLengthUnit`](https://docs.rs/avian2d/latest/avian2d/dynamics/solver/struct.PhysicsLengthUnit.html)
    /// in the avian2d crate.
    pub length_unit: f32,
    /// The value for [`GravityScale`](https://docs.rs/avian2d/latest/avian2d/dynamics/rigid_body/struct.GravityScale.html)
    /// in the avian2d crate.
    pub rigid_body_gravity: Vec2,
}

impl Plugin for FallingSandPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default().with_length_unit(self.length_unit))
            .insert_resource(Gravity(self.rigid_body_gravity))
            .init_resource::<WallMeshData>()
            .init_resource::<WallTerrainColliders>()
            .init_resource::<MovableSolidMeshData>()
            .init_resource::<MovableSolidTerrainColliders>()
            .init_resource::<SolidMeshData>()
            .init_resource::<SolidTerrainColliders>()
            .add_systems(
                Update,
                recalculate_and_spawn_static_bodies_for_dirty_chunks.in_set(ParticleSimulationSet),
            );
    }
}

#[derive(Debug)]
struct Grid {
    min: IVec2,
    size: IVec2,
    data: Vec<bool>,
}

impl Grid {
    fn new(min: IVec2, max: IVec2) -> Self {
        let size = max - min + IVec2::ONE;
        let data = vec![false; (size.x * size.y) as usize];
        Self { min, size, data }
    }

    fn index(&self, position: IVec2) -> usize {
        let local = position - self.min;
        (local.y * self.size.x + local.x) as usize
    }

    fn set(&mut self, position: IVec2) {
        let idx = self.index(position);
        self.data[idx] = true;
    }

    fn get(&self, position: IVec2) -> bool {
        if position.x < self.min.x
            || position.y < self.min.y
            || position.x > self.min.x + self.size.x - 1
            || position.y > self.min.y + self.size.y - 1
        {
            return false;
        }
        let idx = self.index(position);
        self.data[idx]
    }

    fn iter_occupied(&self) -> impl Iterator<Item = IVec2> + '_ {
        self.data.iter().enumerate().filter_map(move |(i, &b)| {
            if b {
                let x = i as i32 % self.size.x;
                let y = i as i32 / self.size.x;
                Some(self.min + IVec2::new(x, y))
            } else {
                None
            }
        })
    }
}

#[derive(Resource, Default, Debug)]
struct WallMeshData {
    chunks: bevy::platform::collections::HashMap<usize, (Vec<Vec<Vector>>, Vec<Vec<[u32; 3]>>)>,
}

#[derive(Resource, Default, Debug)]
struct WallTerrainColliders(bevy::platform::collections::HashMap<usize, Vec<Entity>>);

#[derive(Resource, Default, Debug)]
struct MovableSolidMeshData {
    chunks: bevy::platform::collections::HashMap<usize, (Vec<Vec<Vector>>, Vec<Vec<[u32; 3]>>)>,
}

#[derive(Resource, Default, Debug)]
struct MovableSolidTerrainColliders(bevy::platform::collections::HashMap<usize, Vec<Entity>>);

#[derive(Resource, Default, Debug)]
struct SolidMeshData {
    chunks: bevy::platform::collections::HashMap<usize, (Vec<Vec<Vector>>, Vec<Vec<[u32; 3]>>)>,
}

#[derive(Resource, Default, Debug)]
struct SolidTerrainColliders(bevy::platform::collections::HashMap<usize, Vec<Entity>>);

fn recalculate_and_spawn_static_bodies_for_dirty_chunks(
    mut commands: Commands,
    wall_query: Query<&ParticlePosition, With<Wall>>,
    movable_solid_query: Query<(&ParticlePosition, &Moved), With<MovableSolid>>,
    solid_query: Query<(&ParticlePosition, &Moved), With<Solid>>,
    mut wall_mesh_data: ResMut<WallMeshData>,
    mut movable_solid_mesh_data: ResMut<MovableSolidMeshData>,
    mut solid_mesh_data: ResMut<SolidMeshData>,
    mut wall_colliders: ResMut<WallTerrainColliders>,
    mut movable_solid_colliders: ResMut<MovableSolidTerrainColliders>,
    mut solid_colliders: ResMut<SolidTerrainColliders>,
    particle_map: Res<ParticleMap>,
) {
    // Find all chunks that have dirty rects (indicating movement)
    let dirty_chunks: Vec<usize> = particle_map
        .iter_chunks()
        .enumerate()
        .filter_map(|(chunk_index, chunk)| {
            if chunk.dirty_rect().is_some() {
                Some(chunk_index)
            } else {
                None
            }
        })
        .collect();

    if dirty_chunks.is_empty() {
        return;
    }

    // Clear mesh data and colliders for dirty chunks only
    for &chunk_index in &dirty_chunks {
        // Clear mesh data
        wall_mesh_data.chunks.remove(&chunk_index);
        movable_solid_mesh_data.chunks.remove(&chunk_index);
        solid_mesh_data.chunks.remove(&chunk_index);

        // Clear and despawn existing colliders for this specific chunk
        if let Some(entities) = wall_colliders.0.remove(&chunk_index) {
            for entity in entities {
                commands.entity(entity).despawn();
            }
        }
        if let Some(entities) = movable_solid_colliders.0.remove(&chunk_index) {
            for entity in entities {
                commands.entity(entity).despawn();
            }
        }
        if let Some(entities) = solid_colliders.0.remove(&chunk_index) {
            for entity in entities {
                commands.entity(entity).despawn();
            }
        }
    }

    // Rebuild data for all dirty chunks
    for &chunk_index in &dirty_chunks {
        if let Some(chunk) = particle_map.iter_chunks().nth(chunk_index) {
            // Process walls
            let wall_positions: Vec<IVec2> = chunk
                .iter()
                .filter_map(|(pos, entity)| {
                    if wall_query.contains(*entity) {
                        Some(*pos)
                    } else {
                        None
                    }
                })
                .collect();

            if !wall_positions.is_empty() {
                let (vertices_list, indices_list) = process_solid_positions(wall_positions);
                if !vertices_list.is_empty() {
                    wall_mesh_data
                        .chunks
                        .insert(chunk_index, (vertices_list.clone(), indices_list.clone()));

                    // Spawn colliders for this chunk
                    let mut chunk_entities = Vec::new();
                    for (vertices, indices) in vertices_list.iter().zip(&indices_list) {
                        if !indices.is_empty() && !vertices.is_empty() {
                            let entity = commands
                                .spawn((
                                    RigidBody::Static,
                                    Collider::trimesh(vertices.clone(), indices.clone()),
                                ))
                                .id();
                            chunk_entities.push(entity);
                        }
                    }
                    if !chunk_entities.is_empty() {
                        wall_colliders.0.insert(chunk_index, chunk_entities);
                    }
                }
            }

            // Process movable solids
            let movable_solid_positions: Vec<IVec2> = chunk
                .iter()
                .filter_map(|(pos, entity)| {
                    if let Ok((_, moved)) = movable_solid_query.get(*entity) {
                        if !moved.0 { Some(*pos) } else { None }
                    } else {
                        None
                    }
                })
                .collect();

            if !movable_solid_positions.is_empty() {
                let (vertices_list, indices_list) =
                    process_solid_positions(movable_solid_positions);
                if !vertices_list.is_empty() {
                    movable_solid_mesh_data
                        .chunks
                        .insert(chunk_index, (vertices_list.clone(), indices_list.clone()));

                    // Spawn colliders for this chunk
                    let mut chunk_entities = Vec::new();
                    for (vertices, indices) in vertices_list.iter().zip(&indices_list) {
                        if !indices.is_empty() && !vertices.is_empty() {
                            let entity = commands
                                .spawn((
                                    RigidBody::Static,
                                    Collider::trimesh(vertices.clone(), indices.clone()),
                                ))
                                .id();
                            chunk_entities.push(entity);
                        }
                    }
                    if !chunk_entities.is_empty() {
                        movable_solid_colliders
                            .0
                            .insert(chunk_index, chunk_entities);
                    }
                }
            }

            // Process solids
            let solid_positions: Vec<IVec2> = chunk
                .iter()
                .filter_map(|(pos, entity)| {
                    if let Ok((_, moved)) = solid_query.get(*entity) {
                        if !moved.0 { Some(*pos) } else { None }
                    } else {
                        None
                    }
                })
                .collect();

            if !solid_positions.is_empty() {
                let (vertices_list, indices_list) = process_solid_positions(solid_positions);
                if !vertices_list.is_empty() {
                    solid_mesh_data
                        .chunks
                        .insert(chunk_index, (vertices_list.clone(), indices_list.clone()));

                    // Spawn colliders for this chunk
                    let mut chunk_entities = Vec::new();
                    for (vertices, indices) in vertices_list.iter().zip(&indices_list) {
                        if !indices.is_empty() && !vertices.is_empty() {
                            let entity = commands
                                .spawn((
                                    RigidBody::Static,
                                    Collider::trimesh(vertices.clone(), indices.clone()),
                                ))
                                .id();
                            chunk_entities.push(entity);
                        }
                    }
                    if !chunk_entities.is_empty() {
                        solid_colliders.0.insert(chunk_index, chunk_entities);
                    }
                }
            }
        }
    }
}

fn process_solid_positions(positions: Vec<IVec2>) -> (Vec<Vec<Vector>>, Vec<Vec<[u32; 3]>>) {
    use earcutr::earcut;
    use std::collections::{HashSet, VecDeque};

    let mut unvisited: HashSet<IVec2> = positions.iter().copied().collect();
    let mut all_vertices = Vec::new();
    let mut all_indices = Vec::new();

    while let Some(&start) = unvisited.iter().next() {
        let mut group = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        unvisited.remove(&start);

        while let Some(current) = queue.pop_front() {
            group.push(current);

            for dir in [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y] {
                let neighbor = current + dir;
                if unvisited.remove(&neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }

        let min = group
            .iter()
            .copied()
            .fold(IVec2::splat(i32::MAX), bevy::prelude::IVec2::min);
        let max = group
            .iter()
            .copied()
            .fold(IVec2::splat(i32::MIN), bevy::prelude::IVec2::max);
        let mut grid = Grid::new(min, max);
        for position in &group {
            grid.set(*position);
        }

        let loop_vertices = extract_ordered_perimeter_loop(&grid);
        if loop_vertices.len() < 3 {
            continue;
        }

        let flattened: Vec<f64> = loop_vertices
            .iter()
            .flat_map(|v| vec![v.x as f64, v.y as f64])
            .collect();

        if let Ok(indices_raw) = earcut(&flattened, &[], 2) {
            let triangle_indices: Vec<[u32; 3]> = indices_raw
                .chunks(3)
                .map(|c| [c[0] as u32, c[1] as u32, c[2] as u32])
                .collect();

            let vertices = loop_vertices
                .into_iter()
                .map(|v| Vector::new(v.x, v.y))
                .collect();

            all_vertices.push(vertices);
            all_indices.push(triangle_indices);
        }
    }

    (all_vertices, all_indices)
}

fn extract_ordered_perimeter_loop(grid: &Grid) -> Vec<Vec2> {
    let edges = extract_perimeter_edges(grid);
    if edges.is_empty() {
        return Vec::new();
    }

    let mut ordered = Vec::new();
    let mut remaining = edges;

    let [current_start, mut current_end] = remaining.swap_remove(0);
    ordered.push(current_start);
    ordered.push(current_end);

    while !remaining.is_empty() {
        let mut found = false;
        for i in 0..remaining.len() {
            let [start, end] = remaining[i];
            if start == current_end {
                ordered.push(end);
                current_end = end;
                remaining.swap_remove(i);
                found = true;
                break;
            } else if end == current_end {
                ordered.push(start);
                current_end = start;
                remaining.swap_remove(i);
                found = true;
                break;
            }
        }

        if !found {
            warn!("Could not form closed perimeter loop; perimeter might be disjoint or broken.");
            break;
        }

        if ordered[0] == current_end {
            break;
        }
    }

    if ordered.len() > 1 && ordered[0] == *ordered.last().unwrap() {
        ordered.pop();
    }

    ordered
}

fn extract_perimeter_edges(grid: &Grid) -> Vec<[Vec2; 2]> {
    let mut edges = Vec::new();

    let directions = [
        (IVec2::new(1, 0), Vec2::new(0.5, 0.5), Vec2::new(0.5, -0.5)),
        (
            IVec2::new(-1, 0),
            Vec2::new(-0.5, -0.5),
            Vec2::new(-0.5, 0.5),
        ),
        (IVec2::new(0, 1), Vec2::new(-0.5, 0.5), Vec2::new(0.5, 0.5)),
        (
            IVec2::new(0, -1),
            Vec2::new(0.5, -0.5),
            Vec2::new(-0.5, -0.5),
        ),
    ];

    for position in grid.iter_occupied() {
        let base = position.as_vec2();
        for (offset, v0, v1) in directions {
            if !grid.get(position + offset) {
                edges.push([base + v0, base + v1]);
            }
        }
    }

    edges
}
