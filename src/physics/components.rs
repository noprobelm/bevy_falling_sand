use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ParticleSyncExt;

pub(super) struct ComponentsPlugin;

impl Plugin for ComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StaticRigidBodyParticle>()
            .register_type::<DynamicRigidBodySignal>()
            .register_particle_sync_component::<StaticRigidBodyParticle>()
            .add_message::<DynamicRigidBodySignal>();
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

/// Signal to promote a [`Particle`](crate::Particle) entity into a dynamic rigid body.
///
/// When processed, the particle is removed from the falling sand simulation (its
/// [`GridPosition`](crate::GridPosition) is removed and it is taken out of the
/// [`ParticleMap`](crate::ParticleMap)). A separate rigid body entity is spawned at the
/// particle's former position with the configured velocity and gravity.
///
/// Each frame, the rigid body's position is checked against the `ParticleMap`. If any
/// neighboring cell (or the cell itself) is occupied or at the map edge, the rigid body is
/// despawned and the original particle is restored to the simulation at the nearest vacant
/// position.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::core::{Particle, GridPosition};
/// use bevy_falling_sand::physics::DynamicRigidBodySignal;
///
/// fn launch_particle(
///     mut writer: MessageWriter<DynamicRigidBodySignal>,
///     query: Query<Entity, With<Particle>>,
/// ) {
///     for entity in &query {
///         writer.write(
///             DynamicRigidBodySignal::new(entity)
///                 .with_linear_velocity(Vec2::new(50.0, 100.0)),
///         );
///     }
/// }
/// ```
#[derive(Event, Message, Copy, Clone, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Debug)]
#[type_path = "bfs_physics"]
pub struct DynamicRigidBodySignal {
    pub(super) entity: Entity,
    pub(super) linear_velocity: Vec2,
    pub(super) angular_velocity: f32,
    pub(super) gravity_scale: f32,
}

impl DynamicRigidBodySignal {
    /// Create a signal targeting the given particle entity with default physics parameters.
    #[must_use]
    pub const fn new(entity: Entity) -> Self {
        Self {
            entity,
            linear_velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            gravity_scale: 1.0,
        }
    }

    /// Set the initial linear velocity.
    #[must_use]
    pub const fn with_linear_velocity(mut self, velocity: Vec2) -> Self {
        self.linear_velocity = velocity;
        self
    }

    /// Set the initial angular velocity in radians per second.
    #[must_use]
    pub const fn with_angular_velocity(mut self, velocity: f32) -> Self {
        self.angular_velocity = velocity;
        self
    }

    /// Set the gravity scale.
    #[must_use]
    pub const fn with_gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }
}

/// Placed on the spawned rigid body entity. Links back to the suspended particle.
#[derive(Component, Copy, Clone, Debug)]
pub struct DynamicRigidBodyProxy {
    /// The particle entity that this rigid body is a proxy for.
    pub particle_entity: Entity,
}

/// Placed on the suspended particle entity. Links to its rigid body proxy.
#[derive(Component, Copy, Clone, Debug)]
pub struct SuspendedParticle {
    /// The rigid body entity that acts as a physics proxy for this particle.
    pub rigid_body_entity: Entity,
}
