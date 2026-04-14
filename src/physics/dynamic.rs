//! Dynamic rigid body promotion and rejoining for falling sand particles.
//!
//! Sending a [`PromoteDynamicRigidBodyParticle`] removes a particle from the simulation and spawns a
//! physics-driven rigid body proxy. Each frame the proxy is checked: when it neighbours another
//! particle, reaches a map edge, or its lifetime expires below its speed threshold, the proxy is
//! despawned and the original particle is restored at the nearest vacant position.

use std::time::Duration;

use avian2d::prelude::*;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::{
    ChunkDirtyState, ChunkIndex, GridPosition, Particle, ParticleMap, SyncParticleSignal,
};
use crate::movement::{AirResistance, Density, Momentum, Movement, Speed};
use crate::render::ParticleColor;
use crate::ParticleSyncExt;

/// Collision layers for falling sand physics.
#[derive(PhysicsLayer, Default, Copy, Clone, Debug)]
pub enum DynamicParticleLayer {
    /// The default collision layer for static rigid bodies.
    #[default]
    Default,
    /// Dynamic rigid body particle proxies. By default, members do not collide with each other.
    DynamicParticle,
}

pub(super) struct DynamicPlugin;

impl Plugin for DynamicPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StaticRigidBodyParticle>()
            .register_type::<PromoteDynamicRigidBodyParticle>()
            .register_particle_sync_component::<StaticRigidBodyParticle>()
            .add_message::<PromoteDynamicRigidBodyParticle>();
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

/// Signal to promote a [`Particle`] entity into a dynamic rigid body.
///
/// When processed, the particle is removed from the falling sand simulation (its
/// [`GridPosition`] is removed and it is taken out of the
/// [`ParticleMap`]). A separate rigid body entity is spawned at the
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
/// use bevy_falling_sand::physics::PromoteDynamicRigidBodyParticle;
///
/// fn launch_particle(
///     mut writer: MessageWriter<PromoteDynamicRigidBodyParticle>,
///     query: Query<Entity, With<Particle>>,
/// ) {
///     for entity in &query {
///         writer.write(
///             PromoteDynamicRigidBodyParticle::new(entity)
///                 .with_linear_velocity(Vec2::new(50.0, 100.0)),
///         );
///     }
/// }
/// ```
#[derive(Event, Message, Copy, Clone, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Debug)]
#[type_path = "bfs_physics"]
pub struct PromoteDynamicRigidBodyParticle {
    pub(super) entity: Entity,
    pub(super) linear_velocity: Vec2,
    pub(super) angular_velocity: f32,
    pub(super) gravity_scale: f32,
    pub(super) collide_with_other_dynamic: bool,
    pub(super) minimum_lifetime: Duration,
    pub(super) rejoin_speed_threshold: f32,
}

impl PromoteDynamicRigidBodyParticle {
    /// Create a signal targeting the given particle entity with default physics parameters.
    #[must_use]
    pub const fn new(entity: Entity) -> Self {
        Self {
            entity,
            linear_velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            gravity_scale: 1.0,
            collide_with_other_dynamic: false,
            minimum_lifetime: Duration::from_secs(10),
            rejoin_speed_threshold: 1.0,
        }
    }

    /// Set the linear speed (length of linear velocity) below which an expired
    /// dynamic rigid body proxy will rejoin the falling sand simulation.
    ///
    /// Defaults to `1.0`.
    #[must_use]
    pub const fn with_rejoin_speed_threshold(mut self, threshold: f32) -> Self {
        self.rejoin_speed_threshold = threshold;
        self
    }

    /// Set the maximum amount of time the particle remains a dynamic rigid body
    /// before being forcibly returned to the falling sand simulation.
    ///
    /// Defaults to 10 seconds.
    #[must_use]
    pub const fn with_max_lifetime(mut self, max_lifetime: Duration) -> Self {
        self.minimum_lifetime = max_lifetime;
        self
    }

    /// Allow this dynamic rigid body to collide with other dynamic rigid body particles.
    ///
    /// Defaults to `false`: dynamic particle proxies do not collide with each other,
    /// only with static rigid bodies.
    #[must_use]
    pub const fn with_collide_with_other_dynamic(mut self, enabled: bool) -> Self {
        self.collide_with_other_dynamic = enabled;
        self
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
    /// The grid position this proxy last placed its particle at in the [`ParticleMap`],
    /// or [`None`] if the particle is not currently in the map.
    pub last_map_position: Option<IVec2>,
}

/// Tracks how long a dynamic rigid body proxy has been alive and the speed at which
/// it should rejoin the falling sand simulation once its [`Timer`] expires.
#[derive(Component, Clone, Debug)]
pub struct DynamicRigidBodyLifetime {
    /// Ticks while the proxy is alive. Once finished, the proxy is eligible to rejoin
    /// when its linear speed falls below `rejoin_speed_threshold`.
    pub timer: Timer,
    /// Linear speed below which an expired proxy rejoins the simulation.
    pub rejoin_speed_threshold: f32,
}

/// Placed on the suspended particle entity. Links to its rigid body proxy.
#[derive(Component, Copy, Clone, Debug)]
pub struct SuspendedParticle {
    /// The rigid body entity that acts as a physics proxy for this particle.
    pub rigid_body_entity: Entity,
}

const NEIGHBOR_OFFSETS: [IVec2; 8] = [
    IVec2::new(-1, -1),
    IVec2::new(0, -1),
    IVec2::new(1, -1),
    IVec2::new(-1, 0),
    IVec2::new(1, 0),
    IVec2::new(-1, 1),
    IVec2::new(0, 1),
    IVec2::new(1, 1),
];

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub(super) fn promote_dynamic_rigid_bodies(
    mut commands: Commands,
    mut msgr: MessageReader<PromoteDynamicRigidBodyParticle>,
    mut map: ResMut<ParticleMap>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<&mut ChunkDirtyState>,
    particle_query: Query<(&GridPosition, Option<&ParticleColor>), With<Particle>>,
) {
    for signal in msgr.read() {
        let entity = signal.entity;
        let Ok((grid_pos, particle_color)) = particle_query.get(entity) else {
            continue;
        };
        let position = grid_pos.0;

        if map.get_copied(position) == Ok(Some(entity)) {
            let _ = map.remove(position);
        }

        let chunk_coord = chunk_index.world_to_chunk_coord(position);
        if let Some(chunk_entity) = chunk_index.get(chunk_coord) {
            if let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity) {
                dirty_state.mark_dirty(position);
            }
        }

        let color = particle_color.map_or(Color::WHITE, |pc| pc.0);
        let collision_layers = if signal.collide_with_other_dynamic {
            CollisionLayers::new(DynamicParticleLayer::DynamicParticle, LayerMask::ALL)
        } else {
            CollisionLayers::new(
                DynamicParticleLayer::DynamicParticle,
                LayerMask::ALL ^ LayerMask::from(DynamicParticleLayer::DynamicParticle),
            )
        };
        let rb_entity = commands
            .spawn((
                Transform::from_xyz(position.x as f32, position.y as f32, 0.0),
                RigidBody::Dynamic,
                Collider::rectangle(1.0, 1.0),
                collision_layers,
                LinearVelocity(signal.linear_velocity),
                AngularVelocity(signal.angular_velocity),
                GravityScale(signal.gravity_scale),
                DynamicRigidBodyProxy {
                    particle_entity: entity,
                    last_map_position: None,
                },
                DynamicRigidBodyLifetime {
                    timer: Timer::new(signal.minimum_lifetime, TimerMode::Once),
                    rejoin_speed_threshold: signal.rejoin_speed_threshold,
                },
                Sprite {
                    color,
                    custom_size: Some(Vec2::ONE),
                    ..default()
                },
            ))
            .id();

        commands
            .entity(entity)
            .remove::<(
                GridPosition,
                ParticleColor,
                Movement,
                Speed,
                Density,
                AirResistance,
                Momentum,
            )>()
            .insert(SuspendedParticle {
                rigid_body_entity: rb_entity,
            });
    }
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub(super) fn rejoin_dynamic_rigid_bodies(
    mut commands: Commands,
    mut map: ResMut<ParticleMap>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<&mut ChunkDirtyState>,
    mut sync_writer: MessageWriter<SyncParticleSignal>,
    time: Res<Time>,
    mut proxy_query: Query<
        (
            Entity,
            &Transform,
            &LinearVelocity,
            &mut DynamicRigidBodyProxy,
            &mut DynamicRigidBodyLifetime,
            Has<Sleeping>,
        ),
        Without<Particle>,
    >,
    particle_query: Query<(), With<Particle>>,
    static_query: Query<(), With<StaticRigidBodyParticle>>,
    suspended_query: Query<(), With<SuspendedParticle>>,
) {
    let mut claimed: HashSet<IVec2> = HashSet::default();

    for (rb_entity, transform, linear_velocity, mut proxy, mut lifetime, is_sleeping) in
        &mut proxy_query
    {
        lifetime.timer.tick(time.delta());
        let expired = lifetime.timer.is_finished()
            && linear_velocity.0.length() < lifetime.rejoin_speed_threshold;
        let particle_entity = proxy.particle_entity;

        if particle_query.get(particle_entity).is_err() {
            commands.entity(rb_entity).try_despawn();
            continue;
        }

        let pos = IVec2::new(
            transform.translation.x.floor() as i32,
            transform.translation.y.floor() as i32,
        );

        let has_neighbor_or_edge = std::iter::once(pos)
            .chain(NEIGHBOR_OFFSETS.iter().map(|o| pos + *o))
            .any(|p| match map.get(p) {
                Ok(Some(entity)) => {
                    *entity != particle_entity
                        && !static_query.contains(*entity)
                        && !suspended_query.contains(*entity)
                }
                Err(_) => true,
                Ok(None) => false,
            });

        if !has_neighbor_or_edge && !is_sleeping && !expired {
            continue;
        }

        if let Some(last) = proxy.last_map_position.take() {
            if map.get_copied(last) == Ok(Some(particle_entity)) {
                let _ = map.remove(last);
                let chunk_coord = chunk_index.world_to_chunk_coord(last);
                if let Some(chunk_entity) = chunk_index.get(chunk_coord) {
                    if let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity) {
                        dirty_state.mark_dirty(last);
                    }
                }
            }
        }

        let max_upward_search = 50;

        let mut candidates = std::iter::once(pos).chain(NEIGHBOR_OFFSETS.iter().map(|o| pos + *o));

        let is_vacant = |p: &IVec2| {
            if claimed.contains(p) {
                return false;
            }
            match map.get_copied(*p) {
                Ok(None) => true,
                Ok(Some(e)) => e == particle_entity,
                Err(_) => false,
            }
        };

        let target = candidates.find(|p| is_vacant(p));

        let target = target.or_else(|| {
            (1..=max_upward_search)
                .map(|dy| pos + IVec2::new(0, dy))
                .find(|p| is_vacant(p))
        });

        let Some(target) = target else {
            commands.entity(rb_entity).despawn();
            commands.entity(particle_entity).despawn();
            continue;
        };

        claimed.insert(target);

        let _ = map.insert(target, particle_entity);

        let chunk_coord = chunk_index.world_to_chunk_coord(target);
        if let Some(chunk_entity) = chunk_index.get(chunk_coord) {
            if let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity) {
                dirty_state.mark_dirty(target);
            }
        }

        commands.entity(rb_entity).despawn();
        commands
            .entity(particle_entity)
            .remove::<SuspendedParticle>()
            .insert(GridPosition(target));

        sync_writer.write(SyncParticleSignal::from_entity(particle_entity));
    }
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub(super) fn sync_dynamic_rigid_bodies_with_particles(
    mut map: ResMut<ParticleMap>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<&mut ChunkDirtyState>,
    mut proxy_query: Query<(&Transform, &mut DynamicRigidBodyProxy), Changed<Transform>>,
) {
    let mark_dirty = |position: IVec2,
                      chunk_index: &ChunkIndex,
                      chunk_query: &mut Query<&mut ChunkDirtyState>| {
        let chunk_coord = chunk_index.world_to_chunk_coord(position);
        if let Some(chunk_entity) = chunk_index.get(chunk_coord) {
            if let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity) {
                dirty_state.mark_dirty(position);
            }
        }
    };

    for (transform, mut proxy) in &mut proxy_query {
        let pos = IVec2::new(
            transform.translation.x.floor() as i32,
            transform.translation.y.floor() as i32,
        );

        if proxy.last_map_position == Some(pos) {
            continue;
        }

        if !map.is_position_loaded(pos) {
            continue;
        }

        if let Some(prev) = proxy.last_map_position.take() {
            if map.get_copied(prev) == Ok(Some(proxy.particle_entity)) {
                let _ = map.remove(prev);
            }
            mark_dirty(prev, &chunk_index, &mut chunk_query);
        }

        match map.get_copied(pos) {
            Ok(None) => {
                let _ = map.insert(pos, proxy.particle_entity);
                mark_dirty(pos, &chunk_index, &mut chunk_query);
                proxy.last_map_position = Some(pos);
            }
            _ => {
                proxy.last_map_position = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{AttachedToParticleType, ParticleType, SpawnParticleSignal};
    use crate::FallingSandMinimalPlugin;

    fn create_test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, FallingSandMinimalPlugin::default()));
        app.add_plugins(DynamicPlugin);
        app.add_systems(
            Update,
            (
                promote_dynamic_rigid_bodies,
                rejoin_dynamic_rigid_bodies.after(promote_dynamic_rigid_bodies),
            ),
        );
        app
    }

    fn create_small_test_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            FallingSandMinimalPlugin::default().with_chunk_size(8),
        ));
        app.add_plugins(DynamicPlugin);
        app.add_systems(
            Update,
            (
                promote_dynamic_rigid_bodies,
                rejoin_dynamic_rigid_bodies.after(promote_dynamic_rigid_bodies),
            ),
        );
        app
    }

    fn spawn_particle(app: &mut App, name: &'static str, position: IVec2) -> Entity {
        app.world_mut()
            .write_message(SpawnParticleSignal::new(Particle::new(name), position));
        app.update();

        app.world()
            .resource::<ParticleMap>()
            .get_copied(position)
            .unwrap()
            .expect("particle should exist after spawn")
    }

    fn send_promote(app: &mut App, entity: Entity) {
        app.world_mut()
            .write_message(PromoteDynamicRigidBodyParticle::new(entity));
    }

    fn send_promote_with(app: &mut App, signal: PromoteDynamicRigidBodyParticle) {
        app.world_mut().write_message(signal);
    }

    // ---- promote_dynamic_rigid_bodies ----

    #[test]
    fn promote_removes_particle_from_map() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let pos = IVec2::new(5, 5);
        let entity = spawn_particle(&mut app, "sand", pos);

        send_promote(&mut app, entity);
        app.update();

        let map = app.world().resource::<ParticleMap>();
        assert_eq!(
            map.get_copied(pos).ok().flatten(),
            None,
            "promoted particle should be removed from ParticleMap"
        );
    }

    #[test]
    fn promote_removes_grid_position() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let entity = spawn_particle(&mut app, "sand", IVec2::new(5, 5));

        send_promote(&mut app, entity);
        app.update();

        assert!(
            app.world().entity(entity).get::<GridPosition>().is_none(),
            "promoted particle should not have GridPosition"
        );
    }

    #[test]
    fn promote_spawns_separate_rigid_body() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let pos = IVec2::new(5, 5);
        let entity = spawn_particle(&mut app, "sand", pos);

        send_promote(&mut app, entity);
        app.update();

        assert!(
            app.world().entity(entity).get::<RigidBody>().is_none(),
            "particle entity should NOT have RigidBody"
        );

        let suspended = app
            .world()
            .entity(entity)
            .get::<SuspendedParticle>()
            .expect("particle should have SuspendedParticle");
        let rb_entity = suspended.rigid_body_entity;

        let rb_ref = app.world().entity(rb_entity);
        assert!(rb_ref.get::<RigidBody>().is_some());
        assert!(rb_ref.get::<Collider>().is_some());
        assert!(rb_ref.get::<Transform>().is_some());

        let proxy = rb_ref.get::<DynamicRigidBodyProxy>().unwrap();
        assert_eq!(proxy.particle_entity, entity);

        let transform = rb_ref.get::<Transform>().unwrap();
        assert_eq!(transform.translation.x, 5.0);
        assert_eq!(transform.translation.y, 5.0);
    }

    #[test]
    fn promote_applies_velocity_and_gravity() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let entity = spawn_particle(&mut app, "sand", IVec2::new(5, 5));

        send_promote_with(
            &mut app,
            PromoteDynamicRigidBodyParticle::new(entity)
                .with_linear_velocity(Vec2::new(10.0, 20.0))
                .with_angular_velocity(3.14)
                .with_gravity_scale(2.5),
        );
        app.update();

        let suspended = app
            .world()
            .entity(entity)
            .get::<SuspendedParticle>()
            .unwrap();
        let rb_ref = app.world().entity(suspended.rigid_body_entity);

        assert_eq!(
            rb_ref.get::<LinearVelocity>().unwrap().0,
            Vec2::new(10.0, 20.0)
        );
        assert_eq!(rb_ref.get::<AngularVelocity>().unwrap().0, 3.14);
        assert_eq!(rb_ref.get::<GravityScale>().unwrap().0, 2.5);
    }

    #[test]
    fn promote_preserves_particle_component() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let entity = spawn_particle(&mut app, "sand", IVec2::new(5, 5));

        send_promote(&mut app, entity);
        app.update();

        let particle = app.world().entity(entity).get::<Particle>().unwrap();
        assert_eq!(particle.name, "sand");
    }

    // ---- rejoin_dynamic_rigid_bodies ----

    #[test]
    fn rejoin_when_neighbor_exists() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let neighbor_pos = IVec2::new(5, 5);
        let _neighbor = spawn_particle(&mut app, "sand", neighbor_pos);

        let dynamic_pos = IVec2::new(6, 5);
        let dynamic_entity = spawn_particle(&mut app, "sand", dynamic_pos);

        send_promote(&mut app, dynamic_entity);
        app.update();

        let entity_ref = app.world().entity(dynamic_entity);
        assert!(
            entity_ref.get::<GridPosition>().is_some(),
            "should rejoin the simulation when adjacent to a neighbor"
        );
        assert!(
            entity_ref.get::<SuspendedParticle>().is_none(),
            "SuspendedParticle should be removed after rejoin"
        );
    }

    #[test]
    fn rejoin_despawns_rigid_body_proxy() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let neighbor_pos = IVec2::new(5, 5);
        let _neighbor = spawn_particle(&mut app, "sand", neighbor_pos);

        let dynamic_pos = IVec2::new(6, 5);
        let dynamic_entity = spawn_particle(&mut app, "sand", dynamic_pos);

        send_promote(&mut app, dynamic_entity);
        app.update();

        let proxy_count = app
            .world_mut()
            .query::<&DynamicRigidBodyProxy>()
            .iter(app.world())
            .count();
        assert_eq!(
            proxy_count, 0,
            "rigid body proxy should be despawned after rejoin"
        );
    }

    #[test]
    fn no_rejoin_without_neighbor() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let pos = IVec2::new(20, 20);
        let entity = spawn_particle(&mut app, "sand", pos);

        send_promote(&mut app, entity);
        app.update();
        app.update();

        assert!(
            app.world()
                .entity(entity)
                .get::<SuspendedParticle>()
                .is_some(),
            "should remain suspended when no neighbors exist"
        );
        assert!(
            app.world().entity(entity).get::<GridPosition>().is_none(),
            "should not have GridPosition when still suspended"
        );
    }

    #[test]
    fn rejoin_restores_particle_to_map() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let neighbor_pos = IVec2::new(10, 10);
        let _neighbor = spawn_particle(&mut app, "sand", neighbor_pos);

        let dynamic_pos = IVec2::new(11, 10);
        let dynamic_entity = spawn_particle(&mut app, "sand", dynamic_pos);

        send_promote(&mut app, dynamic_entity);
        app.update();

        let grid_pos = app
            .world()
            .entity(dynamic_entity)
            .get::<GridPosition>()
            .expect("should have rejoined")
            .0;

        let map = app.world().resource::<ParticleMap>();
        assert_eq!(
            map.get_copied(grid_pos).ok().flatten(),
            Some(dynamic_entity),
            "rejoined particle should be in the ParticleMap at its GridPosition"
        );
    }

    #[test]
    fn rejoin_prefers_own_position_when_vacant() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let neighbor_pos = IVec2::new(10, 10);
        let _neighbor = spawn_particle(&mut app, "sand", neighbor_pos);

        let dynamic_pos = IVec2::new(11, 10);
        let dynamic_entity = spawn_particle(&mut app, "sand", dynamic_pos);

        send_promote(&mut app, dynamic_entity);
        app.update();

        let grid_pos = app
            .world()
            .entity(dynamic_entity)
            .get::<GridPosition>()
            .unwrap()
            .0;

        assert_eq!(
            grid_pos, dynamic_pos,
            "should rejoin at its own position when that position is vacant"
        );
    }

    #[test]
    fn rejoin_finds_alternate_position_when_own_is_occupied() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let dynamic_pos = IVec2::new(10, 10);
        let dynamic_entity = spawn_particle(&mut app, "sand", dynamic_pos);

        send_promote(&mut app, dynamic_entity);
        app.update();

        let blocker = spawn_particle(&mut app, "sand", dynamic_pos);
        app.update();

        let entity_ref = app.world().entity(dynamic_entity);
        let grid_pos = entity_ref
            .get::<GridPosition>()
            .expect("should rejoin (blocker is a neighbor)")
            .0;

        assert_ne!(grid_pos, dynamic_pos);

        let dist = (grid_pos - dynamic_pos).abs();
        assert!(dist.x <= 1 && dist.y <= 1);

        let map = app.world().resource::<ParticleMap>();
        assert_eq!(
            map.get_copied(grid_pos).ok().flatten(),
            Some(dynamic_entity)
        );
        assert_eq!(map.get_copied(dynamic_pos).ok().flatten(), Some(blocker));
    }

    #[test]
    fn rejoin_at_map_edge() {
        let mut app = create_small_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let origin = app.world().resource::<ParticleMap>().origin();
        let edge_pos = origin;
        let entity = spawn_particle(&mut app, "sand", edge_pos);

        send_promote(&mut app, entity);
        app.update();

        let entity_ref = app.world().entity(entity);
        assert!(
            entity_ref.get::<GridPosition>().is_some(),
            "should rejoin at map edge because neighbor lookup returns PositionUnloaded"
        );
        assert!(
            entity_ref.get::<SuspendedParticle>().is_none(),
            "SuspendedParticle should be removed after rejoin at edge"
        );
    }

    // ---- no particle loss ----

    #[test]
    fn no_particle_loss_single() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let pos = IVec2::new(5, 5);
        let entity = spawn_particle(&mut app, "sand", pos);

        send_promote(&mut app, entity);
        app.update();

        assert!(app.world().entities().contains(entity));
        assert!(app.world().entity(entity).get::<Particle>().is_some());
        assert!(app
            .world()
            .entity(entity)
            .get::<AttachedToParticleType>()
            .is_some());
    }

    #[test]
    fn no_particle_loss_through_full_cycle() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let anchor = IVec2::new(10, 10);
        let _anchor_entity = spawn_particle(&mut app, "sand", anchor);

        let dynamic_pos = IVec2::new(11, 10);
        let dynamic_entity = spawn_particle(&mut app, "sand", dynamic_pos);

        let initial_count = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .count();

        send_promote(&mut app, dynamic_entity);
        app.update();

        let count_while_dynamic = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .count();
        assert_eq!(count_while_dynamic, initial_count);

        let count_after_rejoin = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .count();
        assert_eq!(count_after_rejoin, initial_count);

        assert!(app.world().entities().contains(dynamic_entity));
        assert_eq!(
            app.world()
                .entity(dynamic_entity)
                .get::<Particle>()
                .unwrap()
                .name,
            "sand"
        );
    }

    #[test]
    fn no_particle_loss_multiple_dynamic_bodies() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let anchor = IVec2::new(10, 10);
        let _anchor = spawn_particle(&mut app, "sand", anchor);

        let positions = [IVec2::new(11, 10), IVec2::new(9, 10), IVec2::new(10, 11)];
        let mut dynamic_entities = Vec::new();
        for &pos in &positions {
            dynamic_entities.push(spawn_particle(&mut app, "sand", pos));
        }

        let total_before = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .count();

        for &entity in &dynamic_entities {
            send_promote(&mut app, entity);
        }
        app.update();

        let total_after = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .count();
        assert_eq!(total_after, total_before);

        for &entity in &dynamic_entities {
            assert!(app.world().entities().contains(entity));
        }
    }

    // ---- collision avoidance ----

    #[test]
    fn two_dynamic_bodies_do_not_rejoin_at_same_position() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let anchor = IVec2::new(10, 10);
        let _anchor = spawn_particle(&mut app, "sand", anchor);

        let pos_a = IVec2::new(11, 10);
        let pos_b = IVec2::new(11, 11);
        let entity_a = spawn_particle(&mut app, "sand", pos_a);
        let entity_b = spawn_particle(&mut app, "sand", pos_b);

        send_promote(&mut app, entity_a);
        send_promote(&mut app, entity_b);
        app.update();

        let rejoined: Vec<(Entity, IVec2)> = app
            .world_mut()
            .query_filtered::<(Entity, &GridPosition), With<Particle>>()
            .iter(app.world())
            .map(|(e, gp)| (e, gp.0))
            .collect();

        let positions: Vec<IVec2> = rejoined.iter().map(|(_, p)| *p).collect();
        let unique: HashSet<IVec2> = positions.iter().copied().collect();
        assert_eq!(positions.len(), unique.len());
    }

    // ---- original state restoration ----

    #[test]
    fn rejoin_restores_original_state() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let anchor = IVec2::new(10, 10);
        let _anchor = spawn_particle(&mut app, "sand", anchor);

        let pos = IVec2::new(11, 10);
        let entity = spawn_particle(&mut app, "sand", pos);

        let particle_before = app.world().entity(entity).get::<Particle>().cloned();
        let attached_before = app
            .world()
            .entity(entity)
            .get::<AttachedToParticleType>()
            .map(|a| a.0);

        send_promote(&mut app, entity);
        app.update();

        let entity_ref = app.world().entity(entity);
        assert_eq!(entity_ref.get::<Particle>().cloned(), particle_before);
        assert_eq!(
            entity_ref.get::<AttachedToParticleType>().map(|a| a.0),
            attached_before,
        );
        assert!(entity_ref.get::<GridPosition>().is_some());
        assert!(entity_ref.get::<SuspendedParticle>().is_none());
    }

    #[test]
    fn all_dynamic_bodies_return_to_original_state_near_neighbors() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let anchor = IVec2::new(15, 15);
        let _anchor = spawn_particle(&mut app, "sand", anchor);

        let dynamic_positions = [
            IVec2::new(16, 15),
            IVec2::new(14, 15),
            IVec2::new(15, 16),
            IVec2::new(15, 14),
        ];

        let mut entities = Vec::new();
        for &pos in &dynamic_positions {
            entities.push(spawn_particle(&mut app, "sand", pos));
        }

        for &entity in &entities {
            send_promote(&mut app, entity);
        }
        app.update();

        for &entity in &entities {
            let entity_ref = app.world().entity(entity);
            assert!(entity_ref.get::<GridPosition>().is_some());
            assert!(entity_ref.get::<SuspendedParticle>().is_none());
            assert!(entity_ref.get::<Particle>().is_some());
            assert!(entity_ref.get::<AttachedToParticleType>().is_some());
        }

        let map = app.world().resource::<ParticleMap>();
        let mut map_positions: HashSet<IVec2> = HashSet::default();
        for &entity in &entities {
            let gp = app.world().entity(entity).get::<GridPosition>().unwrap().0;
            assert_eq!(map.get_copied(gp).ok().flatten(), Some(entity));
            assert!(map_positions.insert(gp));
        }
    }
}
