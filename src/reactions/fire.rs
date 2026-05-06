use std::time::Duration;

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use super::ReactionRng;
use crate::{
    core::{
        ChanceLifetime, ChunkDirtyState, ChunkIndex, DespawnParticleSignal, GridPosition, Particle,
        ParticleMap, ParticleRng, ParticleSyncExt, ParticleSystems, ParticleTypeRegistry,
        SpawnParticleSignal,
    },
    movement::Movement,
};

/// Marker component for particles that ignite nearby neighbors.
///
/// When a particle with this component is in a dirty rect, a spatial query is performed
/// within the configured [`radius`](Fire::radius). Any neighbor with [`Flammable`] (that isn't
/// already [`Burning`]) will be ignited based on the neighbor's [`Flammable::chance_to_ignite`]
/// probability.
///
/// Can be placed on a [`ParticleType`](crate::ParticleType) for permanent fire sources (e.g. lava), or added
/// dynamically when a particle ignites via [`Flammable::spreads_fire`].
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::core::ParticleType;
/// use bevy_falling_sand::reactions::Fire;
///
/// fn setup(mut commands: Commands) {
///     commands.spawn((ParticleType::new("Lava"), Fire::default()));
/// }
/// ```
#[derive(Component, Copy, Clone, PartialEq, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
#[type_path = "bfs_reactions::particle"]
pub struct Fire {
    /// The radius within which this fire source can ignite neighboring particles.
    /// Defaults to 1.0 (immediate neighbors).
    pub radius: f32,
}

impl Default for Fire {
    fn default() -> Self {
        Self { radius: 1.0 }
    }
}

pub(super) struct FirePlugin;

impl Plugin for FirePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Fire>()
            .register_type::<Burning>()
            .register_type::<Flammable>()
            .register_particle_sync_component::<Fire>()
            .register_particle_sync_component::<Flammable>()
            .register_particle_sync_component::<Burning>()
            .add_systems(
                PreUpdate,
                handle_ignites_on_spawn.after(ParticleSystems::Registration),
            )
            .add_systems(
                PostUpdate,
                (
                    handle_burning.in_set(ParticleSystems::Simulation),
                    handle_fire.after(handle_burning),
                )
                    .in_set(ParticleSystems::Simulation),
            );
    }
}

/// Component which indicates an entity has the capacity to burn.
///
/// When a neighboring [`Fire`] source contacts this particle, it may ignite based on
/// [`chance_to_ignite`](Flammable::chance_to_ignite). A [`ReactionRng`] component
/// is automatically inserted when this component is added.
///
/// # Examples
///
/// ```no_run
/// use std::time::Duration;
/// use bevy::prelude::*;
/// use bevy_falling_sand::core::ParticleType;
/// use bevy_falling_sand::reactions::Flammable;
///
/// fn setup(mut commands: Commands) {
///     commands.spawn((
///         ParticleType::new("Wood"),
///         Flammable {
///             duration: Duration::from_secs(5),
///             tick_rate: Duration::from_millis(100),
///             chance_to_ignite: 0.3,
///             spreads_fire: true,
///             ..default()
///         },
///     ));
/// }
/// ```
#[derive(Component, Clone, PartialEq, Debug, Reflect)]
#[component(on_add = Flammable::on_add)]
#[reflect(Component)]
#[type_path = "bfs_reactions::particle"]
pub struct Flammable {
    /// The duration the entity will burn for.
    pub duration: Duration,
    /// The tick rate for the burn effect.
    pub tick_rate: Duration,
    /// The chance the entity will be destroyed per tick while burning.
    /// A [`ChanceLifetime`] component will be added when the particle ignites.
    /// Set to 0.0 to prevent particles from being destroyed by burning.
    pub chance_despawn_per_tick: f64,
    /// Indicates the burn effect might produce a new particle type.
    pub reaction: Option<BurnProduct>,
    /// The chance this particle will be ignited per contact check from a neighboring
    /// [`Fire`] source. Set to 0.0 to prevent contact ignition.
    pub chance_to_ignite: f64,
    /// Whether this particle spreads fire to neighbors while burning.
    pub spreads_fire: bool,
    /// The radius of the [`Fire`] source created when this particle ignites and
    /// `spreads_fire` is true. Defaults to 1.0.
    pub spread_radius: f32,
    /// Whether this particle should be despawned when it stops burning.
    pub despawn_on_extinguish: bool,
    /// Indicates whether the burning entity should ignite upon being spawned.
    pub ignites_on_spawn: bool,
}

impl Default for Flammable {
    fn default() -> Self {
        Self {
            duration: Duration::default(),
            tick_rate: Duration::default(),
            chance_despawn_per_tick: 0.0,
            reaction: None,
            chance_to_ignite: 0.0,
            spreads_fire: false,
            spread_radius: 1.0,
            despawn_on_extinguish: false,
            ignites_on_spawn: false,
        }
    }
}

impl Flammable {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        if !world.entity(context.entity).contains::<ReactionRng>() {
            world
                .commands()
                .entity(context.entity)
                .insert(ReactionRng::default());
        }
    }
}

impl Flammable {
    /// Initialize a new `Flammable` with a specific duration, tick rate, and various optional parameters
    /// which exert influence on behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use bevy_falling_sand::reactions::Flammable;
    ///
    /// let burns = Flammable::new(
    ///     Duration::from_secs(5),
    ///     Duration::from_millis(100),
    ///     0.1, None, 0.3, true, 1.0, false, false,
    /// );
    /// assert_eq!(burns.chance_to_ignite, 0.3);
    /// ```
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        duration: Duration,
        tick_rate: Duration,
        chance_destroy_per_tick: f64,
        reaction: Option<BurnProduct>,
        chance_to_ignite: f64,
        spreads_fire: bool,
        spread_radius: f32,
        despawn_on_extinguish: bool,
        ignites_on_spawn: bool,
    ) -> Self {
        Self {
            duration,
            tick_rate,
            chance_despawn_per_tick: chance_destroy_per_tick,
            reaction,
            chance_to_ignite,
            spreads_fire,
            spread_radius,
            despawn_on_extinguish,
            ignites_on_spawn,
        }
    }

    /// Initialize a new [`Burning`] from [`Flammable`] data.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use bevy::prelude::default;
    /// use bevy_falling_sand::reactions::{Flammable, Burning};
    ///
    /// let burns = Flammable {
    ///     duration: Duration::from_secs(5),
    ///     tick_rate: Duration::from_millis(100),
    ///     ..default()
    /// };
    /// let burning = burns.to_burning();
    /// assert!(!burning.timer.is_finished());
    /// ```
    #[must_use]
    pub fn to_burning(&self) -> Burning {
        Burning::new(self.duration, self.tick_rate)
    }
}

/// Component which indicates an entity is actively burning.
///
/// Created from a [`Flammable`] component via [`Flammable::to_burning`]. Contains timers that
/// track the remaining burn duration and tick rate for burn effects.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use bevy_falling_sand::reactions::Burning;
///
/// let burning = Burning::new(Duration::from_secs(5), Duration::from_millis(100));
/// assert!(!burning.timer.is_finished());
/// ```
#[derive(Component, Clone, Default, Eq, PartialEq, Debug, Reflect)]
#[reflect(Component)]
#[type_path = "bfs_reactions::particle"]
pub struct Burning {
    /// The duration the entity will burn for.
    pub timer: Timer,
    /// The tick rate for the burn effect.
    pub tick_timer: Timer,
}

impl Burning {
    /// Initialize a new `Burning` with a specific duration and tick rate.
    ///
    /// # Panics
    ///
    /// Panics if `duration` is not greater than `tick_rate`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use bevy_falling_sand::reactions::Burning;
    ///
    /// let burning = Burning::new(Duration::from_secs(5), Duration::from_millis(100));
    /// assert!(!burning.timer.is_finished());
    /// ```
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
}

/// Component indicating a burning particle entity produces a byproduct.
///
/// When attached to a burning particle, there is a per-tick chance of spawning
/// the specified particle type at a neighboring position.
///
/// # Examples
///
/// ```
/// use bevy_falling_sand::core::Particle;
/// use bevy_falling_sand::reactions::BurnProduct;
///
/// let product = BurnProduct::new(Particle::new("Smoke"), 0.1);
/// assert_eq!(product.produces.name, "Smoke");
/// assert_eq!(product.chance_to_produce, 0.1);
/// ```
#[derive(Component, Clone, Default, PartialEq, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
#[type_path = "bfs_reactions::particle"]
pub struct BurnProduct {
    /// What the particle will produce when the reaction occurs.
    pub produces: Particle,
    /// The chance the reaction will occur per frame.
    pub chance_to_produce: f64,
}

impl BurnProduct {
    /// Initialize a new `BurnProduct` with a specific particle and chance to produce it.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_falling_sand::core::Particle;
    /// use bevy_falling_sand::reactions::BurnProduct;
    ///
    /// let product = BurnProduct::new(Particle::new("Ash"), 0.05);
    /// assert_eq!(product.chance_to_produce, 0.05);
    /// ```
    #[must_use]
    pub const fn new(produces: Particle, chance_to_produce: f64) -> Self {
        Self {
            produces,
            chance_to_produce,
        }
    }

    /// Produce a new particle based on the movement priority of the produced particle type.
    pub(crate) fn produce(
        &self,
        rng: &mut ReactionRng,
        position: GridPosition,
        registry: &crate::ParticleTypeRegistry,
        movement_query: &Query<&Movement>,
        msgw_spawn_particle: &mut MessageWriter<SpawnParticleSignal>,
    ) {
        if !self.chance(rng) {
            return;
        }

        let Some(entity) = registry.get(&self.produces.name) else {
            return;
        };

        let movement = movement_query
            .get(*entity)
            .cloned()
            .unwrap_or_else(|_| Movement::empty());

        let positions: Vec<IVec2> = movement
            .moore_neighbors()
            .map(|offset| position.0 + offset)
            .collect();

        if !positions.is_empty() {
            msgw_spawn_particle.write(SpawnParticleSignal::try_multiple(
                self.produces.clone(),
                positions,
            ));
        }
    }

    fn chance(&self, rng: &mut ReactionRng) -> bool {
        rng.chance(self.chance_to_produce)
    }
}

fn handle_ignites_on_spawn(
    mut commands: Commands,
    particle_query: Query<(Entity, &Flammable), Added<Flammable>>,
) {
    for (entity, burns) in &particle_query {
        if burns.ignites_on_spawn {
            let mut entity_commands = commands.entity(entity);
            entity_commands.insert(burns.to_burning());
            if burns.chance_despawn_per_tick > 0.0 {
                entity_commands.insert(ChanceLifetime::with_tick_rate(
                    burns.chance_despawn_per_tick,
                    burns.tick_rate,
                ));
            }
            if let Some(reaction) = &burns.reaction {
                entity_commands.insert(reaction.clone());
            }
            if burns.spreads_fire {
                entity_commands.insert(Fire {
                    radius: burns.spread_radius,
                });
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
fn handle_burning(
    mut commands: Commands,
    mut burning_query: Query<
        (
            Entity,
            &Flammable,
            &mut Burning,
            Option<&BurnProduct>,
            &GridPosition,
            &mut ReactionRng,
        ),
        With<Particle>,
    >,
    time: Res<Time>,
    registry: Res<ParticleTypeRegistry>,
    movement_query: Query<&Movement>,
    mut msgw_spawn_particle: MessageWriter<SpawnParticleSignal>,
    mut msgw_despawn: MessageWriter<DespawnParticleSignal>,
) {
    burning_query.iter_mut().for_each(
        |(entity, burns, mut burning, burn_product, position, mut rng)| {
            if burning.timer.tick(time.delta()).is_finished() {
                if burns.despawn_on_extinguish {
                    msgw_despawn.write(DespawnParticleSignal::from_entity(entity));
                    return;
                }
                commands.entity(entity).try_remove::<(Burning, Fire)>();
                if burns.reaction.is_some() {
                    commands.entity(entity).try_remove::<BurnProduct>();
                }
                if burns.chance_despawn_per_tick > 0.0 {
                    commands.entity(entity).try_remove::<ChanceLifetime>();
                }
                return;
            }
            if burning.tick_timer.tick(time.delta()).is_finished()
                && let Some(burn_product) = burn_product
            {
                burn_product.produce(
                    &mut rng,
                    *position,
                    &registry,
                    &movement_query,
                    &mut msgw_spawn_particle,
                );
            }
        },
    );
}

/// Spreads fire via spatial queries within dirty rects.
///
/// Only processes particles that have [`Fire`] and lie inside a dirty rect.
/// For each such particle, a spatial query is performed within its [`Fire::radius`]. Any
/// neighbor with [`Flammable`] (that isn't already [`Burning`]) will be ignited based on the
/// neighbor's
/// [`Flammable::chance_to_ignite`] probability. If the ignited neighbor has
/// [`Flammable::spreads_fire`] set, a [`Fire`] component is added so the chain continues.
#[allow(clippy::type_complexity, clippy::needless_pass_by_value)]
fn handle_fire(
    mut commands: Commands,
    map: Res<ParticleMap>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<&mut ChunkDirtyState>,
    fire_query: Query<&Fire>,
    burns_query: Query<&Flammable, (With<Particle>, Without<Burning>)>,
    mut rng: ResMut<bevy_turborand::prelude::GlobalRng>,
) {
    use bevy_turborand::DelegatedRng;
    for (_coord, chunk_entity) in chunk_index.iter() {
        let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity) else {
            continue;
        };

        let Some(dirty_rect) = dirty_state.current else {
            continue;
        };

        for y in dirty_rect.min.y..=dirty_rect.max.y {
            for x in dirty_rect.min.x..=dirty_rect.max.x {
                let pos = IVec2::new(x, y);

                let Ok(Some(entity)) = map.get_copied(pos) else {
                    continue;
                };

                let Ok(fire) = fire_query.get(entity) else {
                    continue;
                };

                for (neighbor_pos, neighbor_entity) in map.within_radius(pos, fire.radius) {
                    if neighbor_pos == pos {
                        continue;
                    }

                    let Ok(burns) = burns_query.get(neighbor_entity) else {
                        continue;
                    };

                    if !rng.chance(burns.chance_to_ignite) {
                        dirty_state.mark_dirty(pos);
                        continue;
                    }

                    let mut entity_commands = commands.entity(neighbor_entity);
                    entity_commands.insert(burns.to_burning());
                    if burns.chance_despawn_per_tick > 0.0 {
                        entity_commands.insert(ChanceLifetime::with_tick_rate(
                            burns.chance_despawn_per_tick,
                            burns.tick_rate,
                        ));
                    }
                    if let Some(reaction) = &burns.reaction {
                        entity_commands.insert(reaction.clone());
                    }
                    if burns.spreads_fire {
                        entity_commands.insert(Fire {
                            radius: burns.spread_radius,
                        });
                    }
                }
            }
        }
    }
}
