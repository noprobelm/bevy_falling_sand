use bevy::prelude::*;

use crate::core::SpatialMap;

/// Alias for the particle spatial map.
///
/// This map is intended to store [`Particle`](crate::Particle) entities IDs
pub type ParticleMap = SpatialMap<Entity>;

impl ParticleMap {
    /// Find all particles within radius that have line-of-sight to center.
    ///
    /// Particles that are blocked by entities matching the `blocker_query` are excluded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ParticleMap;
    ///
    /// #[derive(Component)]
    /// struct Wall;
    ///
    /// fn find_visible(map: Res<ParticleMap>, walls: Query<(), With<Wall>>) {
    ///     let visible = map.within_radius_los(IVec2::new(10, 10), 5.0, &walls);
    ///     println!("Visible particles: {}", visible.len());
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn within_radius_los(
        &self,
        center: IVec2,
        radius: f32,
        blocker_query: &Query<(), impl bevy::ecs::query::QueryFilter>,
    ) -> Vec<(IVec2, Entity)> {
        self.within_radius_los_by(center, radius, |entity| blocker_query.get(entity).is_ok())
    }

    /// Check if there's a clear line of sight between two positions.
    ///
    /// Returns `true` if no entity matching the `blocker_query` exists along the path.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ParticleMap;
    ///
    /// #[derive(Component)]
    /// struct Wall;
    ///
    /// fn check_los(map: Res<ParticleMap>, walls: Query<(), With<Wall>>) {
    ///     let can_see = map.has_line_of_sight(
    ///         IVec2::new(0, 0),
    ///         IVec2::new(10, 10),
    ///         &walls,
    ///     );
    ///     println!("Line of sight: {can_see}");
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn has_line_of_sight(
        &self,
        from: IVec2,
        to: IVec2,
        blocker_query: &Query<(), impl bevy::ecs::query::QueryFilter>,
    ) -> bool {
        self.has_line_of_sight_by(from, to, &|entity| blocker_query.get(entity).is_ok())
    }

    /// Cast a ray from `from` toward `to`, returning the first entity that matches
    /// the `blocker_query` along with its position.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::ParticleMap;
    ///
    /// #[derive(Component)]
    /// struct Wall;
    ///
    /// fn find_wall(map: Res<ParticleMap>, walls: Query<(), With<Wall>>) {
    ///     if let Some((pos, entity)) = map.raycast_query(
    ///         IVec2::new(0, 0),
    ///         IVec2::new(10, 10),
    ///         &walls,
    ///     ) {
    ///         println!("Hit wall {entity:?} at {pos}");
    ///     }
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn raycast_query(
        &self,
        from: IVec2,
        to: IVec2,
        blocker_query: &Query<(), impl bevy::ecs::query::QueryFilter>,
    ) -> Option<(IVec2, Entity)> {
        self.raycast(from, to, &|entity| blocker_query.get(entity).is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_map() -> ParticleMap {
        ParticleMap::new(64, 64, IVec2::ZERO)
    }

    mod constructor {
        use super::*;

        #[test]
        fn new_creates_map_with_correct_dimensions() {
            let map = create_map();

            assert_eq!(map.width(), 64);
            assert_eq!(map.height(), 64);
            assert_eq!(map.origin(), IVec2::ZERO);
        }

        #[test]
        fn new_accepts_power_of_two() {
            for size in [32, 64, 128, 256, 512, 1024] {
                let map = ParticleMap::new(size, size, IVec2::ZERO);
                assert_eq!(map.width(), size);
            }
        }

        #[test]
        #[should_panic(expected = "power of 2")]
        fn new_panics_on_non_power_of_two_width() {
            let _ = ParticleMap::new(33, 64, IVec2::ZERO);
        }

        #[test]
        #[should_panic(expected = "power of 2")]
        fn new_panics_on_non_power_of_two_height() {
            let _ = ParticleMap::new(64, 33, IVec2::ZERO);
        }
    }

    mod loaded_region {
        use super::*;

        #[test]
        fn loaded_region_at_origin() {
            let map = create_map();
            let region = map.loaded_region();

            assert_eq!(region.min, IVec2::ZERO);
            assert_eq!(region.max, IVec2::new(63, 63));
        }

        #[test]
        fn loaded_region_with_offset_origin() {
            let map = ParticleMap::new(64, 64, IVec2::new(100, 200));
            let region = map.loaded_region();

            assert_eq!(region.min, IVec2::new(100, 200));
            assert_eq!(region.max, IVec2::new(163, 263));
        }

        #[test]
        fn is_position_loaded_inside() {
            let map = create_map();

            assert!(map.is_position_loaded(IVec2::new(0, 0)));
            assert!(map.is_position_loaded(IVec2::new(32, 32)));
            assert!(map.is_position_loaded(IVec2::new(63, 63)));
        }

        #[test]
        fn is_position_loaded_outside() {
            let map = create_map();

            assert!(!map.is_position_loaded(IVec2::new(-1, 0)));
            assert!(!map.is_position_loaded(IVec2::new(0, -1)));
            assert!(!map.is_position_loaded(IVec2::new(64, 0)));
            assert!(!map.is_position_loaded(IVec2::new(0, 64)));
        }
    }

    mod entity_operations {
        use super::*;

        #[test]
        fn insert_and_get() {
            let mut map = create_map();
            let entity = Entity::from_bits(42);
            let pos = IVec2::new(5, 5);

            let _ = map.insert(pos, entity);

            assert_eq!(map.get_copied(pos), Ok(Some(entity)));
            assert_eq!(map.contains(pos), Ok(true));
        }

        #[test]
        fn insert_returns_previous() {
            let mut map = create_map();
            let e1 = Entity::from_bits(1);
            let e2 = Entity::from_bits(2);
            let pos = IVec2::new(5, 5);

            let _ = map.insert(pos, e1);
            let prev = map.insert(pos, e2);

            assert_eq!(prev, Ok(Some(e1)));
            assert_eq!(map.get_copied(pos), Ok(Some(e2)));
        }

        #[test]
        fn insert_outside_region_fails() {
            let mut map = create_map();
            let entity = Entity::from_bits(42);

            let result = map.insert(IVec2::new(100, 100), entity);

            assert!(result.is_err());
        }

        #[test]
        fn remove_returns_entity() {
            let mut map = create_map();
            let entity = Entity::from_bits(42);
            let pos = IVec2::new(5, 5);

            let _ = map.insert(pos, entity);
            let removed = map.remove(pos);

            assert_eq!(removed, Ok(Some(entity)));
            assert_eq!(map.contains(pos), Ok(false));
        }

        #[test]
        fn remove_empty_returns_none() {
            let mut map = create_map();
            let removed = map.remove(IVec2::new(5, 5));

            assert_eq!(removed, Ok(None));
        }

        #[test]
        fn is_empty_on_new_map() {
            let map = create_map();
            assert!(map.is_empty());
        }

        #[test]
        fn is_empty_after_insert() {
            let mut map = create_map();
            let _ = map.insert(IVec2::new(5, 5), Entity::from_bits(1));

            assert!(!map.is_empty());
        }

        #[test]
        fn swap_positions() {
            let mut map = create_map();
            let e1 = Entity::from_bits(1);
            let e2 = Entity::from_bits(2);
            let pos1 = IVec2::new(5, 5);
            let pos2 = IVec2::new(10, 10);

            let _ = map.insert(pos1, e1);
            let _ = map.insert(pos2, e2);

            let result = map.swap(pos1, pos2);

            assert!(result.is_ok());
            assert_eq!(map.get_copied(pos1), Ok(Some(e2)));
            assert_eq!(map.get_copied(pos2), Ok(Some(e1)));
        }

        #[test]
        fn swap_with_empty() {
            let mut map = create_map();
            let entity = Entity::from_bits(1);
            let pos1 = IVec2::new(5, 5);
            let pos2 = IVec2::new(10, 10);

            let _ = map.insert(pos1, entity);

            let result = map.swap(pos1, pos2);

            assert!(result.is_ok());
            assert_eq!(map.get_copied(pos1), Ok(None));
            assert_eq!(map.get_copied(pos2), Ok(Some(entity)));
        }
    }

    mod toroidal_indexing {
        use super::*;

        #[test]
        fn shift_origin_updates_region() {
            let mut map = create_map();

            let (old_region, new_region) = map.shift_origin(IVec2::new(32, 32));

            assert_eq!(old_region.min, IVec2::ZERO);
            assert_eq!(new_region.min, IVec2::new(32, 32));
            assert_eq!(map.origin(), IVec2::new(32, 32));
        }

        #[test]
        fn data_persists_after_origin_shift() {
            let mut map = create_map();
            let entity = Entity::from_bits(42);

            let _ = map.insert(IVec2::new(40, 40), entity);

            map.shift_origin(IVec2::new(32, 32));

            assert_eq!(map.get_copied(IVec2::new(40, 40)), Ok(Some(entity)));
        }

        #[test]
        fn position_outside_after_shift() {
            let mut map = create_map();
            let entity = Entity::from_bits(42);

            let _ = map.insert(IVec2::new(5, 5), entity);

            map.shift_origin(IVec2::new(32, 32));

            assert!(!map.is_position_loaded(IVec2::new(5, 5)));
        }
    }

    mod spatial_queries {
        use super::*;

        #[test]
        fn within_radius_finds_entities() {
            let mut map = create_map();

            let e1 = Entity::from_bits(1);
            let e2 = Entity::from_bits(2);
            let e3 = Entity::from_bits(3);

            let _ = map.insert(IVec2::new(10, 10), e1);
            let _ = map.insert(IVec2::new(12, 10), e2);
            let _ = map.insert(IVec2::new(20, 20), e3);

            let center = IVec2::new(10, 10);
            assert_eq!(map.within_radius(center, 5.0).count(), 2);
        }
    }

    mod entry_api {
        use super::*;

        #[test]
        fn entry_insert_if_vacant_when_empty() {
            let mut map = create_map();
            let entity = Entity::from_bits(42);
            let pos = IVec2::new(5, 5);

            let mut entry = map.entry(pos).unwrap();
            let inserted = entry.insert_if_vacant(entity);

            assert!(inserted);
            assert_eq!(map.get_copied(pos), Ok(Some(entity)));
        }

        #[test]
        fn entry_insert_if_vacant_when_occupied() {
            let mut map = create_map();
            let e1 = Entity::from_bits(1);
            let e2 = Entity::from_bits(2);
            let pos = IVec2::new(5, 5);

            let _ = map.insert(pos, e1);

            let mut entry = map.entry(pos).unwrap();
            let inserted = entry.insert_if_vacant(e2);

            assert!(!inserted);
            assert_eq!(map.get_copied(pos), Ok(Some(e1)));
        }
    }
}
