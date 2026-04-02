use bevy::prelude::*;

use crate::ParticleSyncExt;

pub(super) struct ComponentsPlugin;

impl Plugin for ComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StaticRigidBodyParticle>()
            .register_particle_sync_component::<StaticRigidBodyParticle>();
    }
}

/// Marker component for particles that contribute to static rigid body collision.
///
/// Add this to a [`ParticleType`](crate::ParticleType) entity so that all
/// particles of that type are included when generating per-chunk collision meshes.
/// Typically used for solid particles like walls, rocks, and settled sand.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::core::ParticleType;
/// use bevy_falling_sand::physics::StaticRigidBodyParticle;
///
/// fn setup(mut commands: Commands) {
///     commands.spawn((
///         ParticleType::new("Stone"),
///         StaticRigidBodyParticle,
///     ));
/// }
/// ```
#[derive(Component, Copy, Clone, Default, Debug, Reflect)]
#[reflect(Component)]
#[type_path = "bfs_physics"]
pub struct StaticRigidBodyParticle;
