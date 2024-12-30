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
        app.add_observer(on_reset_fire)
            .add_observer(on_reset_burns)
            .add_observer(on_reset_burning);
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Fire {
    pub burn_radius: f32,
    pub chance_to_spread: f64,
    pub destroys_on_spread: bool,
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct FireBlueprint(pub Fire);

#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Burns {
    pub duration: Duration,
    pub tick_rate: Duration,
    pub chance_destroy_per_tick: Option<f64>,
    pub reaction: Option<Reacting>,
    pub color: Option<ParticleColor>,
    pub spreads: Option<Fire>,
}

impl Burns {
    #![allow(dead_code)]
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

    pub fn to_burning(&self) -> Burning {
        Burning::new(self.duration, self.tick_rate)
    }
}

#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct BurnsBlueprint(pub Burns);

#[derive(Clone, Eq, PartialEq, Debug, Default, Component, Reflect)]
pub struct Burning {
    pub timer: Timer,
    pub tick_timer: Timer,
}

#[allow(dead_code)]
impl Burning {
    pub fn new(duration: Duration, tick_rate: Duration) -> Burning {
        Burning {
            timer: Timer::new(duration, TimerMode::Repeating),
            tick_timer: Timer::new(tick_rate, TimerMode::Repeating),
        }
    }

    pub fn tick(&mut self, duration: Duration) {
        self.timer.tick(duration);
        self.tick_timer.tick(duration);
    }

    pub fn reset(&mut self) {
        self.timer.reset();
        self.tick_timer.reset();
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Component, Reflect)]
pub struct BurningBlueprint(pub Burning);

#[derive(Clone, PartialEq, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Reacting {
    pub produces: Particle,
    pub chance_to_produce: f64,
}

impl Reacting {
    pub fn new(produces: Particle, chance_to_produce: f64) -> Reacting {
        Reacting {
            produces,
            chance_to_produce,
        }
    }

    pub fn produce(
        &self,
        commands: &mut Commands,
        rng: &mut ReactionRng,
        coordinates: &Coordinates,
    ) {
        if self.chance(rng) {
            commands.spawn((
                self.produces.clone(),
                Transform::from_xyz(coordinates.0.x as f32, coordinates.0.y as f32 + 1., 0.),
            ));
        }
    }

    pub fn chance(&self, rng: &mut ReactionRng) -> bool {
        rng.chance(self.chance_to_produce)
    }
}

#[derive(Event)]
pub struct ResetBurningEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct ResetBurnsEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct ResetFireEvent {
    pub entity: Entity,
}

pub fn on_reset_fire(
    trigger: Trigger<ResetFireEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&FireBlueprint>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(fire) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(fire.0.clone());
        } else {
            commands.entity(trigger.event().entity).remove::<Fire>();
        }
    }
}

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
