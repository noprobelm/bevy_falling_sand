use bevy::prelude::*;
use bevy_turborand::prelude::*;

/// Provides a range of possible colors for a particle. Child particles will access
/// this component from their parent particle when spawning to select a color for themselves at
/// random.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ParticleColors {
    /// The possible range of colors.
    colors: Vec<Color>,
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
}

/// Provides an individual particle with a particular color, usually selected from
/// an existing entity with the ParticleColors component.
#[derive(Component)]
pub struct ParticleColor(pub Color);
