//! `bevy_falling_sand` is a generic plugin for adding falling sand simulation physics to your Bevy project.

use bevy::prelude::*;

pub use components::*;
pub use resources::*;

mod components;
mod resources;

pub struct FallingSandPlugin;

impl Plugin for FallingSandPlugin {
    fn build(&self, app: &mut App) {
    }
}
