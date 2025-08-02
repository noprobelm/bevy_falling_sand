mod utils;

use bevy::prelude::*;
use bevy_falling_sand::prelude::{
    FallingSandAssetsPlugin, FallingSandMinimalPlugin, ParticleDefinitionsAsset,
    ParticleDefinitionsHandle,
};
use utils::status_ui::{FpsText, StatusUIPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FallingSandMinimalPlugin::default(),
            FallingSandAssetsPlugin,
            StatusUIPlugin,
            utils::instructions::InstructionsPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, check_asset_loading)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn(Camera2d);

    // Simple instructions and status panel
    let instructions_text = "This example demonstrates loading particle definitions from assets.\n\
        Check the console for loaded particle types.";
    let panel_id = utils::instructions::spawn_instructions_panel(&mut commands, instructions_text);

    commands.entity(panel_id).with_children(|parent| {
        let style = TextFont::default();
        parent.spawn((FpsText, Text::new("FPS: --"), style.clone()));
    });

    // Load the particle definitions asset
    let particles_handle: Handle<ParticleDefinitionsAsset> =
        asset_server.load("particles/particles.ron");

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
