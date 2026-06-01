//! Geometry utilities for occupied particle-grid cells.
//!
//! These helpers are independent of the simulation runtime. They are useful when you need to
//! group connected grid positions, find cell outlines, simplify outlines, or build a triangle mesh
//! from occupied cells.

use std::collections::VecDeque;

#[cfg(feature = "physics")]
use bevy::log::warn;
use bevy::{platform::collections::HashSet, prelude::*};

/// A triangle mesh generated from occupied particle-grid cells.
#[derive(Clone, Debug, Default)]
pub struct GridCellMesh {
    /// Mesh vertices in the same coordinate space as the input grid cells.
    pub vertices: Vec<Vec2>,
    /// Triangle indices into [`Self::vertices`].
    pub indices: Vec<[u32; 3]>,
}

/// Group unordered, disconnected positions into blobs of cardinally connected positions.
///
/// Duplicate position inputs are ignored.
///
/// ```
/// use bevy::prelude::IVec2;
/// use bevy_falling_sand::utils::connected_components;
///
/// let positions = [
///     IVec2::new(0, 0),
///     IVec2::new(1, 0),
///     IVec2::new(5, 5),
/// ];
/// let components = connected_components(positions);
/// assert_eq!(components.len(), 2);
/// ```
#[must_use]
pub fn connected_components<I>(positions: I) -> Vec<Vec<IVec2>>
where
    I: IntoIterator<Item = IVec2>,
{
    let mut remaining: HashSet<IVec2> = positions.into_iter().collect();
    let mut components = Vec::new();

    while let Some(&start) = remaining.iter().next() {
        remaining.remove(&start);
        let mut component = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);

        while let Some(pos) = queue.pop_front() {
            component.push(pos);
            for offset in [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y] {
                let neighbor = pos + offset;
                if remaining.remove(&neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }

        components.push(component);
    }

    components
}

/// Return the cells in `component` that have at least one missing cardinal neighbor.
///
/// `component` is treated as the universe of filled cells; any cell outside `component` counts as
/// empty for the neighbor test. For a connected blob produced by [`connected_components`], this
/// yields the blob's outline cells.
#[must_use]
pub fn perimeter_positions(component: &[IVec2]) -> Vec<IVec2> {
    let set: HashSet<IVec2> = component.iter().copied().collect();
    component
        .iter()
        .copied()
        .filter(|&pos| {
            [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y]
                .iter()
                .any(|&offset| !set.contains(&(pos + offset)))
        })
        .collect()
}

/// Generate a triangle mesh from occupied particle-grid cells.
///
/// The input positions are interpreted as occupied unit cells whose centers are at
/// `position + 0.5`. Disconnected regions are emitted into one combined mesh.
///
/// `epsilon` controls Douglas-Peucker perimeter simplification. Lower values preserve more
/// boundary detail; higher values reduce vertex count more aggressively.
#[cfg(feature = "physics")]
#[must_use]
pub fn mesh_from_grid_cells<I>(positions: I, epsilon: f32) -> GridCellMesh
where
    I: IntoIterator<Item = IVec2>,
{
    let positions: Vec<IVec2> = positions.into_iter().collect();
    let Some(min) = positions.iter().copied().reduce(IVec2::min) else {
        return GridCellMesh::default();
    };
    let Some(max) = positions.iter().copied().reduce(IVec2::max) else {
        return GridCellMesh::default();
    };

    let size = max - min + IVec2::ONE;
    let width = size.x as usize;
    let height = size.y as usize;
    let bitmap_size = width.max(height);
    let mut bitmap = vec![false; bitmap_size * bitmap_size];

    for position in positions {
        let local = position - min;
        bitmap[local.y as usize * bitmap_size + local.x as usize] = true;
    }

    mesh_from_bitmap(&bitmap, min, bitmap_size, epsilon)
}

/// Generate a triangle mesh from a square row-major occupancy bitmap.
///
/// `origin` is the world-space position represented by bitmap index `0`, and `bitmap_size` is the
/// side length of the square bitmap.
#[cfg(feature = "physics")]
#[must_use]
pub fn mesh_from_bitmap(
    bitmap: &[bool],
    origin: IVec2,
    bitmap_size: usize,
    epsilon: f32,
) -> GridCellMesh {
    let components = mesh_components_from_bitmap(bitmap, origin, bitmap_size, epsilon);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for component in components {
        let offset = vertices.len() as u32;
        vertices.extend(component.vertices);
        indices.extend(
            component
                .indices
                .into_iter()
                .map(|[a, b, c]| [a + offset, b + offset, c + offset]),
        );
    }

    GridCellMesh { vertices, indices }
}

/// Generate one triangle mesh per connected component in a square row-major occupancy bitmap.
#[cfg(feature = "physics")]
#[must_use]
pub fn mesh_components_from_bitmap(
    bitmap: &[bool],
    origin: IVec2,
    bitmap_size: usize,
    epsilon: f32,
) -> Vec<GridCellMesh> {
    use earcutr::earcut;

    let mut visited = vec![false; bitmap.len()];
    let mut meshes = Vec::new();

    for start_i in 0..bitmap.len() {
        if !bitmap[start_i] || visited[start_i] {
            continue;
        }

        let mut group_min = IVec2::splat(i32::MAX);
        let mut group_max = IVec2::splat(i32::MIN);
        let mut group = Vec::new();
        let mut queue = VecDeque::new();

        visited[start_i] = true;
        queue.push_back(start_i);

        while let Some(idx) = queue.pop_front() {
            let lx = (idx % bitmap_size) as i32;
            let ly = (idx / bitmap_size) as i32;
            let world_pos = origin + IVec2::new(lx, ly);
            group.push(world_pos);
            group_min = group_min.min(world_pos);
            group_max = group_max.max(world_pos);

            for [dx, dy] in [[1, 0], [-1, 0], [0, 1], [0, -1]] {
                let nx = lx + dx;
                let ny = ly + dy;
                if nx >= 0 && nx < bitmap_size as i32 && ny >= 0 && ny < bitmap_size as i32 {
                    let ni = (ny as usize) * bitmap_size + nx as usize;
                    if bitmap[ni] && !visited[ni] {
                        visited[ni] = true;
                        queue.push_back(ni);
                    }
                }
            }
        }

        let mut grid = OccupancyGrid::new(group_min, group_max);
        for &position in &group {
            grid.set(position);
        }

        let loop_vertices = ordered_perimeter_loop(&grid);
        if loop_vertices.len() < 3 {
            continue;
        }

        let simplified_vertices = simplify_polyline(&loop_vertices, epsilon);
        if simplified_vertices.len() < 3 {
            continue;
        }

        let flattened: Vec<f64> = simplified_vertices
            .iter()
            .flat_map(|v| vec![f64::from(v.x), f64::from(v.y)])
            .collect();

        if let Ok(indices_raw) = earcut(&flattened, &[], 2) {
            let indices = indices_raw
                .chunks(3)
                .map(|c| [c[0] as u32, c[1] as u32, c[2] as u32])
                .collect();

            meshes.push(GridCellMesh {
                vertices: simplified_vertices,
                indices,
            });
        }
    }

    meshes
}

/// Simplify a polyline using the Douglas-Peucker algorithm.
#[must_use]
pub fn simplify_polyline(points: &[Vec2], epsilon: f32) -> Vec<Vec2> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let mut max_dist = 0.0;
    let mut max_index = 0;
    let end = points.len() - 1;

    for i in 1..end {
        let dist = perpendicular_distance(points[i], points[0], points[end]);
        if dist > max_dist {
            max_dist = dist;
            max_index = i;
        }
    }

    if max_dist > epsilon {
        let left = simplify_polyline(&points[0..=max_index], epsilon);
        let right = simplify_polyline(&points[max_index..], epsilon);

        let mut result = left;
        result.extend_from_slice(&right[1..]);
        result
    } else {
        vec![points[0], points[end]]
    }
}

#[derive(Debug)]
#[cfg(feature = "physics")]
struct OccupancyGrid {
    min: IVec2,
    size: IVec2,
    data: Vec<bool>,
}

#[cfg(feature = "physics")]
impl OccupancyGrid {
    fn new(min: IVec2, max: IVec2) -> Self {
        let size = max - min + IVec2::ONE;
        let data = vec![false; (size.x * size.y) as usize];
        Self { min, size, data }
    }

    #[inline(always)]
    fn index(&self, position: IVec2) -> usize {
        let local = position - self.min;
        (local.y * self.size.x + local.x) as usize
    }

    #[inline(always)]
    fn set(&mut self, position: IVec2) {
        let idx = self.index(position);
        self.data[idx] = true;
    }

    #[inline(always)]
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

#[cfg(feature = "physics")]
fn ordered_perimeter_loop(grid: &OccupancyGrid) -> Vec<Vec2> {
    let edges = perimeter_edges(grid);
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

#[cfg(feature = "physics")]
fn perimeter_edges(grid: &OccupancyGrid) -> Vec<[Vec2; 2]> {
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
        let base = position.as_vec2() + Vec2::splat(0.5);
        for (offset, v0, v1) in directions {
            if !grid.get(position + offset) {
                edges.push([base + v0, base + v1]);
            }
        }
    }

    edges
}

#[inline(always)]
fn perpendicular_distance(point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
    let line_vec = line_end - line_start;
    let point_vec = point - line_start;

    let line_len_sq = line_vec.length_squared();
    if line_len_sq < 1e-6 {
        return point_vec.length();
    }

    let t = (point_vec.dot(line_vec) / line_len_sq).clamp(0.0, 1.0);
    let projection = line_start + line_vec * t;
    (point - projection).length()
}

#[cfg(test)]
mod tests {
    use super::*;

    mod grid_tests {
        use super::*;

        #[test]
        fn new_creates_empty_grid() {
            let grid = OccupancyGrid::new(IVec2::ZERO, IVec2::new(3, 3));
            assert!(!grid.get(IVec2::ZERO));
            assert!(!grid.get(IVec2::new(3, 3)));
        }

        #[test]
        fn set_and_get() {
            let mut grid = OccupancyGrid::new(IVec2::ZERO, IVec2::new(3, 3));
            grid.set(IVec2::new(1, 2));
            assert!(grid.get(IVec2::new(1, 2)));
            assert!(!grid.get(IVec2::new(0, 0)));
        }

        #[test]
        fn get_out_of_bounds_returns_false() {
            let grid = OccupancyGrid::new(IVec2::ZERO, IVec2::new(3, 3));
            assert!(!grid.get(IVec2::new(-1, 0)));
            assert!(!grid.get(IVec2::new(4, 0)));
            assert!(!grid.get(IVec2::new(0, -1)));
            assert!(!grid.get(IVec2::new(0, 4)));
        }

        #[test]
        fn negative_origin() {
            let mut grid = OccupancyGrid::new(IVec2::new(-2, -2), IVec2::new(2, 2));
            grid.set(IVec2::new(-1, -1));
            assert!(grid.get(IVec2::new(-1, -1)));
            assert!(!grid.get(IVec2::new(0, 0)));
        }

        #[test]
        fn iter_occupied_returns_set_positions() {
            let mut grid = OccupancyGrid::new(IVec2::ZERO, IVec2::new(3, 3));
            grid.set(IVec2::new(1, 0));
            grid.set(IVec2::new(2, 3));

            let occupied: Vec<_> = grid.iter_occupied().collect();
            assert_eq!(occupied.len(), 2);
            assert!(occupied.contains(&IVec2::new(1, 0)));
            assert!(occupied.contains(&IVec2::new(2, 3)));
        }
    }

    mod simplify_polyline_tests {
        use super::*;

        #[test]
        fn collinear_points_reduce_to_endpoints() {
            let points = vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(2.0, 0.0),
                Vec2::new(3.0, 0.0),
            ];
            let result = simplify_polyline(&points, 0.1);
            assert_eq!(result.len(), 2);
            assert_eq!(result[0], Vec2::new(0.0, 0.0));
            assert_eq!(result[1], Vec2::new(3.0, 0.0));
        }

        #[test]
        fn l_shape_preserves_corner() {
            let points = vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 5.0),
                Vec2::new(5.0, 5.0),
            ];
            let result = simplify_polyline(&points, 0.1);
            assert_eq!(result.len(), 3);
        }

        #[test]
        fn fewer_than_3_points_returned_as_is() {
            let points = vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)];
            let result = simplify_polyline(&points, 0.1);
            assert_eq!(result, points);
        }

        #[test]
        fn large_epsilon_collapses_to_endpoints() {
            let points = vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(2.0, 0.5),
                Vec2::new(3.0, 0.0),
            ];
            let result = simplify_polyline(&points, 100.0);
            assert_eq!(result.len(), 2);
        }
    }

    mod perpendicular_distance_tests {
        use super::*;

        #[test]
        fn point_on_line_returns_zero() {
            let d = perpendicular_distance(
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 0.0),
                Vec2::new(2.0, 0.0),
            );
            assert!(d.abs() < 1e-5);
        }

        #[test]
        fn point_above_horizontal_line() {
            let d = perpendicular_distance(
                Vec2::new(1.0, 3.0),
                Vec2::new(0.0, 0.0),
                Vec2::new(2.0, 0.0),
            );
            assert!((d - 3.0).abs() < 1e-5);
        }

        #[test]
        fn degenerate_line_returns_distance_to_point() {
            let d = perpendicular_distance(
                Vec2::new(3.0, 4.0),
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 0.0),
            );
            assert!((d - 5.0).abs() < 1e-5);
        }
    }
}
