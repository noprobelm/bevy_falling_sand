use avian2d::math::Vector;
pub use avian2d::prelude::*;

use bevy::prelude::*;
use bfs_core::{ChunkMap, Coordinates, Particle, ParticleSimulationSet};
use bfs_movement::{Liquid, MovableSolid, Moved, Solid, Wall};

pub struct FallingSandPhysicsPlugin {
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
        app.add_systems(
            Update,
            float_dynamic_rigid_bodies.after(ParticleSimulationSet),
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

    fn index(&self, coord: IVec2) -> usize {
        let local = coord - self.min;
        (local.y * self.size.x + local.x) as usize
    }

    fn set(&mut self, coord: IVec2) {
        let idx = self.index(coord);
        self.data[idx] = true;
    }

    fn get(&self, coord: IVec2) -> bool {
        if coord.x < self.min.x
            || coord.y < self.min.y
            || coord.x > self.min.x + self.size.x - 1
            || coord.y > self.min.y + self.size.y - 1
        {
            return false;
        }
        let idx = self.index(coord);
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
    wall_query: Query<&Coordinates, With<Wall>>,
    mut wall_positions: ResMut<WallPerimeterPositions>,
) {
    let coords: Vec<Coordinates> = wall_query.iter().copied().collect();

    if coords.is_empty() {
        wall_positions.0 = (Vec::new(), Vec::new());
        return;
    }

    let min = coords
        .iter()
        .fold(IVec2::new(i32::MAX, i32::MAX), |min, c| min.min(c.0));
    let max = coords
        .iter()
        .fold(IVec2::new(i32::MIN, i32::MIN), |max, c| max.max(c.0));

    let mut grid = Grid::new(min, max);
    for coord in &coords {
        grid.set(coord.0);
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
    movable_solid_query: Query<(&Coordinates, &Moved), With<MovableSolid>>,
    mut mesh_data: ResMut<MovableSolidMeshData>,
) {
    use earcutr::earcut;
    use std::collections::{HashSet, VecDeque};

    let coords: Vec<IVec2> = movable_solid_query
        .iter()
        .filter_map(|(c, m)| if !m.0 { Some(c.0) } else { None })
        .collect();

    if coords.is_empty() {
        mesh_data.vertices.clear();
        mesh_data.indices.clear();
        return;
    }

    let mut unvisited: HashSet<IVec2> = coords.iter().copied().collect();
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
            .fold(IVec2::splat(i32::MAX), |a, b| a.min(b));
        let max = group
            .iter()
            .copied()
            .fold(IVec2::splat(i32::MIN), |a, b| a.max(b));
        let mut grid = Grid::new(min, max);
        for coord in &group {
            grid.set(*coord);
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
    movable_solid_query: Query<(&Coordinates, &Moved), With<Solid>>,
    mut mesh_data: ResMut<SolidMeshData>,
) {
    use earcutr::earcut;
    use std::collections::{HashSet, VecDeque};

    let coords: Vec<IVec2> = movable_solid_query
        .iter()
        .filter_map(|(c, m)| if !m.0 { Some(c.0) } else { None })
        .collect();

    if coords.is_empty() {
        mesh_data.vertices.clear();
        mesh_data.indices.clear();
        return;
    }

    let mut unvisited: HashSet<IVec2> = coords.iter().copied().collect();
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
            .fold(IVec2::splat(i32::MAX), |a, b| a.min(b));
        let max = group
            .iter()
            .copied()
            .fold(IVec2::splat(i32::MIN), |a, b| a.max(b));
        let mut grid = Grid::new(min, max);
        for coord in &group {
            grid.set(*coord);
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

fn condition_walls_changed(
    query: Query<Entity, Changed<Wall>>,
    removed: RemovedComponents<Wall>,
) -> bool {
    if !query.is_empty() || !removed.is_empty() {
        return true;
    }
    false
}

fn condition_movable_solids_changed(
    query: Query<&MovableSolid, Changed<Coordinates>>,
    removed: RemovedComponents<MovableSolid>,
) -> bool {
    !query.is_empty() || !removed.is_empty()
}

fn condition_solids_changed(
    query: Query<&Solid, Changed<Coordinates>>,
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

    for coord in grid.iter_occupied() {
        let base = coord.as_vec2();
        for (offset, v0, v1) in directions {
            if !grid.get(coord + offset) {
                edges.push([base + v0, base + v1]);
            }
        }
    }

    edges
}

fn float_dynamic_rigid_bodies(
    mut rigid_body_query: Query<(
        &RigidBody,
        &Transform,
        &mut GravityScale,
        &mut LinearVelocity,
    )>,
    liquid_query: Query<&Particle, With<Liquid>>,
    chunk_map: Res<ChunkMap>,
) {
    let damping_factor = 0.95;
    rigid_body_query.iter_mut().for_each(
        |(rigid_body, transform, mut gravity_scale, mut linear_velocity)| {
            if rigid_body == &RigidBody::Dynamic {
                if let Some(entity) = chunk_map.get(&IVec2::new(
                    transform.translation.x as i32,
                    transform.translation.y as i32,
                )) {
                    if liquid_query.contains(*entity) {
                        linear_velocity.y *= damping_factor;
                        if linear_velocity.y.abs() < 0.001 {
                            linear_velocity.y = 0.0;
                        }
                        gravity_scale.0 = -1.0;
                    }
                } else {
                    gravity_scale.0 = 1.0;
                }
            }
        },
    );
}
