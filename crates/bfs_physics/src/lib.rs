#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(
    clippy::nursery,
    clippy::pedantic,
    nonstandard_style,
    rustdoc::broken_intra_doc_links,
    missing_docs
)]
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]
//! Integrates avian2d physics with the Falling Sand simulation.
use avian2d::math::Vector;
pub use avian2d::prelude::*;

use bevy::prelude::*;
use bfs_core::ParticlePosition;
use bfs_movement::{MovableSolid, Moved, Solid, Wall};

/// Provides the constructs and systems necessary to integrate avian2d in the Falling Sand simulation.
pub struct FallingSandPhysicsPlugin {
    /// The value for
    /// [`PhysicsLengthUnit`](https://docs.rs/avian2d/latest/avian2d/dynamics/solver/struct.PhysicsLengthUnit.html)
    /// in the avian2d crate.
    pub length_unit: f32,
}

impl Plugin for FallingSandPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default().with_length_unit(self.length_unit));
        app.init_resource::<WallPerimeterPositions>();
        app.init_resource::<WallTerrainColliders>();
        app.init_resource::<MovableSolidMeshData>();
        app.init_resource::<MovableSolidTerrainColliders>();
        app.init_resource::<SolidMeshData>();
        app.init_resource::<SolidTerrainColliders>();
        app.add_systems(Update, map_wall_particles.run_if(condition_walls_changed));
        app.add_systems(Update, spawn_wall_terrain_colliders);
        app.add_systems(
            Update,
            map_movable_solid_particles.run_if(condition_movable_solids_changed),
        );
        app.add_systems(Update, spawn_movable_solid_terrain_colliders);
        app.add_systems(Update, map_solid_particles.run_if(condition_solids_changed));
        app.add_systems(Update, spawn_solid_terrain_colliders);
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
struct WallPerimeterPositions((Vec<Vec<Vec2>>, Vec<Vec<[u32; 2]>>));

#[derive(Resource, Default, Debug)]
struct WallTerrainColliders(Vec<Entity>);

#[derive(Resource, Default, Debug)]
struct MovableSolidMeshData {
    vertices: Vec<Vec<Vector>>,
    indices: Vec<Vec<[u32; 3]>>,
}

#[derive(Resource, Default, Debug)]
struct MovableSolidTerrainColliders(Vec<Entity>);

#[derive(Resource, Default, Debug)]
struct SolidMeshData {
    vertices: Vec<Vec<Vector>>,
    indices: Vec<Vec<[u32; 3]>>,
}

#[derive(Resource, Default, Debug)]
struct SolidTerrainColliders(Vec<Entity>);

#[allow(clippy::needless_pass_by_value)]
fn spawn_wall_terrain_colliders(
    mut commands: Commands,
    mut colliders: ResMut<WallTerrainColliders>,
    perimeter_positions: Res<WallPerimeterPositions>,
) {
    if !perimeter_positions.is_changed() {
        return;
    }

    for entity in colliders.0.drain(..) {
        commands.entity(entity).despawn();
    }

    for (i, vertices) in perimeter_positions.0.0.iter().enumerate() {
        let entity = commands
            .spawn((
                RigidBody::Static,
                Collider::polyline(vertices.clone(), Some(perimeter_positions.0.1[i].clone())),
            ))
            .id();

        colliders.0.push(entity);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn spawn_movable_solid_terrain_colliders(
    mut commands: Commands,
    mut colliders: ResMut<MovableSolidTerrainColliders>,
    mesh_data: Res<MovableSolidMeshData>,
) {
    if !mesh_data.is_changed() {
        return;
    }

    for entity in colliders.0.drain(..) {
        commands.entity(entity).despawn();
    }

    for (vertices, indices) in mesh_data.vertices.iter().zip(&mesh_data.indices) {
        if indices.is_empty() || vertices.is_empty() {
            warn!("Skipping empty trimesh collider (no vertices or triangles)");
            continue;
        }

        let entity = commands
            .spawn((
                RigidBody::Static,
                Collider::trimesh(vertices.clone(), indices.clone()),
            ))
            .id();

        colliders.0.push(entity);
    }
}

fn spawn_solid_terrain_colliders(
    mut commands: Commands,
    mut colliders: ResMut<SolidTerrainColliders>,
    mesh_data: Res<SolidMeshData>,
) {
    if !mesh_data.is_changed() {
        return;
    }

    for entity in colliders.0.drain(..) {
        commands.entity(entity).despawn();
    }

    for (vertices, indices) in mesh_data.vertices.iter().zip(&mesh_data.indices) {
        if indices.is_empty() || vertices.is_empty() {
            warn!("Skipping empty trimesh collider (no vertices or triangles)");
            continue;
        }

        let entity = commands
            .spawn((
                RigidBody::Static,
                Collider::trimesh(vertices.clone(), indices.clone()),
            ))
            .id();

        colliders.0.push(entity);
    }
}

fn map_wall_particles(
    wall_query: Query<&ParticlePosition, With<Wall>>,
    mut wall_positions: ResMut<WallPerimeterPositions>,
) {
    let positions: Vec<ParticlePosition> = wall_query.iter().copied().collect();

    if positions.is_empty() {
        wall_positions.0 = (Vec::new(), Vec::new());
        return;
    }

    let min = positions
        .iter()
        .fold(IVec2::new(i32::MAX, i32::MAX), |min, c| min.min(c.0));
    let max = positions
        .iter()
        .fold(IVec2::new(i32::MIN, i32::MIN), |max, c| max.max(c.0));

    let mut grid = Grid::new(min, max);
    for position in &positions {
        grid.set(position.0);
    }

    let edges = extract_perimeter_edges(&grid);

    let mut components = Vec::new();
    let mut perimeters = Vec::new();

    let mut vertices = Vec::new();
    for edge in &edges {
        vertices.push(edge[0]);
        vertices.push(edge[1]);
    }

    let indices: Vec<[u32; 2]> = (0..vertices.len() as u32)
        .step_by(2)
        .map(|i| [i, i + 1])
        .collect();

    components.push(vertices);
    perimeters.push(indices);

    wall_positions.0 = (components, perimeters);
}

fn map_movable_solid_particles(
    movable_solid_query: Query<(&ParticlePosition, &Moved), With<MovableSolid>>,
    mut mesh_data: ResMut<MovableSolidMeshData>,
) {
    use earcutr::earcut;
    use std::collections::{HashSet, VecDeque};

    let positions: Vec<IVec2> = movable_solid_query
        .iter()
        .filter_map(|(c, m)| if !m.0 { Some(c.0) } else { None })
        .collect();

    if positions.is_empty() {
        mesh_data.vertices.clear();
        mesh_data.indices.clear();
        return;
    }

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

    mesh_data.vertices = all_vertices;
    mesh_data.indices = all_indices;
}

fn map_solid_particles(
    movable_solid_query: Query<(&ParticlePosition, &Moved), With<Solid>>,
    mut mesh_data: ResMut<SolidMeshData>,
) {
    use earcutr::earcut;
    use std::collections::{HashSet, VecDeque};

    let positions: Vec<IVec2> = movable_solid_query
        .iter()
        .filter_map(|(c, m)| if !m.0 { Some(c.0) } else { None })
        .collect();

    if positions.is_empty() {
        mesh_data.vertices.clear();
        mesh_data.indices.clear();
        return;
    }

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

    mesh_data.vertices = all_vertices;
    mesh_data.indices = all_indices;
}

#[allow(clippy::needless_pass_by_value)]
fn condition_walls_changed(
    query: Query<Entity, Changed<Wall>>,
    removed: RemovedComponents<Wall>,
) -> bool {
    if !query.is_empty() || !removed.is_empty() {
        return true;
    }
    false
}

#[allow(clippy::needless_pass_by_value)]
fn condition_movable_solids_changed(
    query: Query<&MovableSolid, Changed<ParticlePosition>>,
    removed: RemovedComponents<MovableSolid>,
) -> bool {
    !query.is_empty() || !removed.is_empty()
}

#[allow(clippy::needless_pass_by_value)]
fn condition_solids_changed(
    query: Query<&Solid, Changed<ParticlePosition>>,
    removed: RemovedComponents<Solid>,
) -> bool {
    !query.is_empty() || !removed.is_empty()
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
