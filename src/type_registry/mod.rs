/// Registers simulation types.

use crate::components::*;
use crate::resources::*;

pub(super) struct ParticleTypeRegistryPlugin;

impl bevy::prelude::Plugin for ParticleTypeRegistryPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<bevy::prelude::Name>()
            .register_type::<ParticleTypeMap>()
            .register_type::<ParticleType>()
            .register_type::<Density>()
            .register_type::<ParticleColor>()
            .register_type::<Velocity>()
            .register_type::<Momentum>()
            .register_type::<Particle>();
    }
}
