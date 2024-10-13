use super::{Reacting, ReactionRng};
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_spatial::SpatialAccess;
use bfs_color::*;
use bfs_core::{Coordinates, Particle, RemoveParticleEvent};
use bfs_spatial::ParticleTree;

pub struct BurningPlugin;

impl Plugin for BurningPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Fire>()
            .register_type::<Burns>()
            .register_type::<Burning>();
        app.add_systems(Update, (handle_fire, handle_burning));
    }
}

/// Marker for particle types that can inflict a burning status.
#[derive(Clone, PartialEq, PartialOrd, Debug, Default, Component, Reflect)]
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

/// Burns particles within a radius of entities that posses the `Fire` component.
pub fn handle_fire(
    mut commands: Commands,
    mut fire_query: Query<(&Fire, &Coordinates, &mut ReactionRng)>,
    burns_query: Query<(Entity, &Burns), (With<Particle>, Without<Burning>)>,
    particle_tree: Res<ParticleTree>,
) {
    fire_query
        .iter_mut()
        .for_each(|(fire, coordinates, mut rng)| {
            let mut destroy_fire: bool = false;
            if !rng.chance(fire.chance_to_spread) {
                return;
            }
            particle_tree
                .within_distance(coordinates.0.as_vec2(), fire.burn_radius)
                .iter()
                .for_each(|(_, entity)| {
                    if let Ok((entity, burns)) = burns_query.get(entity.unwrap()) {
                        commands.entity(entity).insert(burns.to_burning());
                        if let Some(colors) = &burns.color {
                            commands.entity(entity).insert(colors.clone());
                            commands.entity(entity).insert(RandomizesColor::new(0.75));
                        }
                        if let Some(fire) = &burns.spreads {
                            commands.entity(entity).insert(fire.clone());
                        }
                        if fire.destroys_on_spread {
                            destroy_fire = true;
                        }
                    }
                });
            if destroy_fire {
                commands.trigger(RemoveParticleEvent {
                    coordinates: coordinates.0,
                    despawn: true,
                });
            }
        });
}

/// Handles all burning particles for the frame.
pub fn handle_burning(
    mut commands: Commands,
    mut burning_query: Query<(
        Entity,
        &mut Particle,
        &mut Burns,
        &mut Burning,
        &mut ReactionRng,
        &Coordinates,
    )>,
    time: Res<Time>,
) {
    burning_query.iter_mut().for_each(
        |(entity, particle, mut burns, mut burning, mut rng, coordinates)| {
            if burning.timer.tick(time.delta()).finished() {
                if burns.chance_destroy_per_tick.is_some() {
                    commands.trigger(RemoveParticleEvent {
                        coordinates: coordinates.0,
                        despawn: true,
                    })
                } else {
                    commands.entity(entity).remove::<Burning>();
                    commands.trigger(ResetParticleColorEvent { entity });
                    commands.trigger(ResetRandomizesColorEvent { entity });
                    commands.trigger(ResetFlowsColorEvent { entity });

                    particle.into_inner();
                }
                return;
            }
            if burning.tick_timer.tick(time.delta()).finished() {
                if let Some(ref mut reaction) = &mut burns.reaction {
                    reaction.produce(&mut commands, &mut rng, coordinates);
                }
                if let Some(chance_destroy) = burns.chance_destroy_per_tick {
                    if rng.chance(chance_destroy) {
                        commands.trigger(RemoveParticleEvent {
                            coordinates: coordinates.0,
                            despawn: true,
                        })
                    }
                }
            }
        },
    );
}
