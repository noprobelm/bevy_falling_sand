use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use crate::core::{ChunkCoord, ChunkIndex, ChunkSystems};

use super::ChunkRegion;

pub(super) struct DirtyTrackingPlugin;

impl Plugin for DirtyTrackingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            advance_chunk_dirty_state.in_set(ChunkSystems::DirtyAdvance),
        );
    }
}

/// Border flag for the north edge.
pub const BORDER_N: u8 = 0b0000_0001;
/// Border flag for the northeast corner.
pub const BORDER_NE: u8 = 0b0000_0010;
/// Border flag for the east edge.
pub const BORDER_E: u8 = 0b0000_0100;
/// Border flag for the southeast corner.
pub const BORDER_SE: u8 = 0b0000_1000;
/// Border flag for the south edge.
pub const BORDER_S: u8 = 0b0001_0000;
/// Border flag for the southwest corner.
pub const BORDER_SW: u8 = 0b0010_0000;
/// Border flag for the west edge.
pub const BORDER_W: u8 = 0b0100_0000;
/// Border flag for the northwest corner.
pub const BORDER_NW: u8 = 0b1000_0000;

/// Tracks dirty regions for a chunk.
///
/// Contains both the current frame's dirty rect (from previous frame's mutations)
/// and the next frame's dirty rect (being built from current mutations).
///
/// Border flags indicate which edges of the chunk need a 2-pixel strip scan
/// due to activity in neighboring chunks.
#[derive(Component, Clone, Debug, Default, Reflect)]
#[require(ChunkRegion)]
pub struct ChunkDirtyState {
    /// The dirty region for the current frame (computed from previous frame).
    /// This is the region that needs to be processed/rendered this frame.
    pub current: Option<IRect>,
    /// The dirty region being built for the next frame.
    /// As types are added/removed/moved, this accumulates the affected region.
    pub next: Option<IRect>,
    /// Specific positions that changed for the current frame's rendering.
    /// `None` means the chunk is fully dirty and the entire `current` rect should be iterated.
    /// `Some(set)` means only these positions need pixel updates.
    #[reflect(ignore)]
    pub current_positions: Option<HashSet<IVec2>>,
    /// Positions being accumulated for next frame's rendering.
    /// `None` means the chunk will be fully dirty next frame.
    #[reflect(ignore)]
    pub next_positions: Option<HashSet<IVec2>>,
    /// Bitflags indicating which border strips need scanning this frame.
    pub dirty_borders: u8,
    /// Bitflags being accumulated for next frame's border scanning.
    pub next_dirty_borders: u8,
}

impl ChunkDirtyState {
    /// Creates a new empty dirty state.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_falling_sand::core::ChunkDirtyState;
    ///
    /// let state = ChunkDirtyState::new();
    /// assert!(!state.is_dirty());
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            current: None,
            next: None,
            current_positions: Some(HashSet::new()),
            next_positions: Some(HashSet::new()),
            dirty_borders: 0,
            next_dirty_borders: 0,
        }
    }

    /// Creates a dirty state where the entire region is marked dirty.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ChunkDirtyState;
    ///
    /// let state = ChunkDirtyState::fully_dirty(IRect::new(0, 0, 31, 31));
    /// assert!(state.is_dirty());
    /// assert_eq!(state.current, Some(IRect::new(0, 0, 31, 31)));
    /// ```
    #[must_use]
    pub const fn fully_dirty(region: IRect) -> Self {
        Self {
            current: Some(region),
            next: Some(region),
            current_positions: None,
            next_positions: None,
            dirty_borders: 0xFF,
            next_dirty_borders: 0xFF,
        }
    }

    /// Returns `true` if this chunk has a dirty rect or any border flags set.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ChunkDirtyState;
    ///
    /// let clean = ChunkDirtyState::new();
    /// assert!(!clean.is_dirty());
    ///
    /// let dirty = ChunkDirtyState::fully_dirty(IRect::new(0, 0, 31, 31));
    /// assert!(dirty.is_dirty());
    /// ```
    #[must_use]
    pub const fn is_dirty(&self) -> bool {
        self.current.is_some() || self.dirty_borders != 0
    }

    /// Resets all dirty state for both current and next frames.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ChunkDirtyState;
    ///
    /// let mut state = ChunkDirtyState::fully_dirty(IRect::new(0, 0, 31, 31));
    /// state.clear();
    /// assert!(!state.is_dirty());
    /// ```
    pub fn clear(&mut self) {
        self.current = None;
        self.next = None;
        self.current_positions = Some(HashSet::new());
        self.next_positions = Some(HashSet::new());
        self.dirty_borders = 0;
        self.next_dirty_borders = 0;
    }

    /// Resets only the current frame's dirty state.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ChunkDirtyState;
    ///
    /// let mut state = ChunkDirtyState::fully_dirty(IRect::new(0, 0, 31, 31));
    /// state.clear_current();
    /// assert!(state.current.is_none());
    /// assert!(state.next.is_some());
    /// ```
    pub fn clear_current(&mut self) {
        self.current = None;
        self.current_positions = Some(HashSet::new());
        self.dirty_borders = 0;
    }

    /// Rotates next-frame state into current, inflating the dirty rect by 2 pixels.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ChunkDirtyState;
    ///
    /// let mut state = ChunkDirtyState::new();
    /// state.mark_dirty(IVec2::new(5, 5));
    /// assert!(state.current.is_none());
    ///
    /// state.advance_frame();
    /// assert!(state.current.is_some());
    /// assert!(state.next.is_none());
    ///
    /// let current = state.current.unwrap();
    /// assert_eq!(current.min, IVec2::new(3, 3));
    /// assert_eq!(current.max, IVec2::new(7, 7));
    /// ```
    pub fn advance_frame(&mut self) {
        self.current = self.next.take().map(|rect| rect.inflate(2));
        self.current_positions = self.next_positions.take();
        self.next_positions = Some(HashSet::new());
        self.dirty_borders = self.next_dirty_borders;
        self.next_dirty_borders = 0;
    }

    /// Sets border flags for next frame's border strip scanning.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_falling_sand::core::{ChunkDirtyState, BORDER_N, BORDER_E};
    ///
    /// let mut state = ChunkDirtyState::new();
    /// state.mark_border(BORDER_N | BORDER_E);
    /// assert_eq!(state.next_dirty_borders, BORDER_N | BORDER_E);
    /// ```
    pub const fn mark_border(&mut self, flags: u8) {
        self.next_dirty_borders |= flags;
    }

    /// Expand the next dirty rect to include a point and track the position for rendering.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ChunkDirtyState;
    ///
    /// let mut state = ChunkDirtyState::new();
    /// state.mark_dirty(IVec2::new(5, 5));
    /// state.mark_dirty(IVec2::new(10, 15));
    ///
    /// let rect = state.next.unwrap();
    /// assert!(rect.contains(IVec2::new(5, 5)));
    /// assert!(rect.contains(IVec2::new(10, 15)));
    /// ```
    pub fn mark_dirty(&mut self, point: IVec2) {
        match &mut self.next {
            Some(rect) => *rect = rect.union_point(point),
            None => self.next = Some(IRect::from_center_size(point, IVec2::ONE)),
        }
        if let Some(ref mut positions) = self.next_positions {
            positions.insert(point);
        }
        if let Some(ref mut positions) = self.current_positions {
            positions.insert(point);
        }
    }

    /// Returns all positions in the 2-pixel-wide border strips indicated by `dirty_borders`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{ChunkDirtyState, BORDER_N};
    ///
    /// let mut state = ChunkDirtyState::new();
    /// state.dirty_borders = BORDER_N;
    ///
    /// let chunk_rect = IRect::new(0, 0, 31, 31);
    /// let positions = state.border_strip_positions(chunk_rect);
    /// assert!(positions.iter().all(|p| p.y >= 30));
    /// ```
    #[must_use]
    pub fn border_strip_positions(&self, chunk_rect: IRect) -> Vec<IVec2> {
        let flags = self.dirty_borders;
        if flags == 0 {
            return Vec::new();
        }

        let mut positions = Vec::new();
        let min = chunk_rect.min;
        let max = chunk_rect.max;

        if flags & BORDER_N != 0 {
            for y in (max.y - 1)..=max.y {
                for x in min.x..=max.x {
                    positions.push(IVec2::new(x, y));
                }
            }
        }
        if flags & BORDER_S != 0 {
            for y in min.y..=(min.y + 1) {
                for x in min.x..=max.x {
                    positions.push(IVec2::new(x, y));
                }
            }
        }
        if flags & BORDER_E != 0 {
            for y in min.y..=max.y {
                for x in (max.x - 1)..=max.x {
                    positions.push(IVec2::new(x, y));
                }
            }
        }
        if flags & BORDER_W != 0 {
            for y in min.y..=max.y {
                for x in min.x..=(min.x + 1) {
                    positions.push(IVec2::new(x, y));
                }
            }
        }
        if flags & BORDER_NE != 0 {
            for y in (max.y - 1)..=max.y {
                for x in (max.x - 1)..=max.x {
                    positions.push(IVec2::new(x, y));
                }
            }
        }
        if flags & BORDER_NW != 0 {
            for y in (max.y - 1)..=max.y {
                for x in min.x..=(min.x + 1) {
                    positions.push(IVec2::new(x, y));
                }
            }
        }
        if flags & BORDER_SE != 0 {
            for y in min.y..=(min.y + 1) {
                for x in (max.x - 1)..=max.x {
                    positions.push(IVec2::new(x, y));
                }
            }
        }
        if flags & BORDER_SW != 0 {
            for y in min.y..=(min.y + 1) {
                for x in min.x..=(min.x + 1) {
                    positions.push(IVec2::new(x, y));
                }
            }
        }

        positions
    }

    /// Returns `true` if `pos` falls within any active border strip.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{ChunkDirtyState, BORDER_S};
    ///
    /// let mut state = ChunkDirtyState::new();
    /// state.dirty_borders = BORDER_S;
    ///
    /// let chunk_rect = IRect::new(0, 0, 31, 31);
    /// assert!(state.is_in_border_strip(IVec2::new(15, 0), chunk_rect));
    /// assert!(!state.is_in_border_strip(IVec2::new(15, 15), chunk_rect));
    /// ```
    #[allow(clippy::similar_names)]
    #[must_use]
    pub const fn is_in_border_strip(&self, pos: IVec2, chunk_rect: IRect) -> bool {
        let flags = self.dirty_borders;
        if flags == 0 {
            return false;
        }

        let min = chunk_rect.min;
        let max = chunk_rect.max;

        let in_top_2 = pos.y >= max.y - 1 && pos.y <= max.y;
        let in_bottom_2 = pos.y >= min.y && pos.y <= min.y + 1;
        let in_left_2 = pos.x >= min.x && pos.x <= min.x + 1;
        let in_right_2 = pos.x >= max.x - 1 && pos.x <= max.x;
        let in_x_range = pos.x >= min.x && pos.x <= max.x;
        let in_y_range = pos.y >= min.y && pos.y <= max.y;

        (flags & BORDER_N != 0 && in_top_2 && in_x_range)
            || (flags & BORDER_S != 0 && in_bottom_2 && in_x_range)
            || (flags & BORDER_E != 0 && in_right_2 && in_y_range)
            || (flags & BORDER_W != 0 && in_left_2 && in_y_range)
            || (flags & BORDER_NE != 0 && in_top_2 && in_right_2)
            || (flags & BORDER_NW != 0 && in_top_2 && in_left_2)
            || (flags & BORDER_SE != 0 && in_bottom_2 && in_right_2)
            || (flags & BORDER_SW != 0 && in_bottom_2 && in_left_2)
    }

    /// Expand the next dirty rect to include a region.
    /// Marks the chunk as fully dirty for rendering (position tracking is lost).
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ChunkDirtyState;
    ///
    /// let mut state = ChunkDirtyState::new();
    /// state.mark_dirty_rect(IRect::new(0, 0, 10, 10));
    ///
    /// assert_eq!(state.next, Some(IRect::new(0, 0, 10, 10)));
    /// assert!(state.next_positions.is_none());
    /// ```
    pub fn mark_dirty_rect(&mut self, region: IRect) {
        match &mut self.next {
            Some(rect) => *rect = rect.union(region),
            None => self.next = Some(region),
        }
        self.next_positions = None;
    }
}

#[allow(clippy::needless_pass_by_value)]
fn advance_chunk_dirty_state(
    mut chunk_query: Query<(Entity, &ChunkRegion, &mut ChunkDirtyState)>,
    chunk_index: Res<ChunkIndex>,
) {
    let mut border_notifications: Vec<(ChunkCoord, u8)> = Vec::new();

    for (_, region, mut dirty_state) in &mut chunk_query {
        dirty_state.advance_frame();

        if let Some(current) = dirty_state.current {
            let chunk_rect = region.region();
            let w = chunk_rect.width() + 1;
            let h = chunk_rect.height() + 1;

            let extends_left = current.min.x < chunk_rect.min.x;
            let extends_right = current.max.x > chunk_rect.max.x;
            let extends_down = current.min.y < chunk_rect.min.y;
            let extends_up = current.max.y > chunk_rect.max.y;

            if extends_left || extends_right || extends_down || extends_up {
                if extends_up {
                    let coord = chunk_index
                        .world_to_chunk_coord(IVec2::new(chunk_rect.min.x, chunk_rect.min.y + h));
                    border_notifications.push((coord, BORDER_S));
                }
                if extends_down {
                    let coord = chunk_index
                        .world_to_chunk_coord(IVec2::new(chunk_rect.min.x, chunk_rect.min.y - h));
                    border_notifications.push((coord, BORDER_N));
                }
                if extends_right {
                    let coord = chunk_index
                        .world_to_chunk_coord(IVec2::new(chunk_rect.min.x + w, chunk_rect.min.y));
                    border_notifications.push((coord, BORDER_W));
                }
                if extends_left {
                    let coord = chunk_index
                        .world_to_chunk_coord(IVec2::new(chunk_rect.min.x - w, chunk_rect.min.y));
                    border_notifications.push((coord, BORDER_E));
                }
                if extends_up && extends_right {
                    let coord = chunk_index.world_to_chunk_coord(IVec2::new(
                        chunk_rect.min.x + w,
                        chunk_rect.min.y + h,
                    ));
                    border_notifications.push((coord, BORDER_SW));
                }
                if extends_up && extends_left {
                    let coord = chunk_index.world_to_chunk_coord(IVec2::new(
                        chunk_rect.min.x - w,
                        chunk_rect.min.y + h,
                    ));
                    border_notifications.push((coord, BORDER_SE));
                }
                if extends_down && extends_right {
                    let coord = chunk_index.world_to_chunk_coord(IVec2::new(
                        chunk_rect.min.x + w,
                        chunk_rect.min.y - h,
                    ));
                    border_notifications.push((coord, BORDER_NW));
                }
                if extends_down && extends_left {
                    let coord = chunk_index.world_to_chunk_coord(IVec2::new(
                        chunk_rect.min.x - w,
                        chunk_rect.min.y - h,
                    ));
                    border_notifications.push((coord, BORDER_NE));
                }

                dirty_state.current = intersect_rects(current, chunk_rect);
            }
        }
    }

    for (coord, flags) in border_notifications {
        if let Some(chunk_entity) = chunk_index.get(coord) {
            if let Ok((_, _, mut dirty_state)) = chunk_query.get_mut(chunk_entity) {
                dirty_state.dirty_borders |= flags;
            }
        }
    }
}

#[inline]
fn intersect_rects(a: IRect, b: IRect) -> Option<IRect> {
    let min_x = a.min.x.max(b.min.x);
    let min_y = a.min.y.max(b.min.y);
    let max_x = a.max.x.min(b.max.x);
    let max_y = a.max.y.min(b.max.y);

    if min_x <= max_x && min_y <= max_y {
        Some(IRect::new(min_x, min_y, max_x, max_y))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_not_dirty() {
        let state = ChunkDirtyState::new();
        assert!(!state.is_dirty());
        assert!(state.current.is_none());
        assert!(state.next.is_none());
    }

    #[test]
    fn fully_dirty_sets_both() {
        let region = IRect::new(0, 0, 31, 31);
        let state = ChunkDirtyState::fully_dirty(region);

        assert!(state.is_dirty());
        assert_eq!(state.current, Some(region));
        assert_eq!(state.next, Some(region));
    }

    #[test]
    fn mark_dirty_creates_rect() {
        let mut state = ChunkDirtyState::new();
        state.mark_dirty(IVec2::new(5, 5));

        assert!(state.next.is_some());
        let rect = state.next.unwrap();
        assert!(rect.contains(IVec2::new(5, 5)));
    }

    #[test]
    fn mark_dirty_expands_rect() {
        let mut state = ChunkDirtyState::new();
        state.mark_dirty(IVec2::new(5, 5));
        state.mark_dirty(IVec2::new(10, 15));

        let rect = state.next.unwrap();
        assert!(rect.contains(IVec2::new(5, 5)));
        assert!(rect.contains(IVec2::new(10, 15)));
    }

    #[test]
    fn advance_frame_moves_next_to_current_with_inflation() {
        let mut state = ChunkDirtyState::new();
        state.mark_dirty(IVec2::new(5, 5));

        assert!(state.current.is_none());
        assert!(state.next.is_some());

        state.advance_frame();

        assert!(state.current.is_some());
        assert!(state.next.is_none());

        let current = state.current.unwrap();
        assert_eq!(current.min, IVec2::new(3, 3));
        assert_eq!(current.max, IVec2::new(7, 7));
    }

    #[test]
    fn advance_frame_inflates_chunk_border_positions() {
        let mut state = ChunkDirtyState::new();
        state.mark_dirty(IVec2::new(31, 10));

        state.advance_frame();

        let current = state.current.unwrap();
        assert!(current.contains(IVec2::new(32, 10)));
        assert!(current.contains(IVec2::new(30, 10)));
    }

    #[test]
    fn clear_removes_both() {
        let mut state = ChunkDirtyState::fully_dirty(IRect::new(0, 0, 31, 31));
        state.clear();

        assert!(!state.is_dirty());
        assert!(state.current.is_none());
        assert!(state.next.is_none());
    }
}
