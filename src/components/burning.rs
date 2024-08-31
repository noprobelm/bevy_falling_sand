use bevy::prelude::*;
use bevy::utils::Duration;

use crate::{Coordinates, Particle, PhysicsRng, RandomColors, ParticleReaction};

/// Marker for particle types that can inflict burning.
#[derive(Clone, Debug, Component)]
pub struct Fire {
    /// The burn radius to use for the particle tree spatial query.
    pub burn_radius: f32,
    /// The chance that the particle will produce something (0.0 is lowest chance, 1.0 is highest).
    pub chance_to_spread: f64,
    /// Destroys after lighting something on fire.
    pub destroys_on_ignition: bool
}

/// Component for particles that have the capacity to burn.
#[derive(Clone, Component, Debug)]
pub struct Burns {
    /// Total duration for the burn effect.
    pub timer: Timer,
    /// Tick rate for which a reaction can occur.
    pub tick_timer: Timer,
    /// Destroy the burning particle on burning completion.
    pub destroy: bool,
    /// Chance the particle with destroy per tick.
    pub chance_destroy_per_tick: f64,
    /// The ParticleReaction data.
    pub reaction: Option<ParticleReaction>,
    /// What the particle should produce when it extinguishes.
    pub produces_on_completion: Option<Particle>,
    /// The colors to burn
    pub colors: Option<RandomColors>,
    /// Whether this particle will spread fire.
    pub spreads: Option<Fire>
}

impl Burns {
    /// Creates a new Burns.
    pub fn new(
        duration: Duration,
        tick_rate: Duration,
        destroy: bool,
	chance_destroy_per_tick: f64,
        reaction: Option<ParticleReaction>,
	produces_on_completion: Option<Particle>,
	colors: Option<RandomColors>,
	spreads: Option<Fire>
    ) -> Burns {
        Burns {
            timer: Timer::new(duration, TimerMode::Repeating),
            tick_timer: Timer::new(tick_rate, TimerMode::Repeating),
            destroy,
	    chance_destroy_per_tick,
            reaction,
	    produces_on_completion,
	    colors,
	    spreads
        }
    }

    /// Ticks the burn timer.
    pub fn tick(&mut self, duration: Duration) {
        self.timer.tick(duration);
        self.tick_timer.tick(duration);
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
