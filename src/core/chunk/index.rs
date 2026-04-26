use bevy::prelude::*;

use super::ChunkCoord;

/// Resource mapping chunk coordinates to chunk entities.
///
/// Uses a flat array with toroidal indexing for O(1) lookups without hashing overhead.
/// The origin and dimensions match the [`SpatialMap`](crate::SpatialMap) chunk layout.
#[derive(Resource, Clone, Debug)]
pub struct ChunkIndex {
    data: Vec<Option<Entity>>,
    origin: IVec2,
    width: u32,
    height: u32,
    width_shift: u32,
    width_mask: u32,
    height_mask: u32,
    chunk_size: u32,
    chunk_shift: u32,
}

impl Default for ChunkIndex {
    fn default() -> Self {
        Self::new(16, 16, 32, IVec2::ZERO)
    }
}

impl ChunkIndex {
    /// Create a new chunk index with the given dimensions.
    ///
    /// # Arguments
    /// * `width` - Number of chunks wide (must be power of 2)
    /// * `height` - Number of chunks tall (must be power of 2)
    /// * `chunk_size` - Size of each chunk in world units (must be power of 2)
    /// * `origin` - Origin in chunk coordinates
    ///
    /// # Panics
    /// Panics if width, height, or `chunk_size` is not a power of 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ChunkIndex;
    ///
    /// let index = ChunkIndex::new(16, 16, 32, IVec2::ZERO);
    /// assert_eq!(index.width(), 16);
    /// assert_eq!(index.height(), 16);
    /// assert_eq!(index.chunk_size(), 32);
    /// ```
    #[must_use]
    pub fn new(width: u32, height: u32, chunk_size: u32, origin: IVec2) -> Self {
        assert!(width.is_power_of_two(), "Width must be a power of 2");
        assert!(height.is_power_of_two(), "Height must be a power of 2");
        assert!(
            chunk_size.is_power_of_two(),
            "Chunk size must be a power of 2"
        );

        let capacity = (width as usize) * (height as usize);

        Self {
            data: vec![None; capacity],
            origin,
            width,
            height,
            width_shift: width.trailing_zeros(),
            width_mask: width - 1,
            height_mask: height - 1,
            chunk_size,
            chunk_shift: chunk_size.trailing_zeros(),
        }
    }

    /// Get the chunk size in world units.
    #[inline(always)]
    #[must_use]
    pub const fn chunk_size(&self) -> u32 {
        self.chunk_size
    }

    /// Get the origin in chunk coordinates.
    #[inline(always)]
    #[must_use]
    pub const fn origin(&self) -> IVec2 {
        self.origin
    }

    /// Get the width in chunks.
    #[inline(always)]
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Get the height in chunks.
    #[inline(always)]
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Check if a chunk coordinate is within the indexed region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{ChunkIndex, ChunkCoord};
    ///
    /// let index = ChunkIndex::new(16, 16, 32, IVec2::ZERO);
    /// assert!(index.is_coord_loaded(ChunkCoord::new(0, 0)));
    /// assert!(index.is_coord_loaded(ChunkCoord::new(15, 15)));
    /// assert!(!index.is_coord_loaded(ChunkCoord::new(16, 0)));
    /// ```
    #[inline(always)]
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub const fn is_coord_loaded(&self, coord: ChunkCoord) -> bool {
        let local_x = coord.x().wrapping_sub(self.origin.x);
        let local_y = coord.y().wrapping_sub(self.origin.y);
        local_x >= 0 && local_x < self.width as i32 && local_y >= 0 && local_y < self.height as i32
    }

    /// Convert a chunk coordinate to an array index using toroidal indexing.
    #[inline(always)]
    #[must_use]
    const fn index(&self, coord: ChunkCoord) -> usize {
        let wrapped_x = (coord.x() as u32) & self.width_mask;
        let wrapped_y = (coord.y() as u32) & self.height_mask;
        ((wrapped_y << self.width_shift) | wrapped_x) as usize
    }

    /// Insert a chunk entity at a coordinate.
    ///
    /// Returns the previous entity at that coordinate, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{ChunkIndex, ChunkCoord};
    ///
    /// let mut index = ChunkIndex::new(16, 16, 32, IVec2::ZERO);
    /// let entity = Entity::from_bits(42);
    ///
    /// assert_eq!(index.insert(ChunkCoord::new(0, 0), entity), None);
    /// assert_eq!(index.get(ChunkCoord::new(0, 0)), Some(entity));
    /// ```
    pub fn insert(&mut self, coord: ChunkCoord, entity: Entity) -> Option<Entity> {
        if self.is_coord_loaded(coord) {
            let idx = self.index(coord);
            self.data[idx].replace(entity)
        } else {
            None
        }
    }

    /// Remove a chunk entity at a coordinate.
    ///
    /// Returns the removed entity, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{ChunkIndex, ChunkCoord};
    ///
    /// let mut index = ChunkIndex::new(16, 16, 32, IVec2::ZERO);
    /// let entity = Entity::from_bits(42);
    ///
    /// index.insert(ChunkCoord::new(0, 0), entity);
    /// assert_eq!(index.remove(ChunkCoord::new(0, 0)), Some(entity));
    /// assert_eq!(index.get(ChunkCoord::new(0, 0)), None);
    /// ```
    pub fn remove(&mut self, coord: ChunkCoord) -> Option<Entity> {
        if self.is_coord_loaded(coord) {
            let idx = self.index(coord);
            self.data[idx].take()
        } else {
            None
        }
    }

    /// Get the chunk entity at a coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{ChunkIndex, ChunkCoord};
    ///
    /// let mut index = ChunkIndex::new(16, 16, 32, IVec2::ZERO);
    /// let entity = Entity::from_bits(42);
    /// index.insert(ChunkCoord::new(0, 0), entity);
    ///
    /// assert_eq!(index.get(ChunkCoord::new(0, 0)), Some(entity));
    /// assert_eq!(index.get(ChunkCoord::new(1, 1)), None);
    /// ```
    #[must_use]
    pub fn get(&self, coord: ChunkCoord) -> Option<Entity> {
        if self.is_coord_loaded(coord) {
            self.data[self.index(coord)]
        } else {
            None
        }
    }

    /// Check if a chunk exists at a coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{ChunkIndex, ChunkCoord};
    ///
    /// let mut index = ChunkIndex::new(16, 16, 32, IVec2::ZERO);
    /// index.insert(ChunkCoord::new(0, 0), Entity::from_bits(1));
    ///
    /// assert!(index.contains(ChunkCoord::new(0, 0)));
    /// assert!(!index.contains(ChunkCoord::new(1, 1)));
    /// ```
    #[must_use]
    pub fn contains(&self, coord: ChunkCoord) -> bool {
        self.get(coord).is_some()
    }

    /// Get the number of chunks currently indexed.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{ChunkIndex, ChunkCoord};
    ///
    /// let mut index = ChunkIndex::new(16, 16, 32, IVec2::ZERO);
    /// assert_eq!(index.len(), 0);
    ///
    /// index.insert(ChunkCoord::new(0, 0), Entity::from_bits(1));
    /// assert_eq!(index.len(), 1);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.iter().filter(|e| e.is_some()).count()
    }

    /// Check if the index is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{ChunkIndex, ChunkCoord};
    ///
    /// let mut index = ChunkIndex::new(16, 16, 32, IVec2::ZERO);
    /// assert!(index.is_empty());
    ///
    /// index.insert(ChunkCoord::new(0, 0), Entity::from_bits(1));
    /// assert!(!index.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.iter().all(std::option::Option::is_none)
    }

    /// Iterate over all chunk coordinates and entities in the loaded region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{ChunkIndex, ChunkCoord};
    ///
    /// let mut index = ChunkIndex::new(16, 16, 32, IVec2::ZERO);
    /// index.insert(ChunkCoord::new(0, 0), Entity::from_bits(1));
    /// index.insert(ChunkCoord::new(5, 5), Entity::from_bits(2));
    ///
    /// let chunks: Vec<_> = index.iter().collect();
    /// assert_eq!(chunks.len(), 2);
    /// ```
    #[allow(clippy::cast_possible_wrap)]
    pub fn iter(&self) -> impl Iterator<Item = (ChunkCoord, Entity)> + '_ {
        let origin = self.origin;
        let width = self.width as i32;
        let height = self.height as i32;
        let width_mask = self.width_mask;
        let height_mask = self.height_mask;
        let width_shift = self.width_shift;

        (0..height).flat_map(move |y| {
            (0..width).filter_map(move |x| {
                let coord = ChunkCoord::new(origin.x + x, origin.y + y);
                let wrapped_x = (coord.x() as u32) & width_mask;
                let wrapped_y = (coord.y() as u32) & height_mask;
                let idx = ((wrapped_y << width_shift) | wrapped_x) as usize;
                self.data[idx].map(|entity| (coord, entity))
            })
        })
    }

    /// Get all chunk coordinates that have entities.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{ChunkIndex, ChunkCoord};
    ///
    /// let mut index = ChunkIndex::new(16, 16, 32, IVec2::ZERO);
    /// index.insert(ChunkCoord::new(0, 0), Entity::from_bits(1));
    /// index.insert(ChunkCoord::new(3, 3), Entity::from_bits(2));
    ///
    /// let coords: Vec<_> = index.coords().collect();
    /// assert_eq!(coords.len(), 2);
    /// ```
    pub fn coords(&self) -> impl Iterator<Item = ChunkCoord> + '_ {
        self.iter().map(|(coord, _)| coord)
    }

    /// Get all chunk entities.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{ChunkIndex, ChunkCoord};
    ///
    /// let mut index = ChunkIndex::new(16, 16, 32, IVec2::ZERO);
    /// index.insert(ChunkCoord::new(0, 0), Entity::from_bits(1));
    ///
    /// let entities: Vec<_> = index.entities().collect();
    /// assert_eq!(entities.len(), 1);
    /// ```
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.data.iter().filter_map(|e| *e)
    }

    /// Shift the origin to a new position.
    ///
    /// This does not copy data - it just changes how indices map to coordinates.
    /// Caller is responsible for handling chunks that are no longer valid.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ChunkIndex;
    ///
    /// let mut index = ChunkIndex::new(16, 16, 32, IVec2::ZERO);
    /// index.shift_origin(IVec2::new(4, 4));
    /// assert_eq!(index.origin(), IVec2::new(4, 4));
    /// ```
    pub const fn shift_origin(&mut self, new_origin: IVec2) {
        self.origin = new_origin;
    }

    /// Clear all chunk entities in a region (set to None).
    ///
    /// Clears array indices corresponding to the given coordinate range.
    /// Does not check if coordinates are currently loaded.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{ChunkIndex, ChunkCoord};
    ///
    /// let mut index = ChunkIndex::new(16, 16, 32, IVec2::ZERO);
    /// index.insert(ChunkCoord::new(0, 0), Entity::from_bits(1));
    /// index.insert(ChunkCoord::new(1, 1), Entity::from_bits(2));
    ///
    /// index.clear_region(ChunkCoord::new(0, 0), ChunkCoord::new(1, 1));
    /// assert!(index.is_empty());
    /// ```
    pub fn clear_region(&mut self, min: ChunkCoord, max: ChunkCoord) {
        for y in min.y()..=max.y() {
            for x in min.x()..=max.x() {
                let coord = ChunkCoord::new(x, y);
                let idx = self.index(coord);
                self.data[idx] = None;
            }
        }
    }

    /// Clear all data in the index.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{ChunkIndex, ChunkCoord};
    ///
    /// let mut index = ChunkIndex::new(16, 16, 32, IVec2::ZERO);
    /// index.insert(ChunkCoord::new(0, 0), Entity::from_bits(1));
    /// index.clear();
    /// assert!(index.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.data.fill(None);
    }

    /// Convert a world position to a chunk coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{ChunkIndex, ChunkCoord};
    ///
    /// let index = ChunkIndex::new(16, 16, 32, IVec2::ZERO);
    ///
    /// assert_eq!(index.world_to_chunk_coord(IVec2::new(0, 0)), ChunkCoord::new(0, 0));
    /// assert_eq!(index.world_to_chunk_coord(IVec2::new(31, 31)), ChunkCoord::new(0, 0));
    /// assert_eq!(index.world_to_chunk_coord(IVec2::new(32, 0)), ChunkCoord::new(1, 0));
    /// ```
    #[inline(always)]
    #[must_use]
    pub const fn world_to_chunk_coord(&self, position: IVec2) -> ChunkCoord {
        ChunkCoord::new(
            position.x >> self.chunk_shift,
            position.y >> self.chunk_shift,
        )
    }

    /// Convert a chunk coordinate to its world region (bounding rect).
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{ChunkIndex, ChunkCoord};
    ///
    /// let index = ChunkIndex::new(16, 16, 32, IVec2::ZERO);
    /// let region = index.chunk_coord_to_chunk_region(ChunkCoord::new(1, 0));
    ///
    /// assert_eq!(region.min, IVec2::new(32, 0));
    /// assert_eq!(region.max, IVec2::new(63, 31));
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn chunk_coord_to_chunk_region(&self, coord: ChunkCoord) -> IRect {
        let min_x = coord.x() << self.chunk_shift;
        let min_y = coord.y() << self.chunk_shift;
        let max_x = min_x | (self.chunk_size as i32 - 1);
        let max_y = min_y | (self.chunk_size as i32 - 1);
        IRect::new(min_x, min_y, max_x, max_y)
    }

    /// Get all chunk coordinates within the loaded region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ChunkIndex;
    ///
    /// let index = ChunkIndex::new(4, 4, 32, IVec2::ZERO);
    /// let coords = index.loaded_chunk_coords();
    /// assert_eq!(coords.len(), 16);
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn loaded_chunk_coords(&self) -> Vec<ChunkCoord> {
        let mut coords = Vec::new();
        for y in self.origin.y..self.origin.y + self.height as i32 {
            for x in self.origin.x..self.origin.x + self.width as i32 {
                coords.push(ChunkCoord::new(x, y));
            }
        }
        coords
    }

    /// Get chunk coordinates for a specific group (0-3) within the loaded region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ChunkIndex;
    ///
    /// let index = ChunkIndex::new(4, 4, 32, IVec2::ZERO);
    /// let group0 = index.chunk_coords_for_group(0);
    /// assert_eq!(group0.len(), 4);
    /// assert!(group0.iter().all(|c| c.group() == 0));
    /// ```
    #[must_use]
    pub fn chunk_coords_for_group(&self, group: u8) -> Vec<ChunkCoord> {
        self.loaded_chunk_coords()
            .into_iter()
            .filter(|coord| coord.group() == group)
            .collect()
    }

    /// Partition all loaded chunks into 4 groups for checkerboard parallel processing.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ChunkIndex;
    ///
    /// let index = ChunkIndex::new(4, 4, 32, IVec2::ZERO);
    /// let groups = index.partition_chunks_by_group();
    ///
    /// assert_eq!(groups.len(), 4);
    /// let total: usize = groups.iter().map(|g| g.len()).sum();
    /// assert_eq!(total, 16);
    /// ```
    #[must_use]
    pub fn partition_chunks_by_group(&self) -> [Vec<ChunkCoord>; 4] {
        let mut groups: [Vec<ChunkCoord>; 4] = Default::default();
        for coord in self.loaded_chunk_coords() {
            let group_idx = coord.group() as usize;
            groups[group_idx].push(coord);
        }
        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_index() -> ChunkIndex {
        ChunkIndex::new(16, 16, 32, IVec2::ZERO)
    }

    #[test]
    fn new_creates_index_with_correct_dimensions() {
        let index = create_index();

        assert_eq!(index.width(), 16);
        assert_eq!(index.height(), 16);
        assert_eq!(index.origin(), IVec2::ZERO);
    }

    #[test]
    #[should_panic(expected = "power of 2")]
    fn new_panics_on_non_power_of_two_width() {
        let _ = ChunkIndex::new(15, 16, 32, IVec2::ZERO);
    }

    #[test]
    #[should_panic(expected = "power of 2")]
    fn new_panics_on_non_power_of_two_height() {
        let _ = ChunkIndex::new(16, 15, 32, IVec2::ZERO);
    }

    #[test]
    fn insert_and_get() {
        let mut index = create_index();
        let coord = ChunkCoord::new(5, 5);
        let entity = Entity::from_bits(42);

        index.insert(coord, entity);

        assert_eq!(index.get(coord), Some(entity));
        assert!(index.contains(coord));
    }

    #[test]
    fn insert_returns_previous() {
        let mut index = create_index();
        let coord = ChunkCoord::new(5, 5);
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);

        index.insert(coord, e1);
        let prev = index.insert(coord, e2);

        assert_eq!(prev, Some(e1));
        assert_eq!(index.get(coord), Some(e2));
    }

    #[test]
    fn insert_outside_region_fails() {
        let mut index = create_index();
        let entity = Entity::from_bits(42);

        let result = index.insert(ChunkCoord::new(20, 20), entity);

        assert!(result.is_none());
        assert!(!index.contains(ChunkCoord::new(20, 20)));
    }

    #[test]
    fn remove_returns_entity() {
        let mut index = create_index();
        let coord = ChunkCoord::new(5, 5);
        let entity = Entity::from_bits(42);

        index.insert(coord, entity);
        let removed = index.remove(coord);

        assert_eq!(removed, Some(entity));
        assert!(!index.contains(coord));
    }

    #[test]
    fn len_and_is_empty() {
        let mut index = create_index();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);

        index.insert(ChunkCoord::new(0, 0), Entity::from_bits(1));
        assert!(!index.is_empty());
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn is_coord_loaded() {
        let index = create_index();

        assert!(index.is_coord_loaded(ChunkCoord::new(0, 0)));
        assert!(index.is_coord_loaded(ChunkCoord::new(15, 15)));
        assert!(!index.is_coord_loaded(ChunkCoord::new(16, 0)));
        assert!(!index.is_coord_loaded(ChunkCoord::new(-1, 0)));
    }

    #[test]
    fn shift_origin() {
        let mut index = ChunkIndex::new(16, 16, 32, IVec2::ZERO);
        let entity = Entity::from_bits(42);

        index.insert(ChunkCoord::new(5, 5), entity);
        index.shift_origin(IVec2::new(4, 4));

        assert_eq!(index.origin(), IVec2::new(4, 4));

        assert!(index.is_coord_loaded(ChunkCoord::new(5, 5)));

        assert_eq!(index.get(ChunkCoord::new(5, 5)), Some(entity));
    }

    #[test]
    fn iter_yields_all_chunks() {
        let mut index = create_index();

        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);

        index.insert(ChunkCoord::new(0, 0), e1);
        index.insert(ChunkCoord::new(5, 5), e2);

        assert_eq!(index.iter().count(), 2);
    }
}
