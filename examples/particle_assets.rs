use bevy::prelude::*;
use bfs_assets::{FallingSandAssetsPlugin, ParticleDefinitionsAsset, ParticleDefinitionsHandle};
use bfs_color::FallingSandColorPlugin;
use bfs_core::FallingSandCorePlugin;
use bfs_movement::FallingSandMovementPlugin;
use bfs_reactions::FallingSandReactionsPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FallingSandCorePlugin,
            FallingSandColorPlugin,
            FallingSandMovementPlugin,
            FallingSandReactionsPlugin,
            FallingSandAssetsPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, check_asset_loading)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load the particle definitions asset
    let particles_handle: Handle<ParticleDefinitionsAsset> = 
        asset_server.load("particles/modern_particles.ron");
    
    // Spawn an entity to track the asset loading
    commands.spawn(ParticleDefinitionsHandle::new(particles_handle));
    
    info!("Loading particle definitions from asset file...");
}

fn check_asset_loading(
    handles: Query<&ParticleDefinitionsHandle>,
    assets: Res<Assets<ParticleDefinitionsAsset>>,
) {
    for handle_component in handles.iter() {
        if handle_component.spawned {
            if let Some(asset) = assets.get(&handle_component.handle) {
                info!("Particle definitions loaded! Available particles:");
                for name in asset.definitions().keys() {
                    info!("  - {}", name);
                }
            }
        }
    }
}