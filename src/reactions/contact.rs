use bevy::prelude::*;
use bevy_rand::prelude::{GlobalRng, WyRand};
use serde::{Deserialize, Serialize};

use crate::{
    core::{
        AttachedToParticleType, DespawnParticleSignal, Particle, ParticleChunksMut, ParticleRngExt,
        ParticleSystems, ParticleType, ParticleTypeId, ParticleTypeRegistry, SpawnParticleSignal,
    },
    movement::ParticleMovementSystems,
};

pub(super) struct ContactPlugin;

impl Plugin for ContactPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ContactReaction>()
            .register_type::<ContactRule>()
            .register_type::<ContactOutcome>()
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
/// of another. Rule targets and products are [`ParticleTypeId`] values, so keep the relevant IDs in
/// your own resources or components when configuring reactions.
///
/// A radius of `1` (default) is suitable for most cases. Increasing the radius adds overhead
/// proportional to the number of dirty particles of this type. Find a balance between appearance
/// and selected radius.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::reactions::{ContactOutcome, ContactReaction, ContactRule};
/// use bevy_falling_sand::core::{ParticleType, ParticleTypeId};
///
/// fn setup(mut commands: Commands) {
///     let reacting_type = ParticleTypeId::new();
///     let lava = ParticleTypeId::new();
///     let fire = ParticleTypeId::new();
///
///     commands.spawn(ParticleType::from_id(lava));
///     commands.spawn(ParticleType::from_id(fire));
///     commands.spawn((
///         ParticleType::from_id(reacting_type),
///         ContactReaction::new([ContactRule {
///                 target: lava,
///                 source_outcome: ContactOutcome::Becomes(fire),
///                 target_outcome: ContactOutcome::Unchanged,
///                 chance: 0.8,
///                 radius: 1.0,
///         }]),
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

impl ContactReaction {
    /// Create contact reactions from rules.
    #[must_use]
    pub fn new(rules: impl IntoIterator<Item = ContactRule>) -> Self {
        Self {
            rules: rules.into_iter().collect(),
        }
    }

    /// Add a rule and return the updated reactions.
    #[must_use]
    pub fn with_rule(mut self, rule: ContactRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Add a contact rule.
    pub fn push(&mut self, rule: ContactRule) {
        self.rules.push(rule);
    }

    /// Return the number of contact rules.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.rules.len()
    }

    /// Return whether there are no contact rules.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Iterate through the contact rules.
    pub fn iter(&self) -> impl Iterator<Item = &ContactRule> {
        self.rules.iter()
    }

    /// Iterate mutably through the contact rules.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ContactRule> {
        self.rules.iter_mut()
    }
}

impl FromIterator<ContactRule> for ContactReaction {
    fn from_iter<T: IntoIterator<Item = ContactRule>>(iter: T) -> Self {
        Self::new(iter)
    }
}

impl Extend<ContactRule> for ContactReaction {
    fn extend<T: IntoIterator<Item = ContactRule>>(&mut self, iter: T) {
        self.rules.extend(iter);
    }
}

impl AsRef<[ContactRule]> for ContactReaction {
    fn as_ref(&self) -> &[ContactRule] {
        &self.rules
    }
}

impl IntoIterator for ContactReaction {
    type Item = ContactRule;
    type IntoIter = std::vec::IntoIter<ContactRule>;

    fn into_iter(self) -> Self::IntoIter {
        self.rules.into_iter()
    }
}

impl<'a> IntoIterator for &'a ContactReaction {
    type Item = &'a ContactRule;
    type IntoIter = std::slice::Iter<'a, ContactRule>;

    fn into_iter(self) -> Self::IntoIter {
        self.rules.iter()
    }
}

impl<'a> IntoIterator for &'a mut ContactReaction {
    type Item = &'a mut ContactRule;
    type IntoIter = std::slice::IterMut<'a, ContactRule>;

    fn into_iter(self) -> Self::IntoIter {
        self.rules.iter_mut()
    }
}

/// A single contact reaction rule.
///
/// `target` and `becomes` are [`ParticleTypeId`] values looked up against
/// [`ParticleTypeRegistry`] when the rule is resolved.
///
/// # Examples
///
/// ```no_run
/// use bevy_falling_sand::core::ParticleTypeId;
/// use bevy_falling_sand::reactions::{ContactOutcome, ContactRule};
///
/// let lava = ParticleTypeId::new();
/// let fire = ParticleTypeId::new();
/// let rule = ContactRule {
///     target: lava,
///     source_outcome: ContactOutcome::Unchanged,
///     target_outcome: ContactOutcome::Becomes(fire),
///     chance: 0.5,
///     radius: 1.0,
/// };
/// assert_eq!(rule.chance, 0.5);
/// assert_eq!(rule.target_outcome, ContactOutcome::Becomes(fire));
/// ```
#[derive(Clone, PartialEq, Debug, Reflect, Serialize, Deserialize)]
pub struct ContactRule {
    /// [`ParticleTypeId`] this rule reacts with on contact.
    pub target: ParticleTypeId,
    /// What happens to the source particle when this rule fires.
    #[serde(default)]
    #[reflect(default)]
    pub source_outcome: ContactOutcome,
    /// What happens to the target particle when this rule fires.
    #[serde(default)]
    #[reflect(default)]
    pub target_outcome: ContactOutcome,
    /// Probability per contact per frame (0.0 to 1.0).
    pub chance: f64,
    /// The radius within which to check for the target particle.
    /// Defaults to 1.0 (immediate neighbors).
    #[serde(default = "ContactRule::default_radius")]
    pub radius: f32,
}

/// Describes what happens to one participant when a contact reaction fires.
#[derive(Clone, Copy, Default, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub enum ContactOutcome {
    /// Keep the particle unchanged.
    #[default]
    Unchanged,
    /// Destroy the particle.
    Destroy,
    /// Replace the particle with the specified type.
    Becomes(ParticleTypeId),
}

impl Default for ContactRule {
    fn default() -> Self {
        Self {
            target: ParticleTypeId::default(),
            source_outcome: ContactOutcome::default(),
            target_outcome: ContactOutcome::default(),
            chance: 0.0,
            radius: 1.0,
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
    pub(crate) source_outcome: ContactOutcome,
    pub(crate) target_outcome: ContactOutcome,
    pub(crate) chance: f64,
    pub(crate) radius: f32,
}

/// Resolves `ContactReaction` type IDs into `ResolvedContactReaction` entity references
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
/// target or product ID cannot be found in the registry.
fn try_resolve(
    contact: &ContactReaction,
    registry: &ParticleTypeRegistry,
) -> Option<ResolvedContactReaction> {
    let mut resolved_rules = Vec::with_capacity(contact.len());

    for rule in contact {
        let target_type = *registry.get(rule.target)?;
        for outcome in [rule.source_outcome, rule.target_outcome] {
            if let ContactOutcome::Becomes(particle_type) = outcome {
                let _ = registry.get(particle_type)?;
            }
        }
        resolved_rules.push(ResolvedContactRule {
            target_type,
            source_outcome: rule.source_outcome,
            target_outcome: rule.target_outcome,
            chance: rule.chance,
            radius: rule.radius,
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
    mut particle_chunks: ParticleChunksMut,
    particle_query: Query<&AttachedToParticleType, With<Particle>>,
    rules_query: Query<&ResolvedContactReaction, With<ParticleType>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    mut msgw_spawn: MessageWriter<SpawnParticleSignal>,
    mut msgw_despawn: MessageWriter<DespawnParticleSignal>,
) {
    particle_chunks.for_each_dirty_particle(|map, dirty_state, pos, entity| {
        let Ok(attached) = particle_query.get(entity) else {
            return;
        };

        let Ok(resolved) = rules_query.get(attached.0) else {
            return;
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
                        apply_outcome(rule.source_outcome, pos, &mut msgw_spawn, &mut msgw_despawn);
                        apply_outcome(
                            rule.target_outcome,
                            neighbor_pos,
                            &mut msgw_spawn,
                            &mut msgw_despawn,
                        );
                        reacted = true;
                        break;
                    }
                    dirty_state.mark_dirty(pos);
                }
            }
        }
    });
}

fn apply_outcome(
    outcome: ContactOutcome,
    position: IVec2,
    spawn: &mut MessageWriter<SpawnParticleSignal>,
    despawn: &mut MessageWriter<DespawnParticleSignal>,
) {
    match outcome {
        ContactOutcome::Unchanged => {}
        ContactOutcome::Destroy => {
            despawn.write(DespawnParticleSignal::from_position(position));
        }
        ContactOutcome::Becomes(particle_type) => {
            spawn.write(SpawnParticleSignal::overwrite_existing(
                particle_type,
                position,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(id: usize) -> ContactRule {
        ContactRule {
            target: ParticleTypeId::from_raw(id),
            source_outcome: ContactOutcome::Becomes(ParticleTypeId::from_raw(id + 100)),
            ..default()
        }
    }

    #[test]
    fn with_rule_and_mutable_iteration_update_rules() {
        let mut reactions = ContactReaction::default().with_rule(rule(1));
        for rule in &mut reactions {
            rule.chance = 0.5;
        }

        assert_eq!(reactions.len(), 1);
        assert_eq!(reactions.iter().next().unwrap().chance, 0.5);
    }

    #[test]
    fn outcomes_emit_independent_spawn_and_despawn_signals() {
        fn emit_outcomes(
            mut spawn: MessageWriter<SpawnParticleSignal>,
            mut despawn: MessageWriter<DespawnParticleSignal>,
        ) {
            apply_outcome(
                ContactOutcome::Unchanged,
                IVec2::ZERO,
                &mut spawn,
                &mut despawn,
            );
            apply_outcome(
                ContactOutcome::Destroy,
                IVec2::ONE,
                &mut spawn,
                &mut despawn,
            );
            apply_outcome(
                ContactOutcome::Becomes(ParticleTypeId::from_raw(42)),
                IVec2::X,
                &mut spawn,
                &mut despawn,
            );
        }

        let mut app = App::new();
        app.add_message::<SpawnParticleSignal>()
            .add_message::<DespawnParticleSignal>()
            .add_systems(Update, emit_outcomes);
        app.update();

        let spawned: Vec<_> = app
            .world_mut()
            .resource_mut::<Messages<SpawnParticleSignal>>()
            .drain()
            .collect();
        let despawned: Vec<_> = app
            .world_mut()
            .resource_mut::<Messages<DespawnParticleSignal>>()
            .drain()
            .collect();

        assert_eq!(spawned.len(), 1);
        assert_eq!(spawned[0].particle_type, ParticleTypeId::from_raw(42));
        assert_eq!(spawned[0].positions, [IVec2::X]);
        assert!(spawned[0].overwrite_existing);
        assert_eq!(despawned.len(), 1);
    }
}
