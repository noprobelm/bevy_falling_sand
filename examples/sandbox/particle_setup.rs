//! Shows how to set up custom particle types for your world.
//!
//! `bevy_falling_sand` does not provide a default set of particles. See `examples/assets/particles/particles.ron` for
//! an example of how to create new particle types using RON.
//!
//! Alternatively (and for full access to particle behavior), spawn these bundles into the world to create a new
//! particle type:
//!   - `StaticParticleTypeBundle`: For particles that have no movement behavior (i.e., walls)
//!   - `DynamicParticleTypeBundle`: For particles that have movement behavior
use bevy::{prelude::*, utils::Duration};
use std::path::Path;

use bevy_falling_sand::{material::Material, *};

/// Particle Management Plugin
pub(super) struct ParticleSetupPlugin;

impl bevy::prelude::Plugin for ParticleSetupPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Particle management systems
        app.add_systems(Startup, (setup_particle_types, setup_custom_particle));
    }
}

/// The easiest way to add new particles: publish a ParticleDeserializeEvent.
pub fn setup_particle_types(
    mut ev_particle_deserialize: EventWriter<DeserializeParticleTypesEvent>,
) {
    let mut example_path = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    example_path.push("examples/assets/particles/particles.ron");
    ev_particle_deserialize.send(DeserializeParticleTypesEvent(example_path));
}

/// Demonstrates how to set up a custom particle with code.
pub fn setup_custom_particle(mut commands: Commands) {
    commands.spawn((
        DynamicParticleTypeBundle::new(
            ParticleType::new("My Custom Particle"),
            Density(4),
            Velocity::new(1, 3),
            MovableSolid::new().into_movement_priority(),
            ParticleColors::new(vec![
                Color::srgba(0.22, 0.11, 0.16, 1.0),
                Color::srgba(0.24, 0.41, 0.56, 1.0),
                Color::srgba(0.67, 0.74, 0.55, 1.0),
                Color::srgba(0.91, 0.89, 0.71, 1.0),
                Color::srgba(0.95, 0.61, 0.43, 1.0),
            ]),
        ),
        // If momentum effects are desired, insert the marker component.
        Momentum::ZERO,
        // This particle type can burn when it comes within range of an entity with the Fire component.
        Burns::new(
            Duration::from_secs(10),
            Duration::from_secs(1),
            true,
            0.0,
            Some(Reacting::new(Particle::new("Steam"), 0.5)),
            None,
            None,
            None,
        ),
    ));

    commands.spawn(StaticParticleTypeBundle::new(
        ParticleType::new("My Custom Wall Particle"),
        ParticleColors::new(vec![
            Color::srgba(0.22, 0.11, 0.16, 1.0),
            Color::srgba(0.24, 0.41, 0.56, 1.0),
            Color::srgba(0.67, 0.74, 0.55, 1.0),
            Color::srgba(0.91, 0.89, 0.71, 1.0),
            Color::srgba(0.95, 0.61, 0.43, 1.0),
        ]),
    ));

    commands.spawn((
        DynamicParticleTypeBundle::new(
            ParticleType::new("Water"),
            Density(2),
            Velocity::new(1, 3),
            Liquid::new(5).into_movement_priority(),
            ParticleColors::new(vec![Color::srgba(0.043, 0.5, 0.67, 0.5)]),
        ),
        Momentum::ZERO,
        // This particle type can burn when it comes within range of an entity with the Fire component.
        Burns::new(
            Duration::from_millis(0),
            Duration::from_millis(0),
            false,
            0.0,
            None,
            Some(Particle::new("Steam")),
            None,
            None,
        ),
    ));

    commands.spawn((
        DynamicParticleTypeBundle::new(
            ParticleType::new("FIRE"),
            Density(4),
            Velocity::new(1, 3),
            Gas::new(1).into_movement_priority(),
            ParticleColors::new(vec![
                Color::srgba(1.0, 0.0, 0.0, 1.0),
                Color::srgba(1.0, 0.35, 0.0, 1.0),
                Color::srgba(1.0, 0.6, 0.0, 1.0),
                Color::srgba(1.0, 0.81, 0.0, 1.0),
                Color::srgba(1.0, 0.91, 0.03, 1.0),
            ]),
        ),
        Fire {
            burn_radius: 1.5,
            chance_to_spread: 0.01,
            destroys_on_ignition: false,
        },
        Burns::new(
            Duration::from_secs(1),
            Duration::from_millis(100),
            true,
            0.5,
            None,
            None,
            None,
            None,
        ),
        Burning,
    ));

    commands.spawn((
        StaticParticleTypeBundle::new(
            ParticleType::new("Wood Wall"),
            ParticleColors::new(vec![Color::srgba(0.63, 0.4, 0.18, 1.0)]),
        ),
        Burns::new(
            Duration::from_secs(10),
            Duration::from_millis(100),
            true,
            0.,
            Some(Reacting {
                produces: Particle::new("Steam"),
                chance_to_produce: 0.015,
            }),
            None,
            Some(RandomColors::new(vec![
                Color::srgba(1.0, 0.0, 0.0, 1.0),
                Color::srgba(1.0, 0.35, 0.0, 1.0),
                Color::srgba(1.0, 0.6, 0.0, 1.0),
                Color::srgba(1.0, 0.81, 0.0, 1.0),
                Color::srgba(1.0, 0.91, 0.03, 1.0),
            ])),
            Some(Fire {
                burn_radius: 1.5,
                chance_to_spread: 0.0025,
                destroys_on_ignition: false,
            }),
        ),
    ));
}
