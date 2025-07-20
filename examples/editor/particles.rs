use bevy::prelude::*;
use bfs_assets::{ParticleDefinitionsAsset, ParticleDefinitionsHandle};

pub(crate) struct ParticleSetupPlugin;

impl Plugin for ParticleSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let particles_handle: Handle<ParticleDefinitionsAsset> =
        asset_server.load("particles/particles.ron");
    commands.spawn(ParticleDefinitionsHandle::new(particles_handle));
}
