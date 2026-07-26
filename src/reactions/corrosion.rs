use std::time::Duration;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use bevy_rand::prelude::{GlobalRng, WyRand};

use crate::{
    GridPosition, ParticleSyncExt, ParticleSystems,
    core::{ParticleChunksMut, ParticleRngExt},
};

pub(super) struct CorrosionPlugin;

impl Plugin for CorrosionPlugin {
    fn build(&self, app: &mut App) {
        app.register_particle_sync_component::<Corrosive>()
            .register_particle_sync_component::<Corrodible>()
            .add_systems(
                PostUpdate,
                handle_corrosion.in_set(ParticleSystems::Simulation),
            );
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
    /// Create a corrosive component with the given probability.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_falling_sand::reactions::Corrosive;
    ///
    /// let corrosive = Corrosive::new(0.05);
    /// assert_eq!(corrosive.chance, 0.05);
    /// ```
    #[must_use]
    pub fn new(chance: f64) -> Self {
        Self {
            chance,
            tick_timer: Timer::new(Duration::ZERO, TimerMode::Repeating),
        }
    }

    /// Set the interval between corrosion attempts.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use bevy_falling_sand::reactions::Corrosive;
    ///
    /// let corrosive =
    ///     Corrosive::new(0.05).with_tick_rate(Duration::from_millis(100));
    /// assert_eq!(corrosive.tick_timer.duration(), Duration::from_millis(100));
    /// ```
    #[must_use]
    pub fn with_tick_rate(mut self, tick_rate: Duration) -> Self {
        self.tick_timer.set_duration(tick_rate);
        self
    }
}

/// Marker component for particles subject to corrosive materials
#[derive(Component, Copy, Clone, Eq, PartialEq, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Corrodible;

#[allow(clippy::needless_pass_by_value)]
fn handle_corrosion(
    mut commands: Commands,
    mut particle_chunks: ParticleChunksMut,
    time: Res<Time>,
    mut corrosive: Query<(&mut Corrosive, &GridPosition)>,
    corrodible: Query<&Corrodible>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    particle_chunks.for_each_dirty_particle(|map, dirty_state, pos, entity| {
        let Ok((corrosive, _)) = corrosive.get_mut(entity) else {
            return;
        };

        for (neighbor_pos, neighbor_entity) in map.within_radius(pos, 1.0) {
            if neighbor_pos == pos {
                continue;
            }

            if corrodible.get(neighbor_entity).is_err() {
                continue;
            }

            if !rng.chance(corrosive.chance) {
                dirty_state.mark_dirty(pos);
                continue;
            }

            commands.entity(neighbor_entity).despawn();
        }
    });

    let map = particle_chunks.map();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrosive_constructor_uses_zero_tick_rate() {
        let corrosive = Corrosive::new(0.25);

        assert_eq!(corrosive.chance, 0.25);
        assert_eq!(corrosive.tick_timer.duration(), Duration::ZERO);
    }

    #[test]
    fn corrosive_tick_rate_builder_sets_duration() {
        let tick_rate = Duration::from_millis(100);
        let corrosive = Corrosive::new(0.25).with_tick_rate(tick_rate);

        assert_eq!(corrosive.tick_timer.duration(), tick_rate);
    }
}
