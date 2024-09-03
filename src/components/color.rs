//! Particle color components.
use bevy::prelude::*;
use bevy_turborand::prelude::*;

use crate::PhysicsRng;

/// Provides a range of possible colors for a particle. Child particles will access
/// this component from their parent particle when spawning to select a color for themselves at
/// random.
#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct ParticleColor {
    /// The current color.
    pub selected: Color,
    /// The possible range of colors.
    pub palette: Vec<Color>,
}

impl ParticleColor {
    /// Creates a new ParticleColors component with the specified colors.
    pub fn new(selected: Color, palette: Vec<Color>) -> ParticleColor {
        ParticleColor { selected, palette }
    }

    /// Select a random color from the colors sequence.
    pub fn random<R: TurboRand>(&self, rng: &mut R) -> Color {
	rng.sample(&self.palette).unwrap().clone()
    }
}

/// Flag indicating a particle should change to a new color from its ParticleColors
#[derive(Component, Clone, Default, Reflect)]
pub struct RandomizeColors {
    /// The chance a particle's color will change.
    pub chance: f64
}

impl RandomizeColors {
    /// Creates a new RandomizesColors
    pub fn new(chance: f64) -> RandomizeColors {
        RandomizeColors{chance}
    }
}

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
