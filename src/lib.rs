//! `bevy_falling_sand` is a generic plugin for adding falling sand simulation physics to your Bevy project.

use bevy::prelude::*;

pub use components::*;
pub use resources::*;
pub use systems::*;

mod components;
mod resources;
mod systems;

pub struct FallingSandPlugin;

impl Plugin for FallingSandPlugin {
    fn build(&self, app: &mut App) {
    }
}
