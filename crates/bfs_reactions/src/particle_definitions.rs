use bevy::prelude::*;
use bevy_turborand::RngComponent;
use bfs_color::ColorProfile;
use bfs_core::{
    impl_particle_rng, Particle, ParticlePosition, ParticleRegistrationEvent, ParticleRng,
    ParticleSimulationSet, ParticleTypeId,
};
use std::time::Duration;

pub(super) struct ParticleDefinitionsPlugin;

impl Plugin for ParticleDefinitionsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ReactionRng>()
            .register_type::<Fire>()
            .register_type::<Burning>()
            .register_type::<Burns>()
            .add_systems(
                Update,
                handle_particle_registration.before(ParticleSimulationSet),
            );
    }
}

impl_particle_rng!(ReactionRng, RngComponent);

/// Provides rng for particle reaction systems.
#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct ReactionRng(pub RngComponent);

/// Component which indicates an entity is emitting fire.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Fire {
    /// The radius of the fire, which determines hwo far it can spread.
    pub burn_radius: f32,
    /// The chance for the fire to spread to adjacent particles (per tick)
    pub chance_to_spread: f64,
    /// The entity will be destroyed upon spreading to another particle.
    pub destroys_on_spread: bool,
}

/// Component which indicates an entity has the capacity to burn.
#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Burns {
    /// The duration the entity will burn for.
    pub duration: Duration,
    /// The tick rate for the burn effect.
    pub tick_rate: Duration,
    /// The chance the entity will be destroyed per tick.
    pub chance_destroy_per_tick: Option<f64>,
    /// Indicates the burn effect might produce a new particle type, such as ash or smoke.
    pub reaction: Option<Reacting>,
    /// Indicates the burn effect should have its own [`ColorProfile`]
    pub color: Option<ColorProfile>,
    /// Indicates whether the burning entity can spread fire to adjacent entities.
    pub spreads: Option<Fire>,
    /// Indicates whether the burning entity should ignite upon being spawned.
    pub ignites_on_spawn: bool,
}

impl Burns {
    /// Initialize a new `Burns` with a specific duration, tick rate, and optional parameters.
    #[must_use]
    pub const fn new(
        duration: Duration,
        tick_rate: Duration,
        chance_destroy_per_tick: Option<f64>,
        reaction: Option<Reacting>,
        color: Option<ColorProfile>,
        spreads: Option<Fire>,
        ignites_on_spawn: bool,
    ) -> Self {
        Self {
            duration,
            tick_rate,
            chance_destroy_per_tick,
            reaction,
            color,
            spreads,
            ignites_on_spawn,
        }
    }

    /// Initialize a new [`Burning`] from [`Burns`] data.
    #[must_use]
    pub fn to_burning(&self) -> Burning {
        Burning::new(self.duration, self.tick_rate)
    }
}

/// Component which indicates an entity is actively burning.
#[derive(Clone, Eq, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Burning {
    /// The duration the entity will burn for.
    pub timer: Timer,
    /// The tick rate for the burn effect.
    pub tick_timer: Timer,
}

#[allow(dead_code)]
impl Burning {
    /// Initialize a new `Burning` with a specific duration and tick rate.
    ///
    /// # Panics
    ///
    /// Panics if `duration` is not greater than `tick_rate`.
    #[must_use]
    pub fn new(duration: Duration, tick_rate: Duration) -> Self {
        assert!(
            duration >= tick_rate,
            "Burning duration must be greater than tick rate"
        );
        Self {
            timer: Timer::new(duration, TimerMode::Repeating),
            tick_timer: Timer::new(tick_rate, TimerMode::Repeating),
        }
    }

    /// Tick the burning and tick timer by a specified duration.
    pub fn tick(&mut self, duration: Duration) {
        self.timer.tick(duration);
        self.tick_timer.tick(duration);
    }

    /// Reset the burning and tick timers to zero.
    pub fn reset(&mut self) {
        self.timer.reset();
        self.tick_timer.reset();
    }
}

/// Component indicating a particle entity is undergoing a reaction.
#[derive(Clone, PartialEq, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Reacting {
    /// What the particle will produce when the reaction occurs.
    pub produces: Particle,
    /// The chance the reaction will occur per frame.
    pub chance_to_produce: f64,
}

impl Reacting {
    /// Initialize a new `Reacting` with a specific particle and chance to produce it.
    #[must_use]
    pub const fn new(produces: Particle, chance_to_produce: f64) -> Self {
        Self {
            produces,
            chance_to_produce,
        }
    }

    /// Produce a new particle above the current position of the particle.
    pub fn produce(
        &self,
        commands: &mut Commands,
        rng: &mut ReactionRng,
        position: &ParticlePosition,
    ) {
        if self.chance(rng) {
            commands.spawn((
                self.produces.clone(),
                Transform::from_xyz(position.0.x as f32, position.0.y as f32 + 1., 0.),
            ));
        }
    }

    fn chance(&self, rng: &mut ReactionRng) -> bool {
        rng.chance(self.chance_to_produce)
    }
}

type ReactionsQuery<'a> = (Option<&'a Fire>, Option<&'a Burns>, Option<&'a Burning>);

fn handle_particle_components(
    commands: &mut Commands,
    parent_query: &Query<ReactionsQuery, With<ParticleTypeId>>,
    particle_query: &Query<&ChildOf, With<Particle>>,
    entities: &[Entity],
) {
    for entity in entities {
        if let Ok(child_of) = particle_query.get(*entity) {
            if let Ok((fire, burns, burning)) = parent_query.get(child_of.parent()) {
                commands.entity(*entity).insert(ReactionRng::default());
                if let Some(fire) = fire {
                    commands.entity(*entity).insert(*fire);
                } else {
                    commands.entity(*entity).remove::<Fire>();
                }
                if let Some(burning) = burning {
                    commands.entity(*entity).insert(burning.clone());
                } else {
                    commands.entity(*entity).remove::<Burning>();
                }
                if let Some(burns) = burns {
                    commands.entity(*entity).insert(burns.clone());
                    if burns.ignites_on_spawn {
                        commands.entity(*entity).insert(burns.to_burning());
                        if let Some(fire) = burns.spreads {
                            commands.entity(*entity).insert(fire);
                        }
                    }
                } else {
                    commands.entity(*entity).remove::<Burns>();
                }
            }
        }
    }
}

fn handle_particle_registration(
    mut commands: Commands,
    parent_query: Query<ReactionsQuery, With<ParticleTypeId>>,
    particle_query: Query<&ChildOf, With<Particle>>,
    mut ev_particle_registered: EventReader<ParticleRegistrationEvent>,
) {
    ev_particle_registered.read().for_each(|ev| {
        handle_particle_components(&mut commands, &parent_query, &particle_query, &ev.entities);
    });
}
