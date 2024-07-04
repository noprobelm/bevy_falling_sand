use std::time::Duration;
use bevy::time::{Stopwatch, Timer};
use bevy::prelude::*;

/// This component controls whether a particle should be evaluated for movement in a given frame.
/// By using hibernation logic, we can significantly improve the performance of our simulation by
/// putting particles to "sleep" for a certain period of time, only to be woken up for periodic
/// checks.
///
/// Because particles might have this component frequently added/removed, we use "SparseSet" storage
/// for its data.
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct Hibernating(pub Timer);

impl Default for Hibernating {
    /// Implement a hibernation timer with 300 milliseconds. That is, a particle will perform a self
    /// check for movement every 300 ms.
    fn default() -> Self {
        Hibernating(Timer::new(Duration::from_millis(300), TimerMode::Repeating))
    }
}

/// This component keeps track of when a particle last moved. This is used primarily to influence
/// whether a particle will enter a hibernating state
#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct LastMoved(pub Stopwatch);
