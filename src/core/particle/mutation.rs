//! Components for changing a particle's type.
//!
//! Mutation changes [`AttachedToParticleType`] to the [`ParticleType`](crate::core::ParticleType)
//! registered for a target [`ParticleTypeId`], then synchronizes the particle with the new type.

use std::time::Duration;

use bevy::prelude::*;
use bevy_rand::prelude::{GlobalRng, WyRand};

use crate::core::{
    AttachedToParticleType, Particle, ParticleRngExt, ParticleSyncExt, ParticleSystems,
    ParticleTypeId, ParticleTypeRegistry,
};

pub(super) struct MutationPlugin;

impl Plugin for MutationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ChanceMutation>()
            .register_type::<TimedMutation>()
            .register_particle_sync_component::<ChanceMutation>()
            .register_particle_sync_component::<TimedMutation>()
            .add_systems(
                PostUpdate,
                (handle_timed_mutations, handle_chance_mutations)
                    .in_set(ParticleSystems::Simulation),
            );
    }
}

/// Mutates a particle into another particle type after a specified duration of particle
/// simulation.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use bevy_falling_sand::core::{ParticleTypeId, TimedMutation};
///
/// let water = ParticleTypeId::new();
/// let mutation = TimedMutation::new(water, Duration::from_secs(2));
/// assert_eq!(mutation.target, water);
/// assert_eq!(mutation.duration(), Duration::from_secs(2));
/// ```
#[derive(Component, Clone, Default, Eq, PartialEq, Debug, Reflect)]
#[reflect(Component)]
#[type_path = "bfs_core::particle"]
pub struct TimedMutation {
    /// The [`ParticleTypeId`] this particle should mutate into.
    pub target: ParticleTypeId,
    /// Timer that controls when the particle mutates.
    pub timer: Timer,
}

impl TimedMutation {
    /// Create a timed mutation targeting `target` after `duration`.
    #[must_use]
    pub fn new(target: impl Into<ParticleTypeId>, duration: Duration) -> Self {
        Self {
            target: target.into(),
            timer: Timer::new(duration, TimerMode::Once),
        }
    }

    /// Returns the delay before mutation.
    #[must_use]
    pub fn duration(&self) -> Duration {
        self.timer.duration()
    }
}

/// Gives a particle a chance to mutate at a configured interval of particle simulation.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use bevy_falling_sand::core::{ChanceMutation, ParticleTypeId};
///
/// let water = ParticleTypeId::new();
/// let mutation = ChanceMutation::new(water, 0.05, Duration::from_millis(100));
/// assert_eq!(mutation.target, water);
/// assert_eq!(mutation.chance, 0.05);
/// assert_eq!(mutation.tick_timer.duration(), Duration::from_millis(100));
/// ```
#[derive(Component, Clone, PartialEq, Debug, Reflect)]
#[reflect(Component)]
#[type_path = "bfs_core::particle"]
pub struct ChanceMutation {
    /// The [`ParticleTypeId`] this particle should mutate into.
    pub target: ParticleTypeId,
    /// The probability (0.0 to 1.0) that the particle will mutate each tick.
    pub chance: f64,
    /// Timer that controls how often the chance is evaluated.
    pub tick_timer: Timer,
}

impl Default for ChanceMutation {
    fn default() -> Self {
        Self {
            target: ParticleTypeId::default(),
            chance: 0.0,
            tick_timer: Timer::new(Duration::ZERO, TimerMode::Repeating),
        }
    }
}

impl ChanceMutation {
    /// Create a chance-based mutation targeting `target`.
    #[must_use]
    pub fn new(target: impl Into<ParticleTypeId>, chance: f64, tick_rate: Duration) -> Self {
        Self {
            target: target.into(),
            chance,
            tick_timer: Timer::new(tick_rate, TimerMode::Repeating),
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn handle_chance_mutations(
    mut query: Query<(&mut AttachedToParticleType, &mut ChanceMutation), With<Particle>>,
    registry: Res<ParticleTypeRegistry>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    time: Res<Time>,
) {
    for (mut attached, mut mutation) in &mut query {
        if mutation.tick_timer.tick(time.delta()).just_finished()
            && rng.chance(mutation.chance)
            && let Some(&new_parent) = registry.get(mutation.target)
            && attached.0 != new_parent
        {
            attached.0 = new_parent;
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn handle_timed_mutations(
    mut query: Query<(&mut AttachedToParticleType, &mut TimedMutation), With<Particle>>,
    registry: Res<ParticleTypeRegistry>,
    time: Res<Time>,
) {
    for (mut attached, mut mutation) in &mut query {
        if mutation.timer.tick(time.delta()).is_finished()
            && let Some(&new_parent) = registry.get(mutation.target)
            && attached.0 != new_parent
        {
            attached.0 = new_parent;
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::time::TimeUpdateStrategy;

    use super::*;
    use crate::FallingSandMinimalPlugin;
    use crate::core::{ParticleMap, ParticleSimulationRun, ParticleType, SpawnParticleSignal};

    fn create_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(FallingSandMinimalPlugin::default())
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                50,
            )));
        app
    }

    fn spawn_particle(app: &mut App, particle_type: ParticleTypeId) -> Entity {
        app.world_mut()
            .write_message(SpawnParticleSignal::new(particle_type, IVec2::ZERO));
        app.update();
        app.world()
            .resource::<ParticleMap>()
            .get_copied(IVec2::ZERO)
            .unwrap()
            .unwrap()
    }

    fn sand() -> ParticleTypeId {
        ParticleTypeId::from_raw(0)
    }

    fn water() -> ParticleTypeId {
        ParticleTypeId::from_raw(1)
    }

    fn ghost() -> ParticleTypeId {
        ParticleTypeId::from_raw(999)
    }

    fn attached_to(app: &App, entity: Entity) -> Entity {
        app.world()
            .entity(entity)
            .get::<AttachedToParticleType>()
            .unwrap()
            .0
    }

    #[test]
    fn chance_mutation_default() {
        let mutation = ChanceMutation::default();
        assert_eq!(mutation.chance, 0.0);
        assert_eq!(mutation.tick_timer.duration(), Duration::ZERO);
    }

    #[test]
    fn chance_mutation_new_from_particle_type_id() {
        let mutation = ChanceMutation::new(water(), 0.5, Duration::from_millis(100));
        assert_eq!(mutation.target, water());
        assert_eq!(mutation.chance, 0.5);
        assert_eq!(mutation.tick_timer.duration(), Duration::from_millis(100));
    }

    #[test]
    fn chance_mutation_zero_never_mutates() {
        let mut app = create_test_app();
        let sand_parent = app.world_mut().spawn(ParticleType::from_id(sand())).id();
        app.world_mut().spawn(ParticleType::from_id(water()));
        app.update();

        let particle = spawn_particle(&mut app, sand());
        app.world_mut()
            .entity_mut(particle)
            .insert(ChanceMutation::new(water(), 0.0, Duration::ZERO));

        for _ in 0..100 {
            app.update();
        }

        assert_eq!(attached_to(&app, particle), sand_parent);
    }

    #[test]
    fn chance_mutation_one_always_mutates() {
        let mut app = create_test_app();
        let sand_parent = app.world_mut().spawn(ParticleType::from_id(sand())).id();
        let water_parent = app.world_mut().spawn(ParticleType::from_id(water())).id();
        app.update();

        let particle = spawn_particle(&mut app, sand());
        assert_eq!(attached_to(&app, particle), sand_parent);
        app.world_mut()
            .entity_mut(particle)
            .insert(ChanceMutation::new(water(), 1.0, Duration::ZERO));

        app.update();
        app.update();

        assert_eq!(attached_to(&app, particle), water_parent);
    }

    #[test]
    fn chance_mutation_respects_tick_rate() {
        let mut app = create_test_app();
        let sand_parent = app.world_mut().spawn(ParticleType::from_id(sand())).id();
        let water_parent = app.world_mut().spawn(ParticleType::from_id(water())).id();
        app.update();

        let particle = spawn_particle(&mut app, sand());
        app.world_mut()
            .entity_mut(particle)
            .insert(ChanceMutation::new(water(), 1.0, Duration::from_secs(999)));
        app.update();
        app.update();
        assert_eq!(attached_to(&app, particle), sand_parent);

        *app.world_mut()
            .entity_mut(particle)
            .get_mut::<ChanceMutation>()
            .unwrap() = ChanceMutation::new(water(), 1.0, Duration::ZERO);
        app.update();
        app.update();

        assert_eq!(attached_to(&app, particle), water_parent);
    }

    #[test]
    fn chance_mutation_unregistered_target_is_skipped() {
        let mut app = create_test_app();
        let sand_parent = app.world_mut().spawn(ParticleType::from_id(sand())).id();
        app.update();

        let particle = spawn_particle(&mut app, sand());
        app.world_mut()
            .entity_mut(particle)
            .insert(ChanceMutation::new(ghost(), 1.0, Duration::ZERO));

        app.update();
        app.update();

        assert_eq!(attached_to(&app, particle), sand_parent);
    }

    #[test]
    fn chance_mutation_propagates_from_particle_type() {
        let mut app = create_test_app();
        app.world_mut().spawn((
            ParticleType::from_id(sand()),
            ChanceMutation::new(water(), 1.0, Duration::ZERO),
        ));
        let water_parent = app.world_mut().spawn(ParticleType::from_id(water())).id();
        app.update();

        let particle = spawn_particle(&mut app, sand());
        assert!(
            app.world()
                .entity(particle)
                .get::<ChanceMutation>()
                .is_some()
        );

        app.update();
        app.update();

        assert_eq!(attached_to(&app, particle), water_parent);
    }

    #[test]
    fn timed_mutation_mutates_after_delay() {
        let mut app = create_test_app();
        let source = ParticleTypeId::from_raw(100);
        let target = ParticleTypeId::from_raw(101);
        let source_parent = app.world_mut().spawn(ParticleType::from_id(source)).id();
        let target_parent = app.world_mut().spawn(ParticleType::from_id(target)).id();
        app.update();

        let particle = spawn_particle(&mut app, source);
        assert_eq!(
            app.world()
                .get::<AttachedToParticleType>(particle)
                .unwrap()
                .0,
            source_parent
        );
        app.world_mut()
            .entity_mut(particle)
            .insert(TimedMutation::new(target, Duration::from_millis(100)));

        app.update();
        app.update();
        app.update();

        assert_eq!(
            app.world()
                .get::<AttachedToParticleType>(particle)
                .unwrap()
                .0,
            target_parent
        );
    }

    #[test]
    fn timed_mutation_skips_target_removed_before_execution() {
        let mut app = create_test_app();
        let source = ParticleTypeId::from_raw(110);
        let target = ParticleTypeId::from_raw(111);
        let source_parent = app.world_mut().spawn(ParticleType::from_id(source)).id();
        app.world_mut().spawn(ParticleType::from_id(target));
        app.update();

        let particle = spawn_particle(&mut app, source);
        app.world_mut()
            .entity_mut(particle)
            .insert(TimedMutation::new(target, Duration::from_millis(100)));
        app.update();
        app.world_mut()
            .resource_mut::<ParticleTypeRegistry>()
            .remove(target);
        app.update();
        app.update();

        assert_eq!(
            app.world()
                .get::<AttachedToParticleType>(particle)
                .unwrap()
                .0,
            source_parent
        );
    }

    #[test]
    fn stale_timed_mutation_command_is_ignored() {
        let mut app = create_test_app();
        let source = ParticleTypeId::from_raw(120);
        let old_target = ParticleTypeId::from_raw(121);
        let new_target = ParticleTypeId::from_raw(122);
        let source_parent = app.world_mut().spawn(ParticleType::from_id(source)).id();
        app.world_mut().spawn(ParticleType::from_id(old_target));
        app.world_mut().spawn(ParticleType::from_id(new_target));
        app.update();

        let particle = spawn_particle(&mut app, source);
        app.world_mut()
            .entity_mut(particle)
            .insert(TimedMutation::new(old_target, Duration::from_millis(100)));
        app.update();
        app.world_mut()
            .entity_mut(particle)
            .insert(TimedMutation::new(new_target, Duration::from_secs(10)));
        app.update();
        app.update();

        assert_eq!(
            app.world()
                .get::<AttachedToParticleType>(particle)
                .unwrap()
                .0,
            source_parent
        );
    }

    #[test]
    fn timed_mutation_pauses_with_simulation() {
        let mut app = create_test_app();
        let source = ParticleTypeId::from_raw(130);
        let target = ParticleTypeId::from_raw(131);
        let source_parent = app.world_mut().spawn(ParticleType::from_id(source)).id();
        app.world_mut().spawn(ParticleType::from_id(target));
        app.update();

        let particle = spawn_particle(&mut app, source);
        app.world_mut()
            .entity_mut(particle)
            .insert(TimedMutation::new(target, Duration::from_millis(100)));
        app.world_mut().remove_resource::<ParticleSimulationRun>();

        app.update();
        app.update();
        app.update();

        assert_eq!(
            app.world()
                .get::<AttachedToParticleType>(particle)
                .unwrap()
                .0,
            source_parent
        );
        assert_eq!(
            app.world()
                .get::<TimedMutation>(particle)
                .unwrap()
                .timer
                .elapsed(),
            Duration::ZERO
        );
    }
}
