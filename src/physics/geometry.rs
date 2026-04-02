use avian2d::math::Vector;
use bevy::log::warn;
use bevy::prelude::*;
use std::collections::VecDeque;

use crate::core::ChunkCoord;

pub(super) struct MeshGenerationResult {
    pub(super) chunk_coord: ChunkCoord,
    pub(super) vertices: Vec<Vec<Vector>>,
    pub(super) indices: Vec<Vec<[u32; 3]>>,
}

#[derive(Debug)]
pub(super) struct Grid {
    min: IVec2,
    size: IVec2,
    data: Vec<bool>,
}

impl Grid {
    pub(super) fn new(min: IVec2, max: IVec2) -> Self {
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
    pub(super) fn set(&mut self, position: IVec2) {
        let idx = self.index(position);
        self.data[idx] = true;
    }

    #[inline(always)]
    pub(super) fn get(&self, position: IVec2) -> bool {
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

pub(super) fn process_bitmap(
    bitmap: &[bool],
    origin: IVec2,
    chunk_size: usize,
    epsilon: f32,
) -> (Vec<Vec<Vector>>, Vec<Vec<[u32; 3]>>) {
    use earcutr::earcut;

    let mut visited = vec![false; bitmap.len()];
    let mut all_vertices = Vec::new();
    let mut all_indices = Vec::new();

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
            let lx = (idx % chunk_size) as i32;
            let ly = (idx / chunk_size) as i32;
            let world_pos = origin + IVec2::new(lx, ly);
            group.push(world_pos);
            group_min = group_min.min(world_pos);
            group_max = group_max.max(world_pos);

            for [dx, dy] in [[1, 0], [-1, 0], [0, 1], [0, -1]] {
                let nx = lx + dx;
                let ny = ly + dy;
                if nx >= 0 && nx < chunk_size as i32 && ny >= 0 && ny < chunk_size as i32 {
                    let ni = (ny as usize) * chunk_size + nx as usize;
                    if bitmap[ni] && !visited[ni] {
                        visited[ni] = true;
                        queue.push_back(ni);
                    }
                }
            }
        }

        let mut grid = Grid::new(group_min, group_max);
        for &position in &group {
            grid.set(position);
        }

        let loop_vertices = extract_ordered_perimeter_loop(&grid);
        if loop_vertices.len() < 3 {
            continue;
        }

        let simplified_vertices = douglas_peucker(&loop_vertices, epsilon);
        if simplified_vertices.len() < 3 {
            continue;
        }

        let flattened: Vec<f64> = simplified_vertices
            .iter()
            .flat_map(|v| vec![f64::from(v.x), f64::from(v.y)])
            .collect();

        if let Ok(indices_raw) = earcut(&flattened, &[], 2) {
            let triangle_indices: Vec<[u32; 3]> = indices_raw
                .chunks(3)
                .map(|c| [c[0] as u32, c[1] as u32, c[2] as u32])
                .collect();

            let vertices = simplified_vertices
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
        let base = position.as_vec2() + Vec2::splat(0.5);
        for (offset, v0, v1) in directions {
            if !grid.get(position + offset) {
                edges.push([base + v0, base + v1]);
            }
        }
    }

    edges
}

fn douglas_peucker(points: &[Vec2], epsilon: f32) -> Vec<Vec2> {
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
        let left = douglas_peucker(&points[0..=max_index], epsilon);
        let right = douglas_peucker(&points[max_index..], epsilon);

        let mut result = left;
        result.extend_from_slice(&right[1..]);
        result
    } else {
        vec![points[0], points[end]]
    }
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

#[allow(clippy::unused_async)]
pub(super) async fn generate_mesh_from_bitmap(
    chunk_coord: ChunkCoord,
    bitmap: Vec<bool>,
    origin: IVec2,
    chunk_size: usize,
    epsilon: f32,
) -> MeshGenerationResult {
    let (vertices, indices) = process_bitmap(&bitmap, origin, chunk_size, epsilon);
    MeshGenerationResult {
        chunk_coord,
        vertices,
        indices,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod grid_tests {
        use super::*;

        #[test]
        fn new_creates_empty_grid() {
            let grid = Grid::new(IVec2::ZERO, IVec2::new(3, 3));
            assert!(!grid.get(IVec2::ZERO));
            assert!(!grid.get(IVec2::new(3, 3)));
        }

        #[test]
        fn set_and_get() {
            let mut grid = Grid::new(IVec2::ZERO, IVec2::new(3, 3));
            grid.set(IVec2::new(1, 2));
            assert!(grid.get(IVec2::new(1, 2)));
            assert!(!grid.get(IVec2::new(0, 0)));
        }

        #[test]
        fn get_out_of_bounds_returns_false() {
            let grid = Grid::new(IVec2::ZERO, IVec2::new(3, 3));
            assert!(!grid.get(IVec2::new(-1, 0)));
            assert!(!grid.get(IVec2::new(4, 0)));
            assert!(!grid.get(IVec2::new(0, -1)));
            assert!(!grid.get(IVec2::new(0, 4)));
        }

        #[test]
        fn negative_origin() {
            let mut grid = Grid::new(IVec2::new(-2, -2), IVec2::new(2, 2));
            grid.set(IVec2::new(-1, -1));
            assert!(grid.get(IVec2::new(-1, -1)));
            assert!(!grid.get(IVec2::new(0, 0)));
        }

        #[test]
        fn iter_occupied_returns_set_positions() {
            let mut grid = Grid::new(IVec2::ZERO, IVec2::new(3, 3));
            grid.set(IVec2::new(1, 0));
            grid.set(IVec2::new(2, 3));

            let occupied: Vec<_> = grid.iter_occupied().collect();
            assert_eq!(occupied.len(), 2);
            assert!(occupied.contains(&IVec2::new(1, 0)));
            assert!(occupied.contains(&IVec2::new(2, 3)));
        }
    }

    mod douglas_peucker_tests {
        use super::*;

        #[test]
        fn collinear_points_reduce_to_endpoints() {
            let points = vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(2.0, 0.0),
                Vec2::new(3.0, 0.0),
            ];
            let result = douglas_peucker(&points, 0.1);
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
            let result = douglas_peucker(&points, 0.1);
            assert_eq!(result.len(), 3);
        }

        #[test]
        fn fewer_than_3_points_returned_as_is() {
            let points = vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)];
            let result = douglas_peucker(&points, 0.1);
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
            let result = douglas_peucker(&points, 100.0);
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
