#![forbid(missing_docs)]
#![warn(
    clippy::nursery,
    clippy::pedantic,
    nonstandard_style,
    rustdoc::broken_intra_doc_links
)]
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]

//! This crate sources bevy_falling_sand crates.
pub use bfs_core as core;
pub use bfs_movement as movement;
pub use bfs_color as color;
pub use bfs_debug as debug;
