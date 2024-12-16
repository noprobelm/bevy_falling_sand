//! Defines additional components for particle types to be used as blueprint data when spawning or
//! resetting particles.
//!
//! This module is a standard template that can be followed when extending particle types. Its
//! structure is as follows:
//!   - Defines new components which will be associated with particle types as blueprint information
//!     for child particles.
//!   - Adds events for each new component which manage resetting information for child particles
//!   - Adds observers for each event to specify granular logic through which a particle should have
//!     its information reset. This usually involves referencing the parent `ParticleType`.
//!
//! When a particle should have its information reset (e.g., when spawning or resetting), we can
//! trigger the events defined in this module and communicate with higher level systems that
//! something needs to happen with a given particle.

use bevy::prelude::*;
use bevy::utils::Duration;
use bfs_color::*;
use bfs_core::{Coordinates, Particle, ParticleType};

use crate::ReactionRng;

pub struct ParticleDefinitionsPlugin;

impl Plugin for ParticleDefinitionsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Fire>()
            .register_type::<Burns>()
            .register_type::<Burning>()
            .register_type::<Reacting>();
        app.observe(on_reset_fire)
            .observe(on_reset_burns)
            .observe(on_reset_burning);
    }
}

/// Marker for particle types that can inflict a burning status.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Fire {
    /// The radius of the fire effect.
    pub burn_radius: f32,
    /// The chance that the particle will attempt to burn particles within its radius (0.0 is the lowest chance, 1.0
    /// is the highest).
    pub chance_to_spread: f64,
    /// The particle will destroy after spreading.
    pub destroys_on_spread: bool,
}

/// The Fire blueprint.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct FireBlueprint(pub Fire);

/// Stores information about a particle that can burn.
#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Burns {
    /// Total duration for the burn effect.
    pub duration: Duration,
    /// Tick rate for which a reaction can occur.
    pub tick_rate: Duration,
    /// Chance the particle with destroy per tick. If this is not none, the particle will be destroyed upon completion
    /// of burning. Setting this value makes groups of burning particles look more natural as they disappear.
    pub chance_destroy_per_tick: Option<f64>,
    /// The ParticleReaction data.
    pub reaction: Option<Reacting>,
    /// The colors the particle should burn as.
    pub color: Option<ParticleColor>,
    /// Whether this particle will spread fire.
    pub spreads: Option<Fire>,
}

impl Burns {
    #![allow(dead_code)]
    /// Creates a new Burns.
    pub fn new(
        duration: Duration,
        tick_rate: Duration,
        chance_destroy_per_tick: Option<f64>,
        reaction: Option<Reacting>,
        color: Option<ParticleColor>,
        spreads: Option<Fire>,
    ) -> Burns {
        Burns {
            duration,
            tick_rate,
            chance_destroy_per_tick,
            reaction,
            color,
            spreads,
        }
    }

    /// Provides a new Burning
    pub fn to_burning(&self) -> Burning {
        Burning::new(self.duration, self.tick_rate)
    }
}

/// The Burns blueprint.
#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct BurnsBlueprint(pub Burns);

/// Component for particles that have the capacity to burn.
#[derive(Clone, Eq, PartialEq, Debug, Default, Component, Reflect)]
pub struct Burning {
    /// The Burning timer.
    pub timer: Timer,
    /// The tick rate timer.
    pub tick_timer: Timer,
}

#[allow(dead_code)]
impl Burning {
    /// Creates a new Burning.
    pub fn new(duration: Duration, tick_rate: Duration) -> Burning {
        Burning {
            timer: Timer::new(duration, TimerMode::Repeating),
            tick_timer: Timer::new(tick_rate, TimerMode::Repeating),
        }
    }

    /// Ticks the burn timer.
    pub fn tick(&mut self, duration: Duration) {
        self.timer.tick(duration);
        self.tick_timer.tick(duration);
    }

    /// Resets the Burning status
    pub fn reset(&mut self) {
        self.timer.reset();
        self.tick_timer.reset();
    }
}

/// The Burning blueprint
#[derive(Clone, Eq, PartialEq, Debug, Default, Component, Reflect)]
pub struct BurningBlueprint(pub Burning);

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

/// Triggers a particle to reset its Burning information to its parent's.
#[derive(Event)]
pub struct ResetBurningEvent {
    /// The entity to reset data for.
    pub entity: Entity,
}

/// Triggers a particle to reset its Burns information to its parent's.
#[derive(Event)]
pub struct ResetBurnsEvent {
    /// The entity to reset data for.
    pub entity: Entity,
}

/// Triggers a particle to reset its Fire information to its parent's.
#[derive(Event)]
pub struct ResetFireEvent {
    /// The entity to reset data for.
    pub entity: Entity,
}


/// Observer for resetting a particle's Fire information to its parent's.
pub fn on_reset_fire(
    trigger: Trigger<ResetFireEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&FireBlueprint>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(fire) = parent_query.get(parent.get()).unwrap() {
            commands.entity(trigger.event().entity).insert(fire.0.clone());
        } else {
            commands.entity(trigger.event().entity).remove::<Fire>();
        }
    }
}

/// Observer for resetting a particle's Burns information to its parent's.
pub fn on_reset_burns(
    trigger: Trigger<ResetBurnsEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&BurnsBlueprint>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(burns) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(burns.0.clone());
        } else {
            commands.entity(trigger.event().entity).remove::<Burns>();
        }
    }
}

/// Observer for resetting a particle's Burning information to its parent's.
pub fn on_reset_burning(
    trigger: Trigger<ResetBurningEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&BurningBlueprint>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(burning) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(burning.0.clone());
        } else {
            commands.entity(trigger.event().entity).remove::<Burning>();
        }
    }
}
