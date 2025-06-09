pub use avian2d::prelude::*;

use bevy::prelude::*;
use bfs_core::Coordinates;
use bfs_movement::Wall;

pub struct FallingSandPhysicsPlugin {
    pub length_unit: f32,
}

impl Plugin for FallingSandPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default().with_length_unit(self.length_unit));
        app.init_resource::<PerimeterPositions>();
        app.init_resource::<TerrainColliders>();
        app.add_systems(Update, map_wall_particles.run_if(condition_walls_changed));
        app.add_systems(Update, spawn_terrain_colliders);
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
struct PerimeterPositions((Vec<Vec<Vec2>>, Vec<Vec<[u32; 2]>>));

#[derive(Resource, Default, Debug)]
struct TerrainColliders(Vec<Entity>);

fn spawn_terrain_colliders(
    mut commands: Commands,
    mut colliders: ResMut<TerrainColliders>,
    perimeter_positions: Res<PerimeterPositions>,
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

fn condition_walls_changed(
    query: Query<Entity, Changed<Wall>>,
    removed: RemovedComponents<Wall>,
) -> bool {
    if !query.is_empty() || !removed.is_empty() {
        return true;
    }
    false
}

fn map_wall_particles(
    wall_query: Query<&Coordinates, With<Wall>>,
    mut wall_positions: ResMut<PerimeterPositions>,
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
