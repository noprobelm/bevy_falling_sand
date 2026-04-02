use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents chunk coordinates in the grid.
///
/// Chunk coordinates are distinct from world coordinates - they identify
/// which chunk a position belongs to, not the position within the chunk.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Reflect, Serialize, Deserialize)]
pub struct ChunkCoord(IVec2);

impl PartialOrd for ChunkCoord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ChunkCoord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.0.y.cmp(&other.0.y) {
            std::cmp::Ordering::Equal => self.0.x.cmp(&other.0.x),
            ord => ord,
        }
    }
}

impl From<ChunkCoord> for IVec2 {
    fn from(coord: ChunkCoord) -> Self {
        coord.0
    }
}

impl ChunkCoord {
    /// Create a new chunk coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_falling_sand::core::ChunkCoord;
    ///
    /// let coord = ChunkCoord::new(3, 5);
    /// assert_eq!(coord.x(), 3);
    /// assert_eq!(coord.y(), 5);
    /// ```
    #[inline(always)]
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self(IVec2::new(x, y))
    }

    /// Get the x coordinate.
    #[inline(always)]
    #[must_use]
    pub const fn x(&self) -> i32 {
        self.0.x
    }

    /// Get the y coordinate.
    #[inline(always)]
    #[must_use]
    pub const fn y(&self) -> i32 {
        self.0.y
    }

    /// Get the checkerboard group (0-3) for parallel processing.
    ///
    /// - Group 0: even x, even y
    /// - Group 1: odd x, even y
    /// - Group 2: even x, odd y
    /// - Group 3: odd x, odd y
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_falling_sand::core::ChunkCoord;
    ///
    /// assert_eq!(ChunkCoord::new(0, 0).group(), 0);
    /// assert_eq!(ChunkCoord::new(1, 0).group(), 1);
    /// assert_eq!(ChunkCoord::new(0, 1).group(), 2);
    /// assert_eq!(ChunkCoord::new(1, 1).group(), 3);
    /// ```
    #[inline(always)]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn group(&self) -> u8 {
        let x_parity = (self.0.x & 1) as u8;
        let y_parity = (self.0.y & 1) as u8;
        (y_parity << 1) | x_parity
    }

    /// Get neighboring chunk coordinates (8 directions).
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_falling_sand::core::ChunkCoord;
    ///
    /// let coord = ChunkCoord::new(5, 5);
    /// let neighbors = coord.neighbors();
    /// assert_eq!(neighbors.len(), 8);
    /// ```
    #[must_use]
    pub fn neighbors(&self) -> [Self; 8] {
        [
            Self(self.0 + IVec2::new(-1, 0)),
            Self(self.0 + IVec2::new(1, 0)),
            Self(self.0 + IVec2::new(0, 1)),
            Self(self.0 + IVec2::new(0, -1)),
            Self(self.0 + IVec2::new(-1, 1)),
            Self(self.0 + IVec2::new(1, 1)),
            Self(self.0 + IVec2::new(-1, -1)),
            Self(self.0 + IVec2::new(1, -1)),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn new_creates_chunk_coord() {
        let coord = ChunkCoord::new(5, 10);

        assert_eq!(coord.x(), 5);
        assert_eq!(coord.y(), 10);
        assert_eq!(IVec2::from(coord), IVec2::new(5, 10));
    }

    #[test]
    fn new_with_negative_values() {
        let coord = ChunkCoord::new(-3, -7);

        assert_eq!(coord.x(), -3);
        assert_eq!(coord.y(), -7);
    }

    #[test]
    fn neighbors_are_adjacent() {
        let coord = ChunkCoord::new(5, 5);
        let neighbors = coord.neighbors();

        assert_eq!(neighbors.len(), 8);

        for neighbor in &neighbors {
            let dx = (neighbor.x() - coord.x()).abs();
            let dy = (neighbor.y() - coord.y()).abs();

            assert!(dx <= 1 && dy <= 1, "Neighbor too far: {neighbor:?}");
            assert!(dx > 0 || dy > 0, "Neighbor is same as original");
        }
    }

    #[test]
    fn neighbors_are_unique() {
        let coord = ChunkCoord::new(5, 5);
        let neighbors = coord.neighbors();
        let set: HashSet<_> = neighbors.iter().collect();

        assert_eq!(set.len(), 8);
    }

    #[test]
    fn neighbors_contains_all_directions() {
        let coord = ChunkCoord::new(5, 5);
        let neighbors = coord.neighbors();
        let set: HashSet<_> = neighbors.into_iter().collect();

        assert!(set.contains(&ChunkCoord::new(4, 5)));
        assert!(set.contains(&ChunkCoord::new(6, 5)));
        assert!(set.contains(&ChunkCoord::new(5, 6)));
        assert!(set.contains(&ChunkCoord::new(5, 4)));

        assert!(set.contains(&ChunkCoord::new(4, 6)));
        assert!(set.contains(&ChunkCoord::new(6, 6)));
        assert!(set.contains(&ChunkCoord::new(4, 4)));
        assert!(set.contains(&ChunkCoord::new(6, 4)));
    }

    #[test]
    fn neighbors_at_origin() {
        let coord = ChunkCoord::new(0, 0);
        let neighbors = coord.neighbors();
        let set: HashSet<_> = neighbors.into_iter().collect();

        assert!(set.contains(&ChunkCoord::new(-1, 0)));
        assert!(set.contains(&ChunkCoord::new(1, 0)));
        assert!(set.contains(&ChunkCoord::new(0, 1)));
        assert!(set.contains(&ChunkCoord::new(0, -1)));
        assert!(set.contains(&ChunkCoord::new(-1, 1)));
        assert!(set.contains(&ChunkCoord::new(1, 1)));
        assert!(set.contains(&ChunkCoord::new(-1, -1)));
        assert!(set.contains(&ChunkCoord::new(1, -1)));
    }
}
