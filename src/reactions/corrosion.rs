use std::time::Duration;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    ChunkDirtyState, ChunkIndex, GridPosition, ParticleMap, ParticleSyncExt, ParticleSystems,
};

pub(super) struct CorrosionPlugin;

impl Plugin for CorrosionPlugin {
    fn build(&self, app: &mut App) {
        app.register_particle_sync_component::<Corrosive>()
            .register_particle_sync_component::<Corrodible>()
            .add_systems(Update, handle_corrosion.in_set(ParticleSystems::Simulation));
    }
}

/// Marker component for corrosive materials.
#[derive(Component, Clone, PartialEq, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Corrosive {
    /// The probability (0.0 to 1.0) that the particle will consume an adjacent corrodible particle.
    pub chance: f64,
    /// Timer that controls how often the chance is evaluated.
    pub tick_timer: Timer,
}

impl Default for Corrosive {
    fn default() -> Self {
        Self {
            chance: 0.0,
            tick_timer: Timer::new(Duration::ZERO, TimerMode::Repeating),
        }
    }
}

impl Corrosive {
    /// Create a new corrosive marker with the given probability and tick rate.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use bevy_falling_sand::reactions::Corrosive;
    ///
    /// let corrosive = Corrosive::new(0.05, Duration::from_millis(100));
    /// assert_eq!(corrosive.chance, 0.05);
    /// ```
    #[must_use]
    pub fn new(chance: f64, tick_rate: Duration) -> Self {
        Self {
            chance,
            tick_timer: Timer::new(tick_rate, TimerMode::Repeating),
        }
    }

    /// Create a new corrosive marker with the given probability and tick rate.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use bevy_falling_sand::reactions::Corrosive;
    ///
    /// let corrosive = Corrosive::with_tick_rate(0.05, Duration::from_millis(100));
    /// assert_eq!(corrosive.tick_timer.duration(), Duration::from_millis(100));
    /// ```
    #[must_use]
    pub fn with_tick_rate(chance: f64, tick_rate: Duration) -> Self {
        Self {
            chance,
            tick_timer: Timer::new(tick_rate, TimerMode::Repeating),
        }
    }
}

/// Marker component for particles subject to corrosive materials
#[derive(Component, Copy, Clone, Eq, PartialEq, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Corrodible;

#[allow(clippy::needless_pass_by_value)]
fn handle_corrosion(
    mut commands: Commands,
    map: Res<ParticleMap>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<&mut ChunkDirtyState>,
    time: Res<Time>,
    mut corrosive: Query<(&mut Corrosive, &GridPosition)>,
    corrodible: Query<&Corrodible>,
    mut rng: ResMut<bevy_turborand::prelude::GlobalRng>,
) {
    use bevy_turborand::DelegatedRng;

    for (_coord, chunk_entity) in chunk_index.iter() {
        let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity) else {
            continue;
        };

        let Some(dirty_rect) = dirty_state.current else {
            continue;
        };

        for y in dirty_rect.min.y..=dirty_rect.max.y {
            for x in dirty_rect.min.x..=dirty_rect.max.x {
                let pos = IVec2::new(x, y);

                let Ok(Some(entity)) = map.get_copied(pos) else {
                    continue;
                };

                let Ok((corrosive, _)) = corrosive.get_mut(entity) else {
                    continue;
                };

                for (neighbor_pos, neighbor_entity) in map.within_radius(pos, 1.0) {
                    if neighbor_pos == pos {
                        continue;
                    }

                    let Ok(_) = corrodible.get(neighbor_entity) else {
                        continue;
                    };

                    if !rng.chance(corrosive.chance) {
                        dirty_state.mark_dirty(pos);
                        continue;
                    }

                    if corrodible.get(neighbor_entity).is_ok() {
                        commands.entity(neighbor_entity).despawn();
                    }
                }
            }
        }
    }

    corrosive.iter_mut().for_each(|(mut corrosive, pos)| {
        if let Ok(e) = map.get_copied(pos.0)
            && e.is_some()
        {
            corrosive.tick_timer.tick(time.delta());
            if corrosive.tick_timer.is_finished() && rng.chance(corrosive.chance) {
                for (neighbor_pos, neighbor_entity) in map.within_radius(pos.0, 1.0) {
                    if neighbor_pos == pos.0 {
                        continue;
                    }

                    if corrodible.get(neighbor_entity).is_ok() {
                        commands.entity(neighbor_entity).despawn();
                    }
                }
            }
        }
    });
}
