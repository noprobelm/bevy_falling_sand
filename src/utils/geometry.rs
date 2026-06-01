//! Geometry utilities that can be helpful for grouping blobs of connected `IVec2` positions.
use std::collections::VecDeque;

use bevy::{platform::collections::HashSet, prelude::*};

/// A triangle mesh generated from occupied particle-grid cells.
#[cfg(feature = "physics")]
#[derive(Clone, Debug, Default)]
pub struct GridCellMesh {
    /// Mesh vertices in the same coordinate space as the input grid cells.
    pub vertices: Vec<Vec2>,
    /// Triangle indices into [`Self::vertices`].
    pub indices: Vec<[u32; 3]>,
}

/// Group unordered, disconnected positions into blobs of connected positions.
///
/// Duplicate position inputs are ignored.
///
/// ```
/// use bevy::prelude::IVec2;
/// use bevy_falling_sand::physics::geometry::connected_components;
///
/// let positions = [
///     IVec2::new(0, 0), IVec2::new(1, 0) // This blob is connected
///     IVec2::new(5, 5)                   // Isolated from the first blob
/// ];
/// let components = connected_components(positions);
/// assert_eq!(comopnents.len(), 2);
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

/// Return the cells in `component` that have at least one missing cardinal neighbor
///
/// `component` is treated as the universe of "filled" cells; any cell outside `component` counts as
/// empty for the neighbor test. For a connected blob produced by [`connected_components`], this
/// would yield the blob's outline.
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

    let (mesh_vertices, mesh_indices) =
        crate::physics::geometry::process_bitmap(&bitmap, min, bitmap_size, epsilon);

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for (component_vertices, component_indices) in mesh_vertices.into_iter().zip(mesh_indices) {
        let offset = vertices.len() as u32;
        vertices.extend(component_vertices);
        indices.extend(
            component_indices
                .into_iter()
                .map(|[a, b, c]| [a + offset, b + offset, c + offset]),
        );
    }

    GridCellMesh { vertices, indices }
}
