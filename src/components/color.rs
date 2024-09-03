//! Particle color components.
use bevy::prelude::*;
use bevy_turborand::prelude::*;

use crate::PhysicsRng;

/// Provides a range of possible colors for a particle. Child particles will access
/// this component from their parent particle when spawning to select a color for themselves at
/// random.
#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct ParticleColors {
    /// The possible range of colors.
    pub colors: Vec<Color>,
}

impl ParticleColors {
    /// Creates a new ParticleColors component with the specified colors.
    pub fn new(colors: Vec<Color>) -> Self {
        ParticleColors { colors }
    }

    /// Select a random color from the colors sequence.
    pub fn random<R: TurboRand>(&self, rng: &mut R) -> Color {
	rng.sample(&self.colors).unwrap().clone()
    }

    /// Docs
    pub fn random_with_physics_rng(&self, rng: &mut PhysicsRng) -> Color {
	rng.sample(&self.colors).unwrap().clone()
    }
}

/// Flag indicating a particle should change to a new color from its ParticleColors
#[derive(Component, Default, Reflect)]
pub struct RandomizeColors;

/// Provides a range of possible colors for a particle. Child particles will access
/// this component from their parent particle when spawning to select a color for themselves at
/// random.
#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct RandomColors {
    /// The possible range of colors.
    colors: Vec<Color>,
}

impl RandomColors {
    /// Creates a new RandomColors component with the specified colors.
    pub fn new(colors: Vec<Color>) -> Self {
        RandomColors { colors }
    }

    /// Select a random color from the colors sequence.
    pub fn random<R: TurboRand>(&self, rng: &mut R) -> Color {
	rng.sample(&self.colors).unwrap().clone()
    }

    /// Docs
    pub fn random_with_color_rng(&self, rng: &mut PhysicsRng) -> Color {
	rng.sample(&self.colors).unwrap().clone()
    }
}

/// Docs
#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
pub struct ParticleColor(pub Color, pub Vec<Color>);
