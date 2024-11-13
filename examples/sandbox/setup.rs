//! Shows how to set up custom particle types for your world.
//!
//! `bevy_falling_sand` does not provide a default set of particles. See `examples/assets/particles/particles.ron` for
//! an example of how to create new particle types using RON.
//!
//! Alternatively (and for full access to particle behavior), spawn these bundles into the world to create a new
//! particle type:
//!   - `StaticParticleTypeBundle`: For particles that have no movement behavior (i.e., walls)
//!   - `DynamicParticleTypeBundle`: For particles that have movement behavior
use bevy::prelude::*;

use bevy_falling_sand::asset_loaders::*;
use bevy_falling_sand::bundles::*;
use bevy_falling_sand::color::*;
use bevy_falling_sand::core::*;
use bevy_falling_sand::movement::*;

/// Particle Management Plugin
pub(super) struct ParticleSetupPlugin;

impl bevy::prelude::Plugin for ParticleSetupPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Particle management systems
        app.add_event::<ParticleTypesAssetLoaded>()
            .add_systems(Startup, (setup_custom_particles, load_assets));
        app.add_systems(Update, load_particle_types);
    }
}

/// Demonstrates how to set up a custom particle with code.
pub fn setup_custom_particles(mut commands: Commands) {
    commands.spawn((
        MovableSolidBundle::new(
            ParticleType::new("My Custom Particle"),
            Density(1250),
            Velocity::new(1, 3),
            ParticleColor::new(
                Color::srgba(0.22, 0.11, 0.16, 1.0),
                vec![
                    Color::srgba(0.22, 0.11, 0.16, 1.0),
                    Color::srgba(0.24, 0.41, 0.56, 1.0),
                    Color::srgba(0.67, 0.74, 0.55, 1.0),
                    Color::srgba(0.91, 0.89, 0.71, 1.0),
                    Color::srgba(0.95, 0.61, 0.43, 1.0),
                ],
            ),
        ),
        // If momentum effects are desired, insert the marker component.
        Momentum::ZERO,
        Name::new("My Custom Particle"),
    ));

    commands.spawn((
        StaticParticleTypeBundle::new(
            ParticleType::new("My Custom Wall Particle"),
            ParticleColor::new(
                Color::srgba(0.22, 0.11, 0.16, 1.0),
                vec![
                    Color::srgba(0.22, 0.11, 0.16, 1.0),
                    Color::srgba(0.24, 0.41, 0.56, 1.0),
                    Color::srgba(0.67, 0.74, 0.55, 1.0),
                    Color::srgba(0.91, 0.89, 0.71, 1.0),
                    Color::srgba(0.95, 0.61, 0.43, 1.0),
                ],
            ),
        ),
        Name::new("My Custom Wall Particle"),
    ));
}

#[derive(Event)]
struct ParticleTypesAssetLoaded {
    handle: Handle<ParticleTypesAsset>,
}

fn load_assets(
    mut ev_asset: EventWriter<ParticleTypesAssetLoaded>,
    asset_server: Res<AssetServer>,
) {
    let handle: Handle<ParticleTypesAsset> = asset_server.load("particles/particles.ron");
    ev_asset.send(ParticleTypesAssetLoaded { handle });
}

fn load_particle_types(
    mut commands: Commands,
    mut type_map: ResMut<ParticleTypeMap>,
    mut ev_asset: EventReader<ParticleTypesAssetLoaded>,
    particle_types_asset: Res<Assets<ParticleTypesAsset>>,
) {
    for ev in ev_asset.read() {
        let asset = particle_types_asset.get(&ev.handle).unwrap();
        asset.load_particle_types(&mut commands, &mut type_map);
    }
}
