//! Provides reaction functionality for particles
//!
//! There are currently two types of particle reactions
//! - [Contact reactions](`ContactReaction`): Rulesets for inter-particle reactions. Particle A
//!   interacts with Particle B to produce Particle C.
//! - [Fire] and [Flammability](`Flammable`): Contacts within a certain radius of `Fire` will
//!   ignite, adding "burning" behavior.

mod contact;
mod fire;

use bevy::prelude::*;
use bevy_turborand::RngComponent;
use serde::{Deserialize, Serialize};

use crate::impl_particle_rng;

pub use contact::{Consumes, ContactReaction, ContactRule};
pub use fire::{BurnProduct, Burning, Fire, Flammable};

use contact::ContactPlugin;
use fire::FirePlugin;

impl_particle_rng!(ReactionRng, RngComponent);

/// Provides RNG for particle reaction systems.
///
/// Automatically inserted on entities that receive a [`Flammable`] component.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::reactions::ReactionRng;
///
/// fn check_rng(query: Query<&ReactionRng>) {
///     println!("Entities with reaction RNG: {}", query.iter().len());
/// }
/// ```
#[derive(Component, Clone, Default, PartialEq, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ReactionRng(pub RngComponent);

/// Plugin providing particle reaction systems (fire, contact reactions).
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::reactions::FallingSandReactionsPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(FallingSandReactionsPlugin)
///         .run();
/// }
/// ```
pub struct FallingSandReactionsPlugin;

impl Plugin for FallingSandReactionsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ReactionRng>()
            .add_plugins((FirePlugin, ContactPlugin));
    }
}
