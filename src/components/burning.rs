use bevy::prelude::*;
use bevy::utils::Duration;

use crate::{Coordinates, Particle, PhysicsRng};

/// Marker for particle types that can inflict burning.
#[derive(Clone, Debug, Component)]
pub struct Fire {
    /// The burn radius to use for the particle tree spatial query.
    pub burn_radius: f32,
}

/// Stores information for a particle type's reaction behavior.
#[derive(Clone)]
pub struct ParticleReaction {
    /// What the reaction will produce.
    pub produces: Particle,
    /// The chance that the particle will produce something (0.0 is highest chance, 1.0 is lowest).
    pub chance_to_produce: f64,
}

impl ParticleReaction {
    /// Creates a new ParticleReaction.
    pub fn new(produces: Particle, chance_to_produce: f64) -> ParticleReaction {
        ParticleReaction {
            produces,
            chance_to_produce,
        }
    }

    /// Produces a new particle if the rng determines so.
    pub fn produce(&self, commands: &mut Commands, coordinates: &Coordinates) {
        commands.spawn((
            self.produces.clone(),
            SpatialBundle::from_transform(Transform::from_xyz(
                coordinates.0.x as f32,
                coordinates.0.y as f32 + 1.,
                0.,
            )),
        ));
    }

    /// Returns a boolean value based on a rate. rate represents the chance to return a true value, with 0.0 being no
    /// chance and 1.0 will always return true.
    pub fn chance(&self, rng: &mut PhysicsRng) -> bool {
        rng.chance(self.chance_to_produce)
    }
}

/// Component for particles that have the capacity to burn.
#[derive(Clone, Component)]
pub struct Burns {
    /// Total duration for the burn effect.
    pub timer: Timer,
    /// Tick rate for which a reaction can occur.
    pub tick_timer: Timer,
    /// The ParticleReaction data.
    pub destroy: bool,
    /// Destroy the burning particle on burning completion.
    pub reaction: Option<ParticleReaction>
}

impl Burns {
    /// Creates a new Burns.
    pub fn new(
        duration: Duration,
        tick_rate: Duration,
	destroy: bool,
        reaction: Option<ParticleReaction>,
    ) -> Burns {
        Burns {
            timer: Timer::new(duration, TimerMode::Once),
            tick_timer: Timer::new(tick_rate, TimerMode::Repeating),
	    destroy,
            reaction,
        }
    }

    /// Ticks the burn timer.
    pub fn tick(&mut self, duration: Duration) {
        self.tick_timer.tick(duration);
        self.timer.tick(duration);
    }

    /// Resets the Burns status
    pub fn reset(&mut self) {
        self.timer.reset();
        self.tick_timer.reset();
    }
}

/// Marker Component for particles that are currently burning
#[derive(Clone, Component)]
pub struct Burning;
