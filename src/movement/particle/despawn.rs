use crate::core::ParticleType;
use crate::{core::DespawnParticleTypeChildrenSignal, movement::Movement};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub(super) struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DespawnDynamicParticlesSignal>()
            .add_message::<DespawnStaticParticlesSignal>()
            .add_observer(on_despawn_dynamic_particles)
            .add_observer(on_despawn_static_particles)
            .add_systems(
                PreUpdate,
                (
                    msgr_despawn_dynamic_particles,
                    msgr_despawn_static_particles,
                ),
            );
    }
}

/// Signal for despawning all dynamic (moveable) particles from the world.
///
/// Triggers a [`DespawnParticleTypeChildrenSignal`] for each particle type
/// that has a [`Movement`] component.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::movement::DespawnDynamicParticlesSignal;
///
/// fn clear_dynamic(mut writer: MessageWriter<DespawnDynamicParticlesSignal>) {
///     writer.write(DespawnDynamicParticlesSignal);
/// }
/// ```
#[derive(Event, Message, Clone, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub struct DespawnDynamicParticlesSignal;

/// Signal for despawning all static (non-moving) particles from the world.
///
/// Triggers a [`DespawnParticleTypeChildrenSignal`] for each particle type
/// without a [`Movement`] component.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::movement::DespawnStaticParticlesSignal;
///
/// fn clear_static(mut writer: MessageWriter<DespawnStaticParticlesSignal>) {
///     writer.write(DespawnStaticParticlesSignal);
/// }
/// ```
#[derive(Event, Message, Clone, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub struct DespawnStaticParticlesSignal;

fn msgr_despawn_dynamic_particles(
    mut msgr_clear_dynamic_particles: MessageReader<DespawnDynamicParticlesSignal>,
    mut msgr_clear_particle_type_children: MessageWriter<DespawnParticleTypeChildrenSignal>,
    dynamic_particle_types_query: Query<Entity, (With<Movement>, With<ParticleType>)>,
) {
    msgr_clear_dynamic_particles.read().for_each(|_| {
        dynamic_particle_types_query.iter().for_each(|entity| {
            msgr_clear_particle_type_children.write(
                DespawnParticleTypeChildrenSignal::from_parent_handle(entity),
            );
        });
    });
}

fn on_despawn_dynamic_particles(
    _trigger: On<DespawnDynamicParticlesSignal>,
    mut commands: Commands,
    dynamic_particle_types_query: Query<Entity, (With<Movement>, With<ParticleType>)>,
) {
    dynamic_particle_types_query.iter().for_each(|entity| {
        commands.trigger(DespawnParticleTypeChildrenSignal::from_parent_handle(
            entity,
        ));
    });
}

fn msgr_despawn_static_particles(
    mut msgr_clear_static_particles: MessageReader<DespawnStaticParticlesSignal>,
    mut msgr_clear_particle_type_children: MessageWriter<DespawnParticleTypeChildrenSignal>,
    static_particle_types_query: Query<Entity, (Without<Movement>, With<ParticleType>)>,
) {
    msgr_clear_static_particles.read().for_each(|_| {
        static_particle_types_query.iter().for_each(|entity| {
            msgr_clear_particle_type_children.write(
                DespawnParticleTypeChildrenSignal::from_parent_handle(entity),
            );
        });
    });
}

fn on_despawn_static_particles(
    _trigger: On<DespawnStaticParticlesSignal>,
    mut commands: Commands,
    static_particle_types_query: Query<Entity, (Without<Movement>, With<ParticleType>)>,
) {
    static_particle_types_query.iter().for_each(|entity| {
        commands.trigger(DespawnParticleTypeChildrenSignal::from_parent_handle(
            entity,
        ));
    });
}
