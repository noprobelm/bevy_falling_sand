//! Integrates [avian2d](https://docs.rs/avian2d) physics with the falling sand simulation.
//!
//! This module provides two sub-modules, each owning a distinct domain:
//!
//! - **[`dynamic`]** — sending a [`PromoteDynamicRigidBodyParticle`] for a particle removes it from the
//!   simulation and spawns a separate physics-driven rigid body entity at its position. Each
//!   frame, if the rigid body is within one cell of any particle in the
//!   [`ParticleMap`](crate::ParticleMap) (or at the map edge), the rigid body is despawned and
//!   the original particle is restored to the simulation at the nearest vacant position.
//!
//! - **[`static`](mod@self::static)** — particles marked with [`StaticRigidBodyParticle`] (on the
//!   [`ParticleType`](crate::ParticleType)) contribute to per-chunk collision meshes.
//!
//! ## Static mesh generation
//!
//! The static mesh generation pipeline runs each frame and works
//! per-chunk:
//!
//! 1. **Identify dirty chunks** — chunks whose dirty state just cleared ("settled") are
//!    processed immediately. Chunks still actively dirty are throttled by
//!    [`DirtyChunkUpdateInterval`] to avoid excessive recalculation.
//!
//! 2. **Build occupancy bitmap** — for each chunk to process, scan every position and
//!    record which cells contain a `StaticRigidBodyParticle` entity. Compare against
//!    the cached bitmap from the previous pass; skip if unchanged.
//!
//! 3. **Spawn async mesh generation tasks** — for changed chunks, an async task performs:
//!    - **Flood-fill** to discover connected components of occupied cells.
//!    - **Perimeter extraction** — for each component, find the boundary edges between
//!      occupied and empty cells.
//!    - **Edge ordering** — assemble edges into closed loops.
//!    - **Douglas-Peucker simplification** — reduce vertex count using the epsilon from
//!      [`DouglasPeuckerEpsilon`]. TODO: This needs more in-depth profiling to see if
//!      we actually get gains from this
//!    - **Ear-cut triangulation** — convert the simplified polygon into a triangle mesh.
//!
//! 4. **Poll completed tasks** — merge the per-component meshes into a single trimesh
//!    [`Collider`](avian2d::prelude::Collider) and attach it to a per-chunk static
//!    [`RigidBody`](avian2d::prelude::RigidBody) entity (creating or updating as needed).
//!
//! 5. **Wake sleeping bodies** — any dynamic rigid bodies overlapping recalculated chunks
//!    have their [`Sleeping`](avian2d::prelude::Sleeping) component removed so they
//!    respond to the new collision geometry.

pub mod dynamic;
mod geometry;
pub mod r#static;

use avian2d::prelude::PhysicsInterpolationPlugin;
use bevy::prelude::*;

pub use dynamic::{
    DynamicRigidBodyProxy, PromoteDynamicRigidBodyParticle,
    StaticRigidBodyParticle, SuspendedParticle,
};
pub use r#static::{DirtyChunkUpdateInterval, DouglasPeuckerEpsilon};

use dynamic::DynamicPlugin;
use dynamic::{promote_dynamic_rigid_bodies, rejoin_dynamic_rigid_bodies};
use r#static::calculate_static_rigid_bodies;
use r#static::StaticPlugin;

use crate::movement::ParticleMovementSet;
use crate::physics::dynamic::sync_dynamic_rigid_bodies_with_particles;

/// Plugin providing avian2d rigid body integration for the falling sand simulation.
///
/// Registers all physics components, resources, and systems. Adds the
/// [`avian2d::PhysicsPlugins`] with the configured length unit and gravity.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::physics::FallingSandPhysicsPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(
///             FallingSandPhysicsPlugin::default()
///                 .with_length_unit(8.0),
///         )
///         .run();
/// }
/// ```
pub struct FallingSandPhysicsPlugin {
    /// The value for avian2d's `PhysicsLengthUnit`.
    pub length_unit: f32,
    /// The gravity vector for rigid bodies.
    pub rigid_body_gravity: Vec2,
}

impl Default for FallingSandPhysicsPlugin {
    fn default() -> Self {
        Self {
            length_unit: 1.0,
            rigid_body_gravity: Vec2::new(0.0, -9.81),
        }
    }
}

impl FallingSandPhysicsPlugin {
    /// Set the physics length unit for avian2d.
    #[must_use]
    pub const fn with_length_unit(mut self, unit: f32) -> Self {
        self.length_unit = unit;
        self
    }
}

impl Plugin for FallingSandPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            avian2d::PhysicsPlugins::default()
                .with_length_unit(self.length_unit)
                .set(PhysicsInterpolationPlugin::interpolate_all()),
        )
        .insert_resource(avian2d::prelude::Gravity(self.rigid_body_gravity))
        .add_plugins((DynamicPlugin, StaticPlugin))
        .add_systems(
            Update,
            (
                promote_dynamic_rigid_bodies,
                rejoin_dynamic_rigid_bodies.after(promote_dynamic_rigid_bodies),
                calculate_static_rigid_bodies.after(promote_dynamic_rigid_bodies),
            ),
        )
        .add_systems(
            PostUpdate,
            sync_dynamic_rigid_bodies_with_particles.after(ParticleMovementSet),
        );
    }
}
