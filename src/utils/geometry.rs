//! Geometry utilities that can be helpful for grouping blobs of connected `IVec2` positions.
use std::collections::VecDeque;

use bevy::{platform::collections::HashSet, prelude::*};

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
