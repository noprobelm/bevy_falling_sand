use bevy::prelude::*;
use bevy::utils::Duration;

use crate::{Particle, PhysicsRng};

/// Marker for particle types that can inflict burning.
#[derive(Component)]
pub struct Fire {
    /// The burn radius to use for the particle tree spatial query.
    pub burn_radius: Vec2
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
    pub reaction: Option<ParticleReaction>,
}

impl Burns {
    /// Creates a new Burns.
    pub fn new(
        duration: Duration,
        tick_rate: Duration,
        reaction: Option<ParticleReaction>,
    ) -> Burns {
        Burns {
            timer: Timer::new(duration, TimerMode::Repeating),
            tick_timer: Timer::new(tick_rate, TimerMode::Repeating),
            reaction,
        }
    }

    /// Ticks the burn timer.
    pub fn tick(&mut self, time: Res<Time>) {
	self.tick_timer.tick(time.elapsed());
	self.timer.tick(time.elapsed());
    }
}

/// Marker Component for particles that are currently burning
#[derive(Clone, Component)]
pub struct Burning;
