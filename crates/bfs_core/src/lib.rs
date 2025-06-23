mod map;
mod particle;
mod rng;

use bevy::prelude::*;

pub use map::*;
pub use particle::*;
pub use rng::*;

pub struct FallingSandCorePlugin;

impl Plugin for FallingSandCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ParticleCorePlugin, ParticleMapPlugin));
    }
}
