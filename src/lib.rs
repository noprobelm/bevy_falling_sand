#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
//! This crate provides a [Falling Sand] plugin for [Bevy]. This plugin provides support for:
//! - Creating and rendering custom particles
//! - Simulating particle movement
//! - Mapping walls and solids to static rigid body colliders using avian2d
//! - Spatial querying particle entities
//! - Particle reactions
//! - Debug rendering
//!
//! ## Minimal Example
//!
//! Add the full plugin and specify length units (for [avian2d]) and the kdtree refresh frequency
//! (for [bevy_spatial])
//! ```
//! use bevy::prelude::*;
//! use bevy_falling_sand::prelude::FallingSandPlugin
//!
//! fn main() {
//!    App::new().add_plugins((DefaultPlugins,
//!    FallingSandPlugin::default()
//!        .with_length_unit(8.0)
//!        .with_spatial_refresh_frequency(std::time::Duration::from_millis(50)),
//!        ))
//!        .run();
//! }
//! ```
//!
//! This won't do much on its own, but it does set up all of the systems necessary for you to start
//! adding your own particles. See the examples for more information.
//!
//! ## This crate
//!
//! Like [Bevy], this crate is just a container that makes it easier to consume subcrates. The
//! default (as seen in the example above) enables all of the main features *Bevy Falling Sand*
//! provides, but [`FallingSandMinimalPlugin`] is also available to enable only core features.
//!
//! [Falling Sand]: https://en.wikipedia.org/wiki/Falling-sand_game
//! [avian2d]: https://docs.rs/avian2d/latest/avian2d/
//! [bevy_spatial]: https://docs.rs/bevy_spatial/latest/bevy_spatial/
//! [Bevy]: https://docs.rs/bevy/latest/bevy/
#![warn(
    clippy::nursery,
    clippy::pedantic,
    nonstandard_style,
    rustdoc::broken_intra_doc_links
)]
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]

use bevy::prelude::*;
pub use bfs_internal::*;
