use bevy::prelude::*;
use bfs_color::*;
use bfs_core::{
    impl_particle_blueprint, Coordinates, Particle, ParticleBlueprint, ParticleRegistrationEvent,
    ParticleType,
};
use std::time::Duration;

use crate::ReactionRng;

pub struct ParticleDefinitionsPlugin;

impl Plugin for ParticleDefinitionsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Fire>()
            .register_type::<Burns>()
            .register_type::<Burning>()
            .register_type::<Reacting>()
            .add_systems(Update, handle_particle_registration);
    }
}

impl_particle_blueprint!(FireBlueprint, Fire);
impl_particle_blueprint!(BurnsBlueprint, Burns);
impl_particle_blueprint!(BurningBlueprint, Burning);

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
    pub color: Option<ColorProfile>,
    pub spreads: Option<Fire>,
}

impl Burns {
    #![allow(dead_code)]
    pub fn new(
        duration: Duration,
        tick_rate: Duration,
        chance_destroy_per_tick: Option<f64>,
        reaction: Option<Reacting>,
        color: Option<ColorProfile>,
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

fn handle_particle_components(
    commands: &mut Commands,
    parent_query: &Query<
        (
            Option<&FireBlueprint>,
            Option<&BurnsBlueprint>,
            Option<&BurningBlueprint>,
        ),
        With<ParticleType>,
    >,
    particle_query: &Query<&Parent, With<Particle>>,
    entities: &Vec<Entity>,
) {
    entities.iter().for_each(|entity| {
        if let Ok(parent) = particle_query.get(*entity) {
            if let Ok((fire, burns, burning)) = parent_query.get(parent.get()) {
                commands.entity(*entity).insert(ReactionRng::default());
                if let Some(fire) = fire {
                    commands.entity(*entity).insert(fire.0.clone());
                } else {
                    commands.entity(*entity).remove::<Fire>();
                }
                if let Some(burns) = burns {
                    commands.entity(*entity).insert(burns.0.clone());
                } else {
                    commands.entity(*entity).remove::<Burns>();
                }
                if let Some(burning) = burning {
                    commands.entity(*entity).insert(burning.0.clone());
                } else {
                    commands.entity(*entity).remove::<Burning>();
                }
            }
        }
    });
}

fn handle_particle_registration(
    mut commands: Commands,
    parent_query: Query<
        (
            Option<&FireBlueprint>,
            Option<&BurnsBlueprint>,
            Option<&BurningBlueprint>,
        ),
        With<ParticleType>,
    >,
    particle_query: Query<&Parent, With<Particle>>,
    mut ev_particle_registered: EventReader<ParticleRegistrationEvent>,
) {
    ev_particle_registered.read().for_each(|ev| {
        handle_particle_components(&mut commands, &parent_query, &particle_query, &ev.entities);
    });
}
