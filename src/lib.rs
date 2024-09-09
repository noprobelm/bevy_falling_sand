//! `bevy_falling_sand` is a plugin for adding falling sand physics to your Bevy project.
//!
//! ## Bevy versions
//!
//! | `bevy_falling_sand`   | `bevy`    |
//! |-----------------------|-----------|
//! | 0.3.x                 | 0.14.x    |
//! | 0.1.x                 | 0.13.x    |

//! ## Example
//! If you clone this repository, there is an example available that provides a full GUI interface for a "sandbox" like
//! experience. I recommend running the example with the `--release` flag to maximize performance:
//! ```rust
//! cargo run --example sandbox --release
//! ```
//!
//! ## How to use
//!
//! Spawning a particle is easy, just insert a `ParticleType` variant on an entity with a `Transform`
//! component and it will be added to the simulation:
//! ```rust
//! commands.spawn((
//!     ParticleType::Water,
//!     SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
//!     ));
//! ```
//!
//! ## Current limitations
//! ### ChunkMap size
//! For optimization reasons, the underlying mapping mechanism for particles utilizes a sequence of chunks, each of which
//! will induce a "hibernating" state on itself and the particles it contains if no movement is detected in a given frame.
//! Because of this, the total map size is limited (the default is 1024x1024 spanning from transform `(-512, 512)`
//! through `(512,  -512)`). Any particle processed outside of this region will cause a panic.
//!
//! This will be resolved in a future release, which will modify the ChunkMap to "follow" and entity with an arbitrary
//! specified component (for example, a main camera), loading and unloading chunks as it moves. This will emulate an
//! "infinite" space in which particles can reside.
//!
//! ### Single-threaded simulation
//! Currently, the particle simulation is single threaded. An multi-threaded simulation is planned for a future release.
//!
//! If you want to tweak CPU thread allocation in the meantime to experiment with performance, you might try adjusting
//! the default task pool thread assignment policy that `bevy` provides. I've found differing results in performance
//! based on CPU  manufacturer/architecture (sorry, no benchmarks available)
//! ```rust
//! use bevy::{
//!     core::TaskPoolThreadAssignmentPolicy, prelude::*, tasks::available_parallelism,
//! };
//!
//! fn main() {
//!     let mut app = App::new();
//!     app.add_plugins(DefaultPlugins
//!         .set(TaskPoolPlugin {
//!             task_pool_options: TaskPoolOptions {
//!                 compute: TaskPoolThreadAssignmentPolicy {
//!                     min_threads: available_parallelism(),
//!                     max_threads: std::usize::MAX,
//!                     percent: 1.0,
//!                 },
//!                 ..default()
//!             },
//!         }));
//! }
//! ```
//!
//! ## Visualizing chunk behavior
//!
//! If you want to visualize how chunks behave, insert the `DebugParticles` resource:
//! ```rust
//! app.init_resource::<DebugParticles>()
//! ```
#![forbid(missing_docs)]
#![warn(
    clippy::nursery,
    clippy::pedantic,
    nonstandard_style,
    rustdoc::broken_intra_doc_links
)]
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]

use bevy::prelude::*;
use bevy_turborand::prelude::*;

mod components;
mod events;
mod gizmos;
mod resources;
mod systems;
mod asset_loaders;
mod type_registry;

pub use components::*;
pub use systems::*;
pub use resources::*;
pub use events::*;
pub use gizmos::*;
pub use asset_loaders::*;
use type_registry::ParticleTypeRegistryPlugin;

/// Bevy Falling Sand Plugin
pub struct FallingSandPlugin;

impl Plugin for FallingSandPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RngPlugin::default(),
	    ParticleSystemsPlugin,
            ParticleResourcesPlugin,
            ParticleEventsPlugin,
            ParticleGizmosPlugin,
	    ParticleTypeRegistryPlugin,
	    AssetLoaderPlugin
        ));
    }
}
