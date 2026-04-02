//! Provides movement components, systems, and resources for simulating particle movement
//!
//! # Particle Movement Components
//!
//! The following table provides a brief overview of movement components, their behavior, and
//! whether they are required for a particle to be evaluated for movement.
//!
//! | Particle Component | Description                                                              | Required |
//! | ------------------ | ------------------------------------------------------------------------ | -------- |
//! | [`Movement`]       | Movement pattern as an ordered list of neighbor groups.                  | ✅       |
//! | [`Density`]        | Density of a particle, used for displacement comparisons.                | ✅       |
//! | [`Speed`]          | Controls how many positions a particle can move per frame.               | ✅       |
//! | [`AirResistance`]  | Per-tier air resistance values parallel to movement neighbor groups.     | ✅       |
//! | [`ParticleResistor`]       | How much this particle resists being displaced.                          | ❌       |
//! | [`Momentum`]       | Directional hint that biases movement toward the last direction.         | ❌       |
//!
//! Adding the `Movement` component to a [`ParticleType`](crate::ParticleType) entity
//! automatically adds the minimum required components necessary for movement evaluation, but each
//! component should be manually configured to your liking in order to create sensible movement
//! behaviors and interactions with other particles.
//!
//! For example, a particle with sand-like behavior might look like this
//!
//!```
//! // Spawn a simple particle type with colors and movement behavior resembling sand.
//! use bevy::prelude::*;
//! use bevy_falling_sand::prelude::*;
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn((
//!         ParticleType::new("Sand"),
//!         ColorProfile::palette(vec![
//!             Color::Srgba(Srgba::hex("#FFEB8A").unwrap()),
//!             Color::Srgba(Srgba::hex("#F2E06B").unwrap()),
//!         ]),
//!         // First tier: look directly below. Second tier: look diagonally down.
//!         Movement::from(vec![
//!             vec![IVec2::NEG_Y],
//!             vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
//!         ]),
//!         // If we wanted `Sand` to pass through `Water`, ensure water's density is less than
//!         // `1250`.
//!         Density(1250),
//!         // Max speed of 10, stepping up speed every 5 moves that execute unobstructed
//!         Speed::new(5, 10),
//!     ));
//! }
//!```
//!
//! # Movement Processing Modes
//!
//! The [`MovementSystemState`] and [`ChunkIterationState`] states provide control over how
//! movement simulation is executed
//!
//! - [`MovementSystemState::Chunks`] iterates through particles by chunk
//!   - [`ChunkIterationState::Parallel`] (default) iterates through chunks in a parallel,
//!     checkerboard pattern. Provides significant speedups for particle movement.
//!   - [`ChunkIterationState::Serial`] iterates through chunks serially. Less efficient than
//!     parallel chunk iteration
//! - [`MovementSystemState::Particles`] simply iterates through a
//!   [`Particle`](crate::prelude::Particle) query. Slowest method of movement simulation, but has a
//!   smoother look than chunk iteration.
//!
/// Particle movement components and despawn signals.
pub mod particle;
/// Movement processing modes, states, and systems.
pub mod processing;
/// System set definitions for particle movement.
pub mod schedule;

use bevy::prelude::*;

pub use particle::*;
pub use processing::*;
pub use schedule::*;

use particle::ParticlePlugin;
use processing::ProcessingPlugin;
use schedule::SchedulePlugin;

/// Plugin providing particle movement simulation.
///
/// Registers all movement components, states, and systems.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::movement::FallingSandMovementPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(FallingSandMovementPlugin)
///         .run();
/// }
/// ```
pub struct FallingSandMovementPlugin;

impl Plugin for FallingSandMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SchedulePlugin, ParticlePlugin, ProcessingPlugin));
    }
}
