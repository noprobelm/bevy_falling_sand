use bevy::prelude::*;
use bevy::utils::Duration;

use crate::{ParticleColor, Reacting};

/// Marker for particle types that can inflict a burning status.
#[derive(Clone, Debug, Component)]
pub struct Fire {
    /// The radius of the fire effect.
    pub burn_radius: f32,
    /// The chance that the particle will attempt to burn particles within its radius (0.0 is the lowest chance, 1.0
    /// is the highest).
    pub chance_to_spread: f64,
    /// The particle will destroy after spreading.
    pub destroys_on_spread: bool,
}

#[derive(Clone, Debug)]
/// Behavior for particles that have a chance to be destroyed while in a burning state.
pub struct BurnDestruction {
    /// Chance the particle will be destroyed per tick.
    pub chance_destroy_per_tick: f64,
}

impl BurnDestruction {
    /// Creates a new BurnDestruction
    pub fn new(chance_destroy_per_tick: f64) -> BurnDestruction {
        BurnDestruction {
            chance_destroy_per_tick,
        }
    }
}

/// Stores information about a particle that can burn.
#[derive(Clone, Component, Debug)]
pub struct Burns {
    /// Total duration for the burn effect.
    pub duration: Duration,
    /// Tick rate for which a reaction can occur.
    pub tick_rate: Duration,
    /// Chance the particle with destroy per tick. If this is not none, the particle will be destroyed upon completion
    /// of burning. Setting this value makes groups of burning particles look more natural as they disappear.
    pub chance_destroy_per_tick: Option<f64>,
    /// The ParticleReaction data.
    pub reaction: Option<Reacting>,
    /// The colors the particle should burn as.
    pub color: Option<ParticleColor>,
    /// Whether this particle will spread fire.
    pub spreads: Option<Fire>,
}

impl Burns {
    /// Creates a new Burns.
    pub fn new(
        duration: Duration,
        tick_rate: Duration,
        chance_destroy_per_tick: Option<f64>,
        reaction: Option<Reacting>,
        color: Option<ParticleColor>,
        spreads: Option<Fire>,
    ) -> Burns {
        Burns {
            duration,
            tick_rate,
            chance_destroy_per_tick,
            reaction,
            color,
            spreads,
        }
    }

    /// Provides a new Burning
    pub fn to_burning(&self) -> Burning {
        Burning::new(self.duration, self.tick_rate)
    }
}

/// Component for particles that have the capacity to burn.
#[derive(Clone, Component, Debug)]
pub struct Burning {
    /// The Burning timer.
    pub timer: Timer,
    /// The tick rate timer.
    pub tick_timer: Timer,
}

impl Burning {
    /// Creates a new Burning.
    pub fn new(duration: Duration, tick_rate: Duration) -> Burning {
        Burning {
            timer: Timer::new(duration, TimerMode::Repeating),
            tick_timer: Timer::new(tick_rate, TimerMode::Repeating),
        }
    }

    /// Ticks the burn timer.
    pub fn tick(&mut self, duration: Duration) {
        self.timer.tick(duration);
        self.tick_timer.tick(duration);
    }

    /// Resets the Burning status
    pub fn reset(&mut self) {
        self.timer.reset();
        self.tick_timer.reset();
    }
}
