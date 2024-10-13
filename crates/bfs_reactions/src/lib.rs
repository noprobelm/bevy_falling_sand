mod burning;

use bevy::prelude::*;
use bevy_turborand::{prelude::RngComponent, DelegatedRng};
use std::ops::RangeBounds;
use bfs_core::{Coordinates, Particle};

pub use burning::*;

pub struct FallingSandReactionsPlugin;

impl Plugin for FallingSandReactionsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ReactionRng>()
            .register_type::<Reacting>();
	app.add_plugins(BurningPlugin);
    }
}

/// RNG to use when dealing with any entity that needs random reaction behavior.
#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct ReactionRng(pub RngComponent);

impl ReactionRng {
    /// Shuffles a given slice.
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        self.0.shuffle(slice);
    }

    /// Returns a boolean value based on a rate. rate represents the chance to return a true value, with 0.0 being no
    /// chance and 1.0 will always return true.
    pub fn chance(&mut self, rate: f64) -> bool {
        self.0.chance(rate)
    }

    /// Samples a random item from a slice of values.
    pub fn sample<'a, T>(&mut self, list: &'a [T]) -> Option<&'a T> {
        self.0.sample(&list)
    }

    /// Returns a usize value for stable indexing across different word size platforms.
    pub fn index(&mut self, bound: impl RangeBounds<usize>) -> usize {
        self.0.index(bound)
    }
}

/// Component for particles that are creating new particles as part of a reaction.
#[derive(Clone, PartialEq, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Reacting {
    /// What the reaction will produce.
    pub produces: Particle,
    /// The chance that the particle will produce something (0.0 is lowest chance, 1.0 is highest).
    pub chance_to_produce: f64,
}

impl Reacting {
    /// Creates a new Reacting.
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
