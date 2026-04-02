use bevy::prelude::*;

/// Component marking an entity as a chunk. Stores the world region this chunk covers.
#[derive(Component, Clone, Debug, PartialEq, Eq, Reflect)]
pub struct ChunkRegion(IRect);

impl ChunkRegion {
    /// Create a new chunk region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ChunkRegion;
    ///
    /// let region = ChunkRegion::new(IRect::new(0, 0, 31, 31));
    /// assert_eq!(region.region(), IRect::new(0, 0, 31, 31));
    /// ```
    #[must_use]
    pub const fn new(region: IRect) -> Self {
        Self(region)
    }

    /// Get the region this chunk covers.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ChunkRegion;
    ///
    /// let region = ChunkRegion::new(IRect::new(0, 0, 31, 31));
    /// assert_eq!(region.region(), IRect::new(0, 0, 31, 31));
    /// ```
    #[must_use]
    pub const fn region(&self) -> IRect {
        self.0
    }

    /// Get the width of this chunk.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ChunkRegion;
    ///
    /// let region = ChunkRegion::new(IRect::new(0, 0, 31, 31));
    /// assert_eq!(region.width(), 31);
    /// ```
    #[must_use]
    pub fn width(&self) -> i32 {
        self.0.width()
    }

    /// Get the height of this chunk.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ChunkRegion;
    ///
    /// let region = ChunkRegion::new(IRect::new(0, 0, 31, 31));
    /// assert_eq!(region.height(), 31);
    /// ```
    #[must_use]
    pub fn height(&self) -> i32 {
        self.0.height()
    }

    /// Check if a position is within this chunk's region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ChunkRegion;
    ///
    /// let region = ChunkRegion::new(IRect::new(0, 0, 31, 31));
    /// assert!(region.contains(IVec2::new(15, 15)));
    /// assert!(!region.contains(IVec2::new(32, 0)));
    /// ```
    #[must_use]
    pub fn contains(&self, position: IVec2) -> bool {
        self.0.contains(position)
    }
}

impl Default for ChunkRegion {
    fn default() -> Self {
        Self(IRect::new(0, 0, 31, 31))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_region() {
        let region = IRect::new(0, 0, 31, 31);
        let chunk = ChunkRegion::new(region);

        assert_eq!(chunk.region(), region);
    }

    #[test]
    fn contains_checks_bounds() {
        let chunk = ChunkRegion::new(IRect::new(0, 0, 31, 31));

        assert!(chunk.contains(IVec2::new(0, 0)));
        assert!(chunk.contains(IVec2::new(15, 15)));
        assert!(chunk.contains(IVec2::new(31, 31)));
        assert!(!chunk.contains(IVec2::new(-1, 0)));
        assert!(!chunk.contains(IVec2::new(32, 0)));
    }
}
