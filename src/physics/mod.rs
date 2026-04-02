//! Integrates [avian2d](https://docs.rs/avian2d) physics with the falling sand simulation
//!
//! This module generates static rigid body collision meshes from particles marked with
//! [`StaticRigidBodyParticle`]. The mesh generation pipeline runs each frame and works
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

mod components;
mod geometry;
mod resources;
mod systems;

use bevy::prelude::*;

pub use components::StaticRigidBodyParticle;
pub use resources::{DirtyChunkUpdateInterval, DouglasPeuckerEpsilon};

use components::ComponentsPlugin;
use resources::ResourcesPlugin;
use systems::calculate_static_rigid_bodies;

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
        app.add_plugins(avian2d::PhysicsPlugins::default().with_length_unit(self.length_unit))
            .insert_resource(avian2d::prelude::Gravity(self.rigid_body_gravity))
            .add_plugins((ComponentsPlugin, ResourcesPlugin))
            .add_systems(Update, calculate_static_rigid_bodies);
    }
}
