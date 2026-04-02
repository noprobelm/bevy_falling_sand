use bevy::prelude::*;

/// Error returned when accessing a position outside the loaded region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionUnloaded(IVec2);

impl From<PositionUnloaded> for IVec2 {
    fn from(pos: PositionUnloaded) -> Self {
        pos.0
    }
}

impl PositionUnloaded {
    /// Create a new unloaded position.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_falling_sand::core::PositionUnloaded;
    ///
    /// let err = PositionUnloaded::new(100, 200);
    /// assert_eq!(err.x(), 100);
    /// assert_eq!(err.y(), 200);
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
}

/// A discrete position spatial map built for efficient indexing of generic types.
///
/// The map's width and height must be powers of 2 (but do not have to be equal to each other)
/// and are measured in world units.
///
/// The origin can be any [`IVec2`] value and is designed to be changed at any time. This map
/// uses toroidal indexing so that shifting the origin does not require copying data.
///
/// # Type Parameters
/// * `T` - The type stored at each position.
#[derive(Clone, Debug, Resource)]
pub struct SpatialMap<T> {
    data: Vec<Option<T>>,
    origin: IVec2,
    width: u32,
    height: u32,
    width_shift: u32,
    width_mask: u32,
    height_mask: u32,
}

impl<T: Clone> SpatialMap<T> {
    /// Create a new spatial map with the given dimensions.
    ///
    /// # Arguments
    /// * `width` - Width of the map (must be power of 2)
    /// * `height` - Height of the map (must be power of 2)
    /// * `origin` - Initial world coordinate of the top-left corner
    ///
    /// # Panics
    /// Panics if width or height is not a power of 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let map = SpatialMap::<u32>::new(64, 32, IVec2::ZERO);
    /// assert_eq!(map.width(), 64);
    /// assert_eq!(map.height(), 32);
    /// ```
    #[must_use]
    pub fn new(width: u32, height: u32, origin: IVec2) -> Self {
        assert!(width.is_power_of_two(), "Width must be a power of 2");
        assert!(height.is_power_of_two(), "Height must be a power of 2");

        let capacity = (width as usize) * (height as usize);

        Self {
            data: vec![None; capacity],
            origin,
            width,
            height,
            width_shift: width.trailing_zeros(),
            width_mask: width - 1,
            height_mask: height - 1,
        }
    }

    /// Get the width of the loaded region.
    #[inline(always)]
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Get the height of the loaded region.
    #[inline(always)]
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Get the current origin (top-left world coordinate of loaded region).
    #[inline(always)]
    #[must_use]
    pub const fn origin(&self) -> IVec2 {
        self.origin
    }

    /// Get the loaded region as a rectangle in world coordinates.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let map = SpatialMap::<u32>::new(64, 32, IVec2::new(10, 20));
    /// let region = map.loaded_region();
    /// assert_eq!(region.min, IVec2::new(10, 20));
    /// assert_eq!(region.max, IVec2::new(73, 51));
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn loaded_region(&self) -> IRect {
        IRect::new(
            self.origin.x,
            self.origin.y,
            self.origin.x + self.width as i32 - 1,
            self.origin.y + self.height as i32 - 1,
        )
    }

    /// Check if a world position is within the currently loaded region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// assert!(map.is_position_loaded(IVec2::new(0, 0)));
    /// assert!(map.is_position_loaded(IVec2::new(63, 63)));
    /// assert!(!map.is_position_loaded(IVec2::new(64, 0)));
    /// assert!(!map.is_position_loaded(IVec2::new(-1, 0)));
    /// ```
    #[inline(always)]
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub const fn is_position_loaded(&self, position: IVec2) -> bool {
        let local_x = position.x.wrapping_sub(self.origin.x);
        let local_y = position.y.wrapping_sub(self.origin.y);
        local_x >= 0 && local_x < self.width as i32 && local_y >= 0 && local_y < self.height as i32
    }

    /// Convert a world position to an array index
    #[inline(always)]
    #[must_use]
    const fn index(&self, world_pos: IVec2) -> usize {
        // Equivalent to world_pos.xy % self.width (I have to leave these notes for myself because
        // I'm a dummy)
        let wrapped_x = (world_pos.x as u32) & self.width_mask;
        let wrapped_y = (world_pos.y as u32) & self.height_mask;
        // wrapped_y * self.width + wrapped_x
        ((wrapped_y << self.width_shift) | wrapped_x) as usize
    }

    /// Get a reference to the value at a world position.
    ///
    /// # Errors
    /// Returns `Err(PositionUnloaded)` if the position is outside the loaded region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(5, 5), 42).unwrap();
    ///
    /// assert_eq!(map.get(IVec2::new(5, 5)).unwrap(), Some(&42));
    /// assert_eq!(map.get(IVec2::new(0, 0)).unwrap(), None);
    /// assert!(map.get(IVec2::new(100, 0)).is_err());
    /// ```
    #[inline(always)]
    pub fn get(&self, position: IVec2) -> Result<Option<&T>, PositionUnloaded> {
        if self.is_position_loaded(position) {
            Ok(self.data[self.index(position)].as_ref())
        } else {
            Err(PositionUnloaded(position))
        }
    }

    /// Get a mutable reference to the value at a world position.
    ///
    /// # Errors
    /// Returns `Err(PositionUnloaded)` if the position is outside the loaded region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(5, 5), 42).unwrap();
    ///
    /// if let Some(value) = map.get_mut(IVec2::new(5, 5)).unwrap() {
    ///     *value = 99;
    /// }
    /// assert_eq!(map.get(IVec2::new(5, 5)).unwrap(), Some(&99));
    /// ```
    #[inline(always)]
    pub fn get_mut(&mut self, position: IVec2) -> Result<Option<&mut T>, PositionUnloaded> {
        if self.is_position_loaded(position) {
            let idx = self.index(position);
            Ok(self.data[idx].as_mut())
        } else {
            Err(PositionUnloaded(position))
        }
    }

    /// Get a reference to the value at a world position without bounds checking.
    ///
    /// # Safety
    /// Caller must ensure position is within the loaded region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(5, 5), 42).unwrap();
    ///
    /// // SAFETY: (5, 5) is within the loaded region [0..64, 0..64].
    /// let value = unsafe { map.get_unchecked(IVec2::new(5, 5)) };
    /// assert_eq!(*value, 42);
    /// ```
    #[inline(always)]
    #[must_use]
    pub unsafe fn get_unchecked(&self, position: IVec2) -> &T {
        self.data
            .get_unchecked(self.index(position))
            .as_ref()
            .unwrap_unchecked()
    }

    /// Get a mutable reference to the value at a world position without bounds checking.
    ///
    /// # Safety
    /// Caller must ensure position is within the loaded region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(5, 5), 42).unwrap();
    ///
    /// // SAFETY: (5, 5) is within the loaded region [0..64, 0..64].
    /// unsafe { *map.get_unchecked_mut(IVec2::new(5, 5)) = 99; }
    /// assert_eq!(map.get(IVec2::new(5, 5)).unwrap(), Some(&99));
    /// ```
    #[inline(always)]
    pub unsafe fn get_unchecked_mut(&mut self, position: IVec2) -> &mut T {
        let idx = self.index(position);
        self.data.get_unchecked_mut(idx).as_mut().unwrap_unchecked()
    }

    /// Insert a value at a world position, returning the previous value.
    ///
    /// # Errors
    /// Returns `Err(PositionUnloaded)` if the position is outside the loaded region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    ///
    /// assert_eq!(map.insert(IVec2::new(5, 5), 42).unwrap(), None);
    /// assert_eq!(map.insert(IVec2::new(5, 5), 99).unwrap(), Some(42));
    /// assert!(map.insert(IVec2::new(100, 0), 1).is_err());
    /// ```
    #[inline(always)]
    pub fn insert(&mut self, position: IVec2, value: T) -> Result<Option<T>, PositionUnloaded> {
        if self.is_position_loaded(position) {
            let idx = self.index(position);
            Ok(self.data[idx].replace(value))
        } else {
            Err(PositionUnloaded(position))
        }
    }

    /// Swap the values at two positions.
    ///
    /// # Errors
    /// Returns `Err(PositionUnloaded)` if either position is outside the loaded region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<&str>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(0, 0), "a").unwrap();
    /// map.insert(IVec2::new(1, 0), "b").unwrap();
    ///
    /// map.swap(IVec2::new(0, 0), IVec2::new(1, 0)).unwrap();
    ///
    /// assert_eq!(map.get(IVec2::new(0, 0)).unwrap(), Some(&"b"));
    /// assert_eq!(map.get(IVec2::new(1, 0)).unwrap(), Some(&"a"));
    /// ```
    #[inline(always)]
    pub fn swap(&mut self, first: IVec2, second: IVec2) -> Result<(), PositionUnloaded> {
        if !self.is_position_loaded(first) {
            return Err(PositionUnloaded(first));
        }
        if !self.is_position_loaded(second) {
            return Err(PositionUnloaded(second));
        }
        let idx1 = self.index(first);
        let idx2 = self.index(second);
        self.data.swap(idx1, idx2);
        Ok(())
    }

    /// Get an entry for conditional operations at a position.
    ///
    /// # Errors
    /// Returns `Err(PositionUnloaded)` if the position is outside the loaded region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    ///
    /// let mut entry = map.entry(IVec2::new(5, 5)).unwrap();
    /// assert!(entry.is_vacant());
    /// entry.set(42);
    /// assert!(entry.is_occupied());
    /// assert_eq!(entry.get(), Some(&42));
    /// ```
    #[inline(always)]
    pub fn entry(&mut self, position: IVec2) -> Result<SpatialEntry<'_, T>, PositionUnloaded> {
        if self.is_position_loaded(position) {
            let idx = self.index(position);
            Ok(SpatialEntry {
                slot: &mut self.data[idx],
            })
        } else {
            Err(PositionUnloaded(position))
        }
    }

    /// Shift the origin to a new position.
    ///
    /// This does not copy data - it just changes how indices map to world positions.
    /// Caller is responsible for handling the data in positions that are no longer
    /// valid (unloading) and populating new positions (loading).
    ///
    /// Returns the old and new regions for the caller to handle loading/unloading.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// let (old_region, new_region) = map.shift_origin(IVec2::new(1, 0));
    ///
    /// assert_eq!(old_region.min, IVec2::new(0, 0));
    /// assert_eq!(new_region.min, IVec2::new(1, 0));
    /// assert_eq!(map.origin(), IVec2::new(1, 0));
    /// ```
    #[allow(clippy::cast_possible_wrap)]
    pub fn shift_origin(&mut self, new_origin: IVec2) -> (IRect, IRect) {
        let old_region = self.loaded_region();
        self.origin = new_origin;
        let new_region = self.loaded_region();
        (old_region, new_region)
    }

    /// Clear all data in the specified region (set to default).
    ///
    /// Clears the array indices that correspond to the given world region.
    /// Does not check if positions are currently loaded, allowing cleanup
    /// of regions after an origin shift.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(0, 0), 1).unwrap();
    /// map.insert(IVec2::new(1, 1), 2).unwrap();
    ///
    /// map.clear_region(IRect::new(0, 0, 1, 1));
    ///
    /// assert_eq!(map.get(IVec2::new(0, 0)).unwrap(), None);
    /// assert_eq!(map.get(IVec2::new(1, 1)).unwrap(), None);
    /// ```
    pub fn clear_region(&mut self, region: IRect) {
        for y in region.min.y..=region.max.y {
            for x in region.min.x..=region.max.x {
                let pos = IVec2::new(x, y);
                let idx = self.index(pos);
                self.data[idx] = None;
            }
        }
    }

    /// Take all values from the specified region, clearing each slot.
    ///
    /// Like [`clear_region`](Self::clear_region), this bypasses the loaded-position
    /// check so it can drain regions that are no longer within the current origin.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(0, 0), 10).unwrap();
    /// map.insert(IVec2::new(1, 0), 20).unwrap();
    ///
    /// let drained: Vec<u32> = map.drain_region(IRect::new(0, 0, 1, 0)).collect();
    /// assert_eq!(drained, vec![10, 20]);
    /// assert_eq!(map.get(IVec2::new(0, 0)).unwrap(), None);
    /// ```
    pub fn drain_region(&mut self, region: IRect) -> impl Iterator<Item = T> + '_ {
        (region.min.y..=region.max.y)
            .flat_map(move |y| (region.min.x..=region.max.x).map(move |x| IVec2::new(x, y)))
            .filter_map(|pos| {
                let idx = self.index(pos);
                self.data[idx].take()
            })
    }

    /// Clear all data in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(0, 0), 42).unwrap();
    /// map.clear();
    /// assert!(map.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.data.fill(None);
    }

    /// Iterate over all positions and values in the loaded region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(4, 4, IVec2::ZERO);
    /// map.insert(IVec2::new(0, 0), 1).unwrap();
    ///
    /// let occupied: Vec<_> = map.iter()
    ///     .filter(|(_, v)| v.is_some())
    ///     .collect();
    /// assert_eq!(occupied.len(), 1);
    /// ```
    #[allow(clippy::cast_possible_wrap)]
    pub fn iter(&self) -> impl Iterator<Item = (IVec2, &Option<T>)> {
        let origin = self.origin;
        let width = self.width as i32;
        let height = self.height as i32;
        let width_mask = self.width_mask;
        let height_mask = self.height_mask;
        let width_shift = self.width_shift;

        (0..height).flat_map(move |y| {
            (0..width).map(move |x| {
                let pos = IVec2::new(origin.x + x, origin.y + y);
                // pos.xy % width/height
                let wrapped_x = (pos.x as u32) & width_mask;
                let wrapped_y = (pos.y as u32) & height_mask;
                // wrapped_y * width + wrawpped_x
                let idx = ((wrapped_y << width_shift) | wrapped_x) as usize;
                (pos, &self.data[idx])
            })
        })
    }

    /// Check if a value exists at the given position.
    ///
    /// # Errors
    /// Returns `Err(PositionUnloaded)` if the position is outside the loaded region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(5, 5), 42).unwrap();
    ///
    /// assert_eq!(map.contains(IVec2::new(5, 5)).unwrap(), true);
    /// assert_eq!(map.contains(IVec2::new(0, 0)).unwrap(), false);
    /// ```
    #[inline(always)]
    pub fn contains(&self, position: IVec2) -> Result<bool, PositionUnloaded> {
        self.get(position).map(|opt| opt.is_some())
    }

    /// Remove the value at a position.
    ///
    /// # Errors
    /// Returns `Err(PositionUnloaded)` if the position is outside the loaded region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(5, 5), 42).unwrap();
    ///
    /// assert_eq!(map.remove(IVec2::new(5, 5)).unwrap(), Some(42));
    /// assert_eq!(map.remove(IVec2::new(5, 5)).unwrap(), None);
    /// ```
    #[inline(always)]
    pub fn remove(&mut self, position: IVec2) -> Result<Option<T>, PositionUnloaded> {
        if self.is_position_loaded(position) {
            let idx = self.index(position);
            Ok(self.data[idx].take())
        } else {
            Err(PositionUnloaded(position))
        }
    }

    /// Returns `true` if the map contains no values in the loaded region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// assert!(map.is_empty());
    ///
    /// map.insert(IVec2::new(0, 0), 1).unwrap();
    /// assert!(!map.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.iter().all(std::option::Option::is_none)
    }

    /// Iterate over all occupied positions and their values.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(0, 0), 10).unwrap();
    /// map.insert(IVec2::new(1, 1), 20).unwrap();
    ///
    /// let occupied: Vec<_> = map.iter_occupied().collect();
    /// assert_eq!(occupied.len(), 2);
    /// ```
    pub fn iter_occupied(&self) -> impl Iterator<Item = (IVec2, &T)> + '_ {
        self.iter()
            .filter_map(|(pos, opt)| opt.as_ref().map(|v| (pos, v)))
    }
}

impl<T: Clone + Copy> SpatialMap<T> {
    /// Find all values within a rectangular region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(5, 5), 1).unwrap();
    /// map.insert(IVec2::new(6, 6), 2).unwrap();
    /// map.insert(IVec2::new(30, 30), 3).unwrap();
    ///
    /// let rect = IRect::new(4, 4, 7, 7);
    /// let found: Vec<_> = map.within_rect(rect).collect();
    /// assert_eq!(found.len(), 2);
    /// ```
    pub fn within_rect(&self, rect: IRect) -> impl Iterator<Item = (IVec2, T)> + '_ {
        self.within_rect_impl(rect.min, rect.max)
    }

    #[inline(always)]
    pub(crate) fn within_rect_impl(
        &self,
        min_pos: IVec2,
        max_pos: IVec2,
    ) -> impl Iterator<Item = (IVec2, T)> + '_ {
        let loaded = self.loaded_region();
        let clamped_min = min_pos.max(loaded.min);
        let clamped_max = max_pos.min(loaded.max);

        (clamped_min.y..=clamped_max.y).flat_map(move |y| {
            (clamped_min.x..=clamped_max.x).filter_map(move |x| {
                let pos = IVec2::new(x, y);
                self.get(pos).ok().flatten().map(|&v| (pos, v))
            })
        })
    }

    /// Find all values within a circular radius of a center position.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(5, 5), 1).unwrap();
    /// map.insert(IVec2::new(6, 5), 2).unwrap();
    /// map.insert(IVec2::new(20, 20), 3).unwrap();
    ///
    /// let nearby: Vec<_> = map.within_radius(IVec2::new(5, 5), 2.0).collect();
    /// assert_eq!(nearby.len(), 2);
    /// ```
    #[inline(always)]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn within_radius(
        &self,
        center: IVec2,
        radius: f32,
    ) -> impl Iterator<Item = (IVec2, T)> + '_ {
        let radius_i32 = radius.ceil() as i32;
        let min_pos = center - IVec2::splat(radius_i32);
        let max_pos = center + IVec2::splat(radius_i32);
        let radius_squared = radius * radius;

        self.within_rect_impl(min_pos, max_pos)
            .filter(move |(pos, _)| {
                let diff = *pos - center;
                let dist_sq = (diff.x * diff.x + diff.y * diff.y) as f32;
                dist_sq <= radius_squared
            })
    }

    /// Get a copy of the value at a position if one exists.
    ///
    /// # Errors
    /// Returns `Err(PositionUnloaded)` if the position is outside the loaded region.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(5, 5), 42).unwrap();
    ///
    /// assert_eq!(map.get_copied(IVec2::new(5, 5)).unwrap(), Some(42));
    /// assert_eq!(map.get_copied(IVec2::new(0, 0)).unwrap(), None);
    /// ```
    #[inline(always)]
    pub fn get_copied(&self, position: IVec2) -> Result<Option<T>, PositionUnloaded> {
        #[allow(clippy::redundant_closure_for_method_calls)]
        self.get(position).map(|opt| opt.copied())
    }

    /// Iterate over all occupied positions and their values by copy.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(0, 0), 1).unwrap();
    /// map.insert(IVec2::new(1, 1), 2).unwrap();
    ///
    /// let count = map.iter_copied().count();
    /// assert_eq!(count, 2);
    /// ```
    pub fn iter_copied(&self) -> impl Iterator<Item = (IVec2, T)> + '_ {
        self.iter_occupied().map(|(pos, &v)| (pos, v))
    }

    /// Cast a ray from `from` toward `to`, returning the first value that satisfies
    /// `is_hit` along with its position.
    ///
    /// Uses a Bresenham-style ray march. The `from` and `to` endpoints are excluded
    /// from hit testing.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(5, 5), 99).unwrap();
    ///
    /// let hit = map.raycast(IVec2::new(0, 0), IVec2::new(10, 10), &|_| true);
    /// assert_eq!(hit, Some((IVec2::new(5, 5), 99)));
    ///
    /// let miss = map.raycast(IVec2::new(0, 0), IVec2::new(10, 10), &|_| false);
    /// assert_eq!(miss, None);
    /// ```
    #[inline(always)]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    #[must_use]
    pub fn raycast<F>(&self, from: IVec2, to: IVec2, is_hit: &F) -> Option<(IVec2, T)>
    where
        F: Fn(T) -> bool,
    {
        if from == to {
            return None;
        }

        let diff = to - from;
        let steps = diff.x.abs().max(diff.y.abs());

        if steps <= 1 {
            return None;
        }

        let step_x = diff.x as f32 / steps as f32;
        let step_y = diff.y as f32 / steps as f32;

        let mut x = from.x as f32 + 0.5;
        let mut y = from.y as f32 + 0.5;

        for _ in 1..steps {
            x += step_x;
            y += step_y;

            let check_pos = IVec2::new(x.floor() as i32, y.floor() as i32);

            if check_pos == from {
                continue;
            }

            if let Ok(Some(value)) = self.get_copied(check_pos) {
                if is_hit(value) {
                    return Some((check_pos, value));
                }
            }
        }

        None
    }

    /// Check line of sight between two positions using a predicate.
    ///
    /// Returns `true` if no value along the path satisfies `is_blocker`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// let clear = map.has_line_of_sight_by(
    ///     IVec2::new(0, 0),
    ///     IVec2::new(10, 10),
    ///     &|_val| false,
    /// );
    /// assert!(clear);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn has_line_of_sight_by<F>(&self, from: IVec2, to: IVec2, is_blocker: &F) -> bool
    where
        F: Fn(T) -> bool,
    {
        self.raycast(from, to, is_blocker).is_none()
    }

    /// Find all values within radius that have line-of-sight to center, using a
    /// custom predicate.
    ///
    /// Values blocked by entries satisfying `is_blocker` are excluded.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// let visible = map.within_radius_los_by(
    ///     IVec2::new(10, 10),
    ///     5.0,
    ///     |_val| false,
    /// );
    /// assert!(visible.is_empty());
    /// ```
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    #[inline(always)]
    #[must_use]
    pub fn within_radius_los_by<F>(
        &self,
        center: IVec2,
        radius: f32,
        is_blocker: F,
    ) -> Vec<(IVec2, T)>
    where
        F: Fn(T) -> bool,
    {
        let radius_i32 = radius.ceil() as i32;
        let min_pos = center - IVec2::splat(radius_i32);
        let max_pos = center + IVec2::splat(radius_i32);
        let radius_squared = radius * radius;

        self.within_rect_impl(min_pos, max_pos)
            .filter(|(pos, _)| {
                let diff = *pos - center;
                let dist_sq = (diff.x * diff.x + diff.y * diff.y) as f32;
                dist_sq <= radius_squared
            })
            .filter(|(pos, _)| self.has_line_of_sight_by(center, *pos, &is_blocker))
            .collect()
    }
}

/// Entry for a position that may or may not contain a value.
///
/// Obtained via [`SpatialMap::entry`]. Provides conditional insert and update operations
/// without requiring multiple lookups.
///
/// # Examples
///
/// ```
/// use bevy::prelude::*;
/// use bevy_falling_sand::core::SpatialMap;
///
/// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
///
/// let mut entry = map.entry(IVec2::new(0, 0)).unwrap();
/// assert!(entry.is_vacant());
///
/// entry.set(42);
/// assert!(entry.is_occupied());
/// assert_eq!(entry.get(), Some(&42));
///
/// let old = entry.take();
/// assert_eq!(old, Some(42));
/// assert!(entry.is_vacant());
/// ```
pub struct SpatialEntry<'a, T> {
    slot: &'a mut Option<T>,
}

impl<T> SpatialEntry<'_, T> {
    /// Get a reference to the current value if present.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(0, 0), 42).unwrap();
    ///
    /// let entry = map.entry(IVec2::new(0, 0)).unwrap();
    /// assert_eq!(entry.get(), Some(&42));
    /// ```
    #[must_use]
    pub const fn get(&self) -> Option<&T> {
        self.slot.as_ref()
    }

    /// Get a mutable reference to the current value if present.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(0, 0), 42).unwrap();
    ///
    /// let mut entry = map.entry(IVec2::new(0, 0)).unwrap();
    /// if let Some(val) = entry.get_mut() {
    ///     *val = 99;
    /// }
    /// assert_eq!(entry.get(), Some(&99));
    /// ```
    pub const fn get_mut(&mut self) -> Option<&mut T> {
        self.slot.as_mut()
    }

    /// Set the value, returning the previous value.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// let mut entry = map.entry(IVec2::new(0, 0)).unwrap();
    ///
    /// assert_eq!(entry.set(42), None);
    /// assert_eq!(entry.set(99), Some(42));
    /// assert_eq!(entry.get(), Some(&99));
    /// ```
    pub const fn set(&mut self, value: T) -> Option<T> {
        self.slot.replace(value)
    }

    /// Take the value out of the slot, leaving None.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(0, 0), 42).unwrap();
    ///
    /// let mut entry = map.entry(IVec2::new(0, 0)).unwrap();
    /// assert_eq!(entry.take(), Some(42));
    /// assert!(entry.is_vacant());
    /// ```
    pub const fn take(&mut self) -> Option<T> {
        self.slot.take()
    }

    /// Check if the slot is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// let entry = map.entry(IVec2::new(0, 0)).unwrap();
    /// assert!(entry.is_vacant());
    /// ```
    #[must_use]
    pub const fn is_vacant(&self) -> bool {
        self.slot.is_none()
    }

    /// Check if the slot is occupied.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// map.insert(IVec2::new(0, 0), 42).unwrap();
    ///
    /// let entry = map.entry(IVec2::new(0, 0)).unwrap();
    /// assert!(entry.is_occupied());
    /// ```
    #[must_use]
    pub const fn is_occupied(&self) -> bool {
        self.slot.is_some()
    }

    /// Insert a value if the slot is empty (None).
    ///
    /// Returns `true` if inserted, `false` if occupied.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// let mut entry = map.entry(IVec2::new(0, 0)).unwrap();
    ///
    /// assert!(entry.insert_if_vacant(42));
    /// assert!(!entry.insert_if_vacant(99));
    /// assert_eq!(entry.get(), Some(&42));
    /// ```
    pub fn insert_if_vacant(&mut self, value: T) -> bool {
        if self.slot.is_none() {
            *self.slot = Some(value);
            true
        } else {
            false
        }
    }

    /// Insert a value using a closure if the slot is empty.
    ///
    /// Returns `true` if inserted, `false` if occupied.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// let mut entry = map.entry(IVec2::new(0, 0)).unwrap();
    ///
    /// assert!(entry.insert_if_vacant_with(|| 42));
    /// assert_eq!(entry.get(), Some(&42));
    /// ```
    pub fn insert_if_vacant_with<F>(&mut self, f: F) -> bool
    where
        F: FnOnce() -> T,
    {
        if self.slot.is_none() {
            *self.slot = Some(f());
            true
        } else {
            false
        }
    }

    /// Get a reference to the value, inserting a default if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// let mut entry = map.entry(IVec2::new(0, 0)).unwrap();
    ///
    /// let value = entry.or_insert(100);
    /// assert_eq!(*value, 100);
    /// ```
    pub fn or_insert(&mut self, default: T) -> &mut T {
        self.slot.get_or_insert(default)
    }

    /// Get a reference to the value, inserting with a closure if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SpatialMap;
    ///
    /// let mut map = SpatialMap::<u32>::new(64, 64, IVec2::ZERO);
    /// let mut entry = map.entry(IVec2::new(0, 0)).unwrap();
    ///
    /// let value = entry.or_insert_with(|| 42);
    /// assert_eq!(*value, 42);
    /// ```
    pub fn or_insert_with<F>(&mut self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        self.slot.get_or_insert_with(f)
    }
}
