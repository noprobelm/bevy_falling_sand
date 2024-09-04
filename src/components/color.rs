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
    /// The index to reference when changing through colors sequentially.
    color_index: usize,
    /// The current color.
    pub selected: Color,
    /// The possible range of colors.
    pub palette: Vec<Color>,
}

impl ParticleColor {
    /// Creates a new ParticleColors component with the specified colors.
    pub fn new(selected: Color, palette: Vec<Color>) -> ParticleColor {
        ParticleColor { color_index: 0, selected, palette }
    }

    /// Select a random color from the colors sequence.
    pub fn new_with_random<R: TurboRand>(&self, rng: &mut R) -> ParticleColor {
	let color_index = rng.index(0..self.palette.len());
	ParticleColor { color_index, selected: *self.palette.get(color_index).unwrap(), palette: self.palette.clone() }
    }

    /// Randomize the current color.
    pub fn randomize(&mut self, rng: &mut PhysicsRng) {
	self.color_index = rng.index(0..self.palette.len());
	self.selected = *self.palette.get(self.color_index).unwrap();
    }

    /// Change to the next color in the palette
    pub fn set_next(&mut self) {
	if self.palette.len() - 1 == self.color_index {
	    self.color_index = 0;
	} else {
	    self.color_index += 1;
	}
	self.selected = *self.palette.get(self.color_index).unwrap();
    }
}

/// Indicates the logic for a particle that changes colors.
#[derive(Clone, PartialEq, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct RandomizesColor {
    /// The chance a particle's color will change.
    pub rate: f64
}

impl RandomizesColor {
    /// Creates a new RandomizesColors
    pub fn new(chance: f64) -> RandomizesColor {
        RandomizesColor{rate: chance}
    }
}

/// Indicates the logic for a particle that changes colors.
#[derive(Clone, PartialEq, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct FlowsColor {
    /// The chance a particle's color will change.
    pub rate: f64
}

impl FlowsColor {
    /// Creates a new RandomizesColors
    pub fn new(chance: f64) -> FlowsColor {
        FlowsColor{rate: chance}
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
