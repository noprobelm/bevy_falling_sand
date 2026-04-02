use bevy::platform::collections::HashSet;
use bevy::prelude::*;

/// Controls whether movement is processed chunk-by-chunk or particle-by-particle.
///
/// Use [`NextState<MovementSystemState>`] to transition between modes.
/// When switching to [`Particles`](MovementSystemState::Particles), the
/// [`ChunkIterationState`] sub-state becomes inactive.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::movement::MovementSystemState;
///
/// fn switch_to_particles(mut next: ResMut<NextState<MovementSystemState>>) {
///     next.set(MovementSystemState::Particles);
/// }
/// ```
#[derive(States, Copy, Clone, Reflect, Default, Debug, Eq, PartialEq, Hash)]
pub enum MovementSystemState {
    /// Process movement chunk-by-chunk. Enables [`ChunkIterationState`].
    #[default]
    Chunks,
    /// Process movement particle-by-particle.
    Particles,
}

/// Controls whether chunk-based movement iterates in parallel or serially.
///
/// This is a [`SubStates`] of [`MovementSystemState::Chunks`] — it is only
/// active when `MovementSystemState` is `Chunks`. When the parent state
/// switches to `Particles`, this sub-state becomes inactive automatically.
///
/// Use [`NextState<ChunkIterationState>`] to transition between modes.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::movement::ChunkIterationState;
///
/// fn switch_to_serial(mut next: ResMut<NextState<ChunkIterationState>>) {
///     next.set(ChunkIterationState::Serial);
/// }
/// ```
#[derive(SubStates, Copy, Clone, Reflect, Default, Debug, Eq, PartialEq, Hash)]
#[source(MovementSystemState = MovementSystemState::Chunks)]
pub enum ChunkIterationState {
    /// Parallel chunk processing using a checkerboard partitioning pattern.
    #[default]
    Parallel,
    /// Serial chunk processing, iterating one chunk at a time.
    Serial,
}

/// Internal resource tracking visited entities/positions during a movement frame.
#[derive(Resource, Default)]
pub struct MovementState {
    pub(crate) visited_entities: HashSet<Entity>,
    pub(crate) visited_positions: HashSet<IVec2>,
}
