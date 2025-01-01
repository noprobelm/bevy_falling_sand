//! Shows how to set up custom particle types for your world.
//!
//! `bevy_falling_sand` does not provide a default set of particles. See `examples/assets/particles/particles.ron` for
//! an example of how to create new particle types using RON.
//!
//! Alternatively (and for full access to particle behavior), spawn these bundles into the world to create a new
//! particle type:
//!   - `WallBundle`: For particles that have no movement behavior (i.e., walls)
//!   - `DynamicParticleTypeBundle`: For particles that have movement behavior
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_falling_sand::bundles::*;
use bevy_falling_sand::color::*;
use bevy_falling_sand::core::*;
use bevy_falling_sand::movement::*;
use bevy_falling_sand::reactions::*;

/// Particle Management Plugin
pub(super) struct ParticleSetupPlugin;

impl bevy::prelude::Plugin for ParticleSetupPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Particle management systems
        app.add_systems(Startup, setup_particles);
    }
}

/// Demonstrates how to set up a custom particle with code.
pub fn setup_particles(mut commands: Commands) {
    commands.spawn((
        SolidBundle::new(
            ParticleType::new("Rock"),
            Density(1250),
            Velocity::new(1, 3),
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#6B738C").unwrap()),
                vec![
                    Color::Srgba(Srgba::hex("#6B738C").unwrap()),
                    Color::Srgba(Srgba::hex("#8C96AB").unwrap()),
                    Color::Srgba(Srgba::hex("#B2C4D6").unwrap()),
                ],
            ),
        ),
        Name::new("Rock"),
    ));

    commands.spawn((
        WallBundle::new(
            ParticleType::new("Ice Wall"),
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#8CDBF880").unwrap()),
                vec![Color::Srgba(Srgba::hex("#8CDBF880").unwrap())],
            ),
        ),
        BurnsBlueprint(Burns::new(
            Duration::from_secs(2),
            Duration::from_millis(100),
            Some(0.01),
            Some(Reacting::new(Particle::new("Water"), 0.2)),
            None,
            None,
        )),
        Name::new("Ice Wall"),
    ));

    commands.spawn((
        MovableSolidBundle::new(
            ParticleType::new("My Custom Particle"),
            Density(1250),
            Velocity::new(1, 3),
            ColorProfile::new_with_selected(
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
        MomentumBlueprint(Momentum::ZERO),
        Name::new("My Custom Particle"),
    ));

    commands.spawn((
        WallBundle::new(
            ParticleType::new("My Custom Wall Particle"),
            ColorProfile::new_with_selected(
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

    commands.spawn((
        LiquidBundle::new(
            ParticleType::new("Water"),
            Density(750),
            Velocity::new(1, 3),
            5,
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#0B80AB80").unwrap()),
                vec![Color::Srgba(Srgba::hex("#0B80AB80").unwrap())],
            ),
        ),
        MomentumBlueprint(Momentum::ZERO),
        Name::new("Water"),
    ));

    commands.spawn((
        LiquidBundle::new(
            ParticleType::new("Sparkly Slime"),
            Density(850),
            Velocity::new(1, 2),
            1,
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#94B5C7FF").unwrap()),
                vec![
                    Color::Srgba(Srgba::hex("#94B5C7FF").unwrap()),
                    Color::Srgba(Srgba::hex("#DEEDABFF").unwrap()),
                    Color::Srgba(Srgba::hex("#F0CF66FF").unwrap()),
                    Color::Srgba(Srgba::hex("#D6826BFF").unwrap()),
                    Color::Srgba(Srgba::hex("#BD4F6BFF").unwrap()),
                    Color::Srgba(Srgba::hex("#F05C5EFF").unwrap()),
                ],
            ),
        ),
        MomentumBlueprint(Momentum::ZERO),
        FlowsColorBlueprint(FlowsColor::new(0.1)),
        Name::new("Sparkly Slime"),
    ));

    commands.spawn((
        LiquidBundle::new(
            ParticleType::new("Slime"),
            Density(850),
            Velocity::new(1, 2),
            1,
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#8FA73980").unwrap()),
                vec![
                    Color::Srgba(Srgba::hex("#82983480").unwrap()),
                    Color::Srgba(Srgba::hex("#8FA73980").unwrap()),
                ],
            ),
        ),
        MomentumBlueprint(Momentum::ZERO),
        FlowsColorBlueprint(FlowsColor::new(0.1)),
        Name::new("Slime"),
    ));

    commands.spawn((
        LiquidBundle::new(
            ParticleType::new("Whiskey"),
            Density(850),
            Velocity::new(1, 3),
            5,
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#D6997080").unwrap()),
                vec![Color::Srgba(Srgba::hex("#D6997080").unwrap())],
            ),
        ),
        MomentumBlueprint(Momentum::ZERO),
        Name::new("Whiskey"),
    ));

    commands.spawn((
        LiquidBundle::new(
            ParticleType::new("Blood"),
            Density(800),
            Velocity::new(1, 3),
            5,
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#780606").unwrap()),
                vec![Color::Srgba(Srgba::hex("#780606").unwrap())],
            ),
        ),
        MomentumBlueprint(Momentum::ZERO),
        Name::new("Blood"),
    ));

    commands.spawn((
        LiquidBundle::new(
            ParticleType::new("Oil"),
            Density(730),
            Velocity::new(1, 3),
            3,
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#2B1229").unwrap()),
                vec![Color::Srgba(Srgba::hex("#2B1229").unwrap())],
            ),
        ),
        MomentumBlueprint(Momentum::ZERO),
        BurnsBlueprint(Burns::new(
            Duration::from_secs(5),
            Duration::from_millis(100),
            Some(0.1),
            Some(Reacting::new(Particle::new("Smoke"), 0.035)),
            Some(ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#FF0000").unwrap()),
                vec![
                    Color::Srgba(Srgba::hex("#FF5900").unwrap()),
                    Color::Srgba(Srgba::hex("#FF0000").unwrap()),
                    Color::Srgba(Srgba::hex("#FF9900").unwrap()),
                    Color::Srgba(Srgba::hex("#FFCF00").unwrap()),
                    Color::Srgba(Srgba::hex("#FFE808").unwrap()),
                ],
            )),
            Some(Fire {
                burn_radius: 2.,
                chance_to_spread: 0.2,
                destroys_on_spread: false,
            }),
        )),
        Name::new("Oil"),
    ));

    commands.spawn((
        MovableSolidBundle::new(
            ParticleType::new("Sand"),
            Density(1250),
            Velocity::new(1, 3),
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#F2E06B").unwrap()),
                vec![
                    Color::Srgba(Srgba::hex("#FFEB8A").unwrap()),
                    Color::Srgba(Srgba::hex("#F2E06B").unwrap()),
                ],
            ),
        ),
        // If momentum effects are desired, insert the marker component.
        MomentumBlueprint(Momentum::ZERO),
        Name::new("Sand"),
    ));

    commands.spawn((
        MovableSolidBundle::new(
            ParticleType::new("Dirt"),
            Density(1250),
            Velocity::new(1, 3),
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#916B4C").unwrap()),
                vec![
                    Color::Srgba(Srgba::hex("#916B4C").unwrap()),
                    Color::Srgba(Srgba::hex("#73573D").unwrap()),
                ],
            ),
        ),
        // If momentum effects are desired, insert the marker component.
        MomentumBlueprint(Momentum::ZERO),
        Name::new("Dirt"),
    ));

    commands.spawn((
        MovableSolidBundle::new(
            ParticleType::new("Snow"),
            Density(1250),
            Velocity::new(1, 3),
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#EAFDF8").unwrap()),
                vec![
                    Color::Srgba(Srgba::hex("#EAFDF8").unwrap()),
                    Color::Srgba(Srgba::hex("#FFFFFF").unwrap()),
                ],
            ),
        ),
        // If momentum effects are desired, insert the marker component.
        MomentumBlueprint(Momentum::ZERO),
        Name::new("Snow"),
    ));

    commands.spawn((
        GasBundle::new(
            ParticleType::new("Steam"),
            Density(250),
            Velocity::new(1, 1),
            3,
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#C7D6E0").unwrap()),
                vec![
                    Color::Srgba(Srgba::hex("#EEF2F4").unwrap()),
                    Color::Srgba(Srgba::hex("#C7D6E0").unwrap()),
                ],
            ),
        ),
        FlowsColorBlueprint(FlowsColor::new(0.1)),
        BurnsBlueprint(Burns::new(
            Duration::from_millis(200),
            Duration::from_millis(100),
            Some(1.),
            Some(Reacting::new(Particle::new("Water"), 1.)),
            None,
            None,
        )),
        Name::new("Steam"),
    ));

    commands.spawn((
        GasBundle::new(
            ParticleType::new("Smoke"),
            Density(275),
            Velocity::new(1, 1),
            1,
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#706966").unwrap()),
                vec![
                    Color::Srgba(Srgba::hex("#706966").unwrap()),
                    Color::Srgba(Srgba::hex("#858073").unwrap()),
                ],
            ),
        ),
        FlowsColorBlueprint(FlowsColor::new(0.1)),
        Name::new("Smoke"),
    ));

    commands.spawn((
        GasBundle::new(
            ParticleType::new("Flammable Gas"),
            Density(200),
            Velocity::new(1, 1),
            1,
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#40621880").unwrap()),
                vec![
                    Color::Srgba(Srgba::hex("#40621880").unwrap()),
                    Color::Srgba(Srgba::hex("#4A731C80").unwrap()),
                ],
            ),
        ),
        FlowsColorBlueprint(FlowsColor::new(0.1)),
        BurnsBlueprint(Burns::new(
            Duration::from_secs(1),
            Duration::from_millis(100),
            Some(0.5),
            None,
            Some(ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#FF0000").unwrap()),
                vec![
                    Color::Srgba(Srgba::hex("#FF5900").unwrap()),
                    Color::Srgba(Srgba::hex("#FF0000").unwrap()),
                    Color::Srgba(Srgba::hex("#FF9900").unwrap()),
                    Color::Srgba(Srgba::hex("#FFCF00").unwrap()),
                    Color::Srgba(Srgba::hex("#FFE808").unwrap()),
                ],
            )),
            Some(Fire {
                burn_radius: 2.,
                chance_to_spread: 1.,
                destroys_on_spread: true,
            }),
        )),
        Name::new("Flammable Gas"),
    ));

    commands.spawn((
        GasBundle::new(
            ParticleType::new("FIRE"),
            Density(450),
            Velocity::new(1, 3),
            1,
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#FF5900FF").unwrap()),
                vec![
                    Color::Srgba(Srgba::hex("#FF5900FF").unwrap()),
                    Color::Srgba(Srgba::hex("#FF9100FF").unwrap()),
                    Color::Srgba(Srgba::hex("#FFCF00FF").unwrap()),
                    Color::Srgba(Srgba::hex("#C74A05FF").unwrap()),
                ],
            ),
        ),
        FlowsColorBlueprint(FlowsColor::new(0.1)),
        FireBlueprint(Fire {
            burn_radius: 1.5,
            chance_to_spread: 0.01,
            destroys_on_spread: false,
        }),
        BurnsBlueprint(Burns::new(
            Duration::from_secs(1),
            Duration::from_millis(100),
            Some(0.5),
            None,
            None,
            None,
        )),
        BurningBlueprint(Burning::new(
            Duration::from_secs(1),
            Duration::from_millis(100),
        )),
        Name::new("FIRE"),
    ));

    commands.spawn((
        WallBundle::new(
            ParticleType::new("Dirt Wall"),
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#916B4C").unwrap()),
                vec![
                    Color::Srgba(Srgba::hex("#916B4C").unwrap()),
                    Color::Srgba(Srgba::hex("#73573D").unwrap()),
                ],
            ),
        ),
        Name::new("Dirt Wall"),
    ));

    commands.spawn((
        WallBundle::new(
            ParticleType::new("Rock Wall"),
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#3B3333").unwrap()),
                vec![
                    Color::Srgba(Srgba::hex("#3B3333").unwrap()),
                    Color::Srgba(Srgba::hex("#4A3D3D").unwrap()),
                    Color::Srgba(Srgba::hex("#5C4A4A").unwrap()),
                    Color::Srgba(Srgba::hex("#665454").unwrap()),
                ],
            ),
        ),
        Name::new("Rock Wall"),
    ));

    commands.spawn((
        WallBundle::new(
            ParticleType::new("Dense Rock Wall"),
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#6B738C").unwrap()),
                vec![
                    Color::Srgba(Srgba::hex("#6B738C").unwrap()),
                    Color::Srgba(Srgba::hex("#8C96AB").unwrap()),
                    Color::Srgba(Srgba::hex("#B2C4D6").unwrap()),
                ],
            ),
        ),
        Name::new("Dense Rock Wall"),
    ));

    commands.spawn((
        WallBundle::new(
            ParticleType::new("Grass Wall"),
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#5C8730").unwrap()),
                vec![
                    Color::Srgba(Srgba::hex("#5C8730").unwrap()),
                    Color::Srgba(Srgba::hex("#3D5C21").unwrap()),
                    Color::Srgba(Srgba::hex("#527A2E").unwrap()),
                    Color::Srgba(Srgba::hex("#5C8C33").unwrap()),
                ],
            ),
        ),
        BurnsBlueprint(Burns::new(
            Duration::from_secs(1),
            Duration::from_millis(100),
            Some(0.5),
            Some(Reacting::new(Particle::new("FIRE"), 1.)),
            Some(ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#FF0000").unwrap()),
                vec![
                    Color::Srgba(Srgba::hex("#FF5900").unwrap()),
                    Color::Srgba(Srgba::hex("#FF0000").unwrap()),
                    Color::Srgba(Srgba::hex("#FF9900").unwrap()),
                    Color::Srgba(Srgba::hex("#FFCF00").unwrap()),
                    Color::Srgba(Srgba::hex("#FFE808").unwrap()),
                ],
            )),
            Some(Fire {
                burn_radius: 1.5,
                chance_to_spread: 1.,
                destroys_on_spread: false,
            }),
        )),
        Name::new("Grass Wall"),
    ));

    commands.spawn((
        WallBundle::new(
            ParticleType::new("Wood Wall"),
            ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#A1662E").unwrap()),
                vec![Color::Srgba(Srgba::hex("#A1662E").unwrap())],
            ),
        ),
        BurnsBlueprint(Burns::new(
            Duration::from_secs(10),
            Duration::from_millis(100),
            Some(0.0),
            Some(Reacting::new(Particle::new("Smoke"), 0.035)),
            Some(ColorProfile::new_with_selected(
                Color::Srgba(Srgba::hex("#FF0000").unwrap()),
                vec![
                    Color::Srgba(Srgba::hex("#FF5900").unwrap()),
                    Color::Srgba(Srgba::hex("#FF0000").unwrap()),
                    Color::Srgba(Srgba::hex("#FF9900").unwrap()),
                    Color::Srgba(Srgba::hex("#FFCF00").unwrap()),
                    Color::Srgba(Srgba::hex("#FFE808").unwrap()),
                ],
            )),
            Some(Fire {
                burn_radius: 1.5,
                chance_to_spread: 0.005,
                destroys_on_spread: false,
            }),
        )),
        Name::new("Wood Wall"),
    ));
}
