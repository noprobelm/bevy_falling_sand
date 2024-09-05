use crate::*;

/// Provides particle reaction behavior.
#[derive(Clone, Debug)]
pub struct Reacting {
    /// What the reaction will produce.
    pub produces: Particle,
    /// The chance that the particle will produce something (0.0 is lowest chance, 1.0 is highest).
    pub chance_to_produce: f64,
}

impl Reacting {
    /// Creates a new ParticleReaction.
    pub fn new(produces: Particle, chance_to_produce: f64) -> Reacting {
        Reacting {
            produces,
            chance_to_produce,
        }
    }

    /// Produces a new particle if the rng determines so.
    pub fn produce(
        &self,
        commands: &mut Commands,
        rng: &mut ReactionRng,
        coordinates: &Coordinates,
    ) {
        if self.chance(rng) {
            commands.spawn((
                self.produces.clone(),
                SpatialBundle::from_transform(Transform::from_xyz(
                    coordinates.0.x as f32,
                    coordinates.0.y as f32 + 1.,
                    0.,
                )),
            ));
        }
    }

    /// Returns a boolean value based on a rate. rate represents the chance to return a true value, with 0.0 being no
    /// chance and 1.0 will always return true.
    pub fn chance(&self, rng: &mut ReactionRng) -> bool {
        rng.chance(self.chance_to_produce)
    }
}
