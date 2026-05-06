use bevy::prelude::*;
use bevy_turborand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    core::{
        AttachedToParticleType, ChunkDirtyState, ChunkIndex, Particle, ParticleMap,
        ParticleSystems, ParticleType, ParticleTypeRegistry, SpawnParticleSignal,
    },
    movement::ParticleMovementSystems,
};

pub(super) struct ContactPlugin;

impl Plugin for ContactPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ContactReaction>()
            .register_type::<ContactRule>()
            .register_type::<Consumes>()
            .add_observer(on_contact_reaction_added)
            .add_observer(on_particle_type_added)
            .add_systems(
                PostUpdate,
                (
                    resolve_changed_contact_reactions,
                    handle_contact_reactions.after(resolve_changed_contact_reactions),
                )
                    .in_set(ParticleSystems::Simulation)
                    .after(ParticleMovementSystems),
            );
    }
}

/// Defines contact reaction rulesets for a particle type.
///
/// Each rule describes what happens when a particle with a ruleset comes within a specified radius
/// of another.
///
/// A radius of `1` (defualt) is suitable for most cases. Increasing the radius adds overhead
/// proportional to the number of dirty particles of this type. Find a balance between appearance
/// and selected radius.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::reactions::{ContactReaction, ContactRule, Consumes};
/// use bevy_falling_sand::core::{Particle, ParticleType};
///
/// fn setup(mut commands: Commands) {
///     commands.spawn((
///         ParticleType::new("Wood"),
///         ContactReaction {
///             rules: vec![ContactRule {
///                 target: Particle::new("Lava"),
///                 becomes: Particle::new("Fire"),
///                 chance: 0.8,
///                 radius: 1.0,
///                 consumes: Consumes::Source,
///             }],
///         },
///     ));
/// }
/// ```
#[derive(Component, Clone, Default, PartialEq, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Component, Default)]
#[type_path = "bfs_reactions::contact"]
pub struct ContactReaction {
    /// The list of contact rules for this particle type.
    pub rules: Vec<ContactRule>,
}

/// A single contact reaction rule.
///
/// # Examples
///
/// ```
/// use bevy_falling_sand::core::Particle;
/// use bevy_falling_sand::reactions::{ContactRule, Consumes};
///
/// let rule = ContactRule {
///     target: Particle::new("Water"),
///     becomes: Particle::new("Steam"),
///     chance: 0.5,
///     radius: 1.0,
///     consumes: Consumes::Target,
/// };
/// assert_eq!(rule.chance, 0.5);
/// assert_eq!(rule.consumes, Consumes::Target);
/// ```
#[derive(Clone, PartialEq, Debug, Reflect, Serialize, Deserialize)]
pub struct ContactRule {
    /// The particle type to react with on contact.
    pub target: Particle,
    /// The particle type the reacting particle becomes.
    pub becomes: Particle,
    /// Probability per contact per frame (0.0 to 1.0).
    pub chance: f64,
    /// The radius within which to check for the target particle.
    /// Defaults to 1.0 (immediate neighbors).
    #[serde(default = "ContactRule::default_radius")]
    pub radius: f32,
    /// Which particle is consumed (replaced by `becomes`) when the reaction occurs.
    #[serde(default)]
    #[reflect(default)]
    pub consumes: Consumes,
}

/// Controls which particle is consumed (replaced by [`ContactRule::becomes`]) when
/// a contact reaction fires.
#[derive(Clone, Copy, Default, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub enum Consumes {
    /// The source particle (the one with the [`ContactReaction`]) is consumed.
    #[default]
    Source,
    /// The target particle (the neighbor that triggered the reaction) is consumed.
    Target,
}

impl Default for ContactRule {
    fn default() -> Self {
        Self {
            target: Particle::default(),
            becomes: Particle::default(),
            chance: 0.0,
            radius: 1.0,
            consumes: Consumes::default(),
        }
    }
}

impl ContactRule {
    const fn default_radius() -> f32 {
        1.0
    }
}

/// Runtime-resolved form of [`ContactReaction`], using entity references for fast matching.
#[derive(Component, Clone, Debug)]
pub(super) struct ResolvedContactReaction {
    pub(crate) rules: Vec<ResolvedContactRule>,
}

/// A single resolved contact reaction rule with entity references.
#[derive(Clone, Debug)]
pub(super) struct ResolvedContactRule {
    pub(crate) target_type: Entity,
    pub(crate) becomes: Particle,
    pub(crate) chance: f64,
    pub(crate) radius: f32,
    pub(crate) consumes: Consumes,
}

/// Resolves `ContactReaction` names into `ResolvedContactReaction` entity references
/// when a `ContactReaction` is added to a `ParticleType` entity.
#[allow(clippy::needless_pass_by_value)]
fn on_contact_reaction_added(
    trigger: On<Add, ContactReaction>,
    mut commands: Commands,
    query: Query<&ContactReaction, With<ParticleType>>,
    registry: Res<ParticleTypeRegistry>,
) {
    let Ok(contact) = query.get(trigger.entity) else {
        return;
    };
    if let Some(resolved) = try_resolve(contact, &registry) {
        commands.entity(trigger.entity).insert(resolved);
    }
}

/// Retries resolution for all unresolved `ContactReaction` entities when a new
/// `ParticleType` is registered (its `on_add` hook populates the registry first).
#[allow(clippy::needless_pass_by_value)]
fn on_particle_type_added(
    _trigger: On<Add, ParticleType>,
    mut commands: Commands,
    query: Query<
        (Entity, &ContactReaction),
        (With<ParticleType>, Without<ResolvedContactReaction>),
    >,
    registry: Res<ParticleTypeRegistry>,
) {
    for (entity, contact) in &query {
        if let Some(resolved) = try_resolve(contact, &registry) {
            commands.entity(entity).insert(resolved);
        }
    }
}

/// Attempts to resolve all rules in a `ContactReaction`. Returns `None` if any
/// target or becomes name cannot be found in the registry.
fn try_resolve(
    contact: &ContactReaction,
    registry: &ParticleTypeRegistry,
) -> Option<ResolvedContactReaction> {
    let mut resolved_rules = Vec::with_capacity(contact.rules.len());

    for rule in &contact.rules {
        let target_type = *registry.get(&rule.target.name)?;
        let _ = registry.get(&rule.becomes.name)?;
        resolved_rules.push(ResolvedContactRule {
            target_type,
            becomes: rule.becomes.clone(),
            chance: rule.chance,
            radius: rule.radius,
            consumes: rule.consumes,
        });
    }

    Some(ResolvedContactReaction {
        rules: resolved_rules,
    })
}

/// Re-resolves `ResolvedContactReaction` when `ContactReaction` is mutated after initial add.
#[allow(clippy::needless_pass_by_value)]
fn resolve_changed_contact_reactions(
    mut commands: Commands,
    query: Query<
        (Entity, &ContactReaction),
        (
            Changed<ContactReaction>,
            With<ParticleType>,
            With<ResolvedContactReaction>,
        ),
    >,
    registry: Res<ParticleTypeRegistry>,
) {
    for (entity, contact) in &query {
        if let Some(resolved) = try_resolve(contact, &registry) {
            commands.entity(entity).insert(resolved);
        }
    }
}

/// Processes contact reactions for particles within dirty rects each simulation tick.
#[allow(clippy::needless_pass_by_value)]
fn handle_contact_reactions(
    map: Res<ParticleMap>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<&mut ChunkDirtyState>,
    particle_query: Query<&AttachedToParticleType, With<Particle>>,
    rules_query: Query<&ResolvedContactReaction, With<ParticleType>>,
    mut rng: ResMut<GlobalRng>,
    mut msgw_spawn: MessageWriter<SpawnParticleSignal>,
) {
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

                let Ok(attached) = particle_query.get(entity) else {
                    continue;
                };

                let Ok(resolved) = rules_query.get(attached.0) else {
                    continue;
                };

                let max_radius = resolved
                    .rules
                    .iter()
                    .map(|r| r.radius)
                    .fold(0.0_f32, f32::max);

                let mut reacted = false;
                for (neighbor_pos, neighbor_entity) in map.within_radius(pos, max_radius) {
                    if reacted || neighbor_pos == pos {
                        continue;
                    }

                    let Ok(neighbor_attached) = particle_query.get(neighbor_entity) else {
                        continue;
                    };

                    let dist_sq = (neighbor_pos - pos).as_vec2().length_squared();

                    for rule in &resolved.rules {
                        if dist_sq > rule.radius * rule.radius {
                            continue;
                        }
                        if neighbor_attached.0 == rule.target_type {
                            if rng.chance(rule.chance) {
                                match rule.consumes {
                                    Consumes::Source => {
                                        msgw_spawn.write(SpawnParticleSignal::overwrite_existing(
                                            rule.becomes.clone(),
                                            pos,
                                        ));
                                    }
                                    Consumes::Target => {
                                        msgw_spawn.write(SpawnParticleSignal::overwrite_existing(
                                            rule.becomes.clone(),
                                            neighbor_pos,
                                        ));
                                    }
                                }
                                reacted = true;
                                break;
                            }
                            dirty_state.mark_dirty(pos);
                        }
                    }
                }
            }
        }
    }
}
