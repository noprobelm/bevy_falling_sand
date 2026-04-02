//! Particle color assignment — profiles, components, and propagation.

/// Color components — profiles, assignment modes, and per-particle color state.
pub mod components;

use bevy::prelude::*;
use bevy_turborand::prelude::*;

pub use components::*;

use crate::core::{GridPosition, ParticleSyncExt, SyncParticleTypeChildrenSignal};

pub(super) struct ParticleColorPlugin;

impl Plugin for ParticleColorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(components::ComponentsPlugin)
            .register_particle_propagator(propagate_color)
            .add_systems(Update, load_texture_handles);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn load_texture_handles(
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    mut asset_events: MessageReader<AssetEvent<Image>>,
    mut profiles: Query<(Entity, &mut ColorProfile)>,
    mut sync_writer: MessageWriter<SyncParticleTypeChildrenSignal>,
) {
    for (_, mut profile) in profiles.iter_mut() {
        let ColorSource::Texture(ref mut tex) = profile.source else {
            continue;
        };
        if tex.handle.is_none() {
            tex.handle = Some(asset_server.load(&tex.path));
        }
    }

    for event in asset_events.read() {
        let AssetEvent::LoadedWithDependencies { id } = event else {
            continue;
        };

        if images.get(*id).is_none() {
            continue;
        }

        for (type_entity, profile) in profiles.iter() {
            if let ColorSource::Texture(ref tex) = profile.source {
                if tex.handle.as_ref().is_some_and(|h| h.id() == *id) {
                    sync_writer.write(SyncParticleTypeChildrenSignal::from_parent_handle(
                        type_entity,
                    ));
                }
            }
        }
    }
}

fn propagate_color(entity: Entity, parent: Entity, commands: &mut Commands) {
    commands.queue(move |world: &mut World| {
        let force_color = world.get::<ForceColor>(entity).map(|fc| fc.0);
        if let Some(color) = force_color {
            world.entity_mut(entity).insert(ParticleColor(color));
            return;
        }

        let with_color = world.get::<WithColor>(entity).map(|wc| wc.0);

        let has_profile = world.get::<ColorProfile>(parent).is_some();
        if !has_profile {
            world.entity_mut(entity).remove::<ParticleColor>();
            return;
        }

        let source = world.get::<ColorProfile>(parent).unwrap().source.clone();
        if let ColorSource::Texture(ref tex) = source {
            let pos = world.get::<GridPosition>(entity).map(|gp| gp.0);
            let color = pos.and_then(|p| {
                let handle = tex.handle.as_ref()?;
                let images = world.resource::<Assets<Image>>();
                let image = images.get(handle)?;
                let data = image.data.as_ref()?;
                let width = image.width() as i32;
                let height = image.height() as i32;
                let px = p.x.rem_euclid(width) as usize;
                let py = p.y.rem_euclid(height) as usize;
                let idx = (py * width as usize + px) * 4;
                Some(Color::srgba(
                    f32::from(data[idx]) / 255.0,
                    f32::from(data[idx + 1]) / 255.0,
                    f32::from(data[idx + 2]) / 255.0,
                    f32::from(data[idx + 3]) / 255.0,
                ))
            });
            if let Some(color) = color {
                world.entity_mut(entity).insert(ParticleColor(color));
            }
            return;
        }

        if let Some(color_idx) = with_color {
            if let Some(color) = world.get::<ColorProfile>(parent).unwrap().index(color_idx) {
                world
                    .entity_mut(entity)
                    .insert((ParticleColor(color), ColorIndex(color_idx)));
            }
            return;
        }

        let assignment = world
            .get::<ColorProfile>(parent)
            .unwrap()
            .assignment
            .clone();

        match assignment {
            ColorAssignment::Sequential => {
                let mut profile = world.get_mut::<ColorProfile>(parent).unwrap();
                if let Some(color) = profile.next() {
                    let idx = match &profile.source {
                        ColorSource::Palette(palette) => palette.index,
                        ColorSource::Gradient(gradient) => gradient.index,
                        ColorSource::Texture(_) => unreachable!(),
                    };
                    world
                        .entity_mut(entity)
                        .insert((ParticleColor(color), ColorIndex(idx)));
                }
            }
            ColorAssignment::Random => {
                let profile = world.get::<ColorProfile>(parent).unwrap().clone();
                let mut rng = world.resource_mut::<GlobalRng>();
                if let Some((color, idx)) = profile.random_with_index(rng.as_mut()) {
                    world
                        .entity_mut(entity)
                        .insert((ParticleColor(color), ColorIndex(idx)));
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::{Particle, ParticleType, SpawnParticleSignal},
        render::{FallingSandRenderPlugin, ParticleColor},
        FallingSandMinimalPlugin,
    };
    use bevy::{asset::AssetPlugin, image::ImagePlugin};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(ImagePlugin::default());
        app.init_resource::<Assets<Mesh>>();
        app.add_plugins(FallingSandMinimalPlugin::default());
        app.add_plugins(FallingSandRenderPlugin);
        app
    }

    #[test]
    fn gradient_produces_valid_colors() {
        let profile = ColorProfile::gradient(
            Color::srgba(1.0, 0.0, 0.0, 1.0),
            Color::srgba(0.0, 0.0, 1.0, 1.0),
            10,
        );

        for idx in [0, 5, 9] {
            let color = profile.index(idx).unwrap();
            let srgba = color.to_srgba();
            assert!(srgba.red.is_finite());
            assert!(srgba.green.is_finite());
            assert!(srgba.blue.is_finite());
            assert!(srgba.alpha.is_finite());
        }
    }

    #[test]
    fn gradient_hsv_produces_valid_colors() {
        let profile = ColorProfile {
            source: ColorSource::Gradient(ColorGradient {
                start: Color::srgba(1.0, 0.0, 0.0, 1.0),
                end: Color::srgba(0.0, 0.0, 1.0, 1.0),
                index: 0,
                steps: 10,
                hsv_interpolation: true,
            }),
            assignment: ColorAssignment::Sequential,
        };

        for idx in 0..10 {
            let color = profile.index(idx).unwrap();
            let srgba = color.to_srgba();
            assert!(srgba.red >= 0.0 && srgba.red <= 1.01);
            assert!(srgba.green >= 0.0 && srgba.green <= 1.01);
            assert!(srgba.blue >= 0.0 && srgba.blue <= 1.01);
            assert!(srgba.alpha >= 0.0 && srgba.alpha <= 1.01);
        }
    }

    #[test]
    fn sequential_gradient_produces_different_colors() {
        let mut app = setup_app();

        app.world_mut().spawn((
            ParticleType::new("TestSequential"),
            ColorProfile {
                source: ColorSource::Gradient(ColorGradient {
                    start: Color::srgba(1.0, 0.0, 0.0, 1.0),
                    end: Color::srgba(0.0, 0.0, 1.0, 1.0),
                    index: 0,
                    steps: 10,
                    hsv_interpolation: false,
                }),
                assignment: ColorAssignment::Sequential,
            },
        ));

        for i in 0..5 {
            app.world_mut()
                .resource_mut::<Messages<SpawnParticleSignal>>()
                .write(SpawnParticleSignal::new(
                    Particle::new("TestSequential"),
                    IVec2::new(i, 0),
                ));
        }

        for _ in 0..5 {
            app.update();
        }

        let colors: Vec<Color> = app
            .world_mut()
            .query::<&ParticleColor>()
            .iter(app.world())
            .map(|pc| pc.0)
            .collect();

        assert!(colors.len() >= 2);
        let first = colors[0].to_srgba();
        let last = colors[colors.len() - 1].to_srgba();
        assert!(
            (first.red - last.red).abs() > 0.01 || (first.blue - last.blue).abs() > 0.01,
            "Sequential gradient should produce different colors"
        );
    }

    #[test]
    fn palette_registration() {
        let mut app = setup_app();

        let palette_colors = vec![
            Color::srgba(1.0, 0.0, 0.0, 1.0),
            Color::srgba(0.0, 1.0, 0.0, 1.0),
            Color::srgba(0.0, 0.0, 1.0, 1.0),
        ];

        app.world_mut().spawn((
            ParticleType::new("TestPalette"),
            ColorProfile::palette(palette_colors.clone()),
        ));

        app.world_mut()
            .resource_mut::<Messages<SpawnParticleSignal>>()
            .write(SpawnParticleSignal::new(
                Particle::new("TestPalette"),
                IVec2::new(0, 0),
            ));

        for _ in 0..5 {
            app.update();
        }

        let colors: Vec<Color> = app
            .world_mut()
            .query::<&ParticleColor>()
            .iter(app.world())
            .map(|pc| pc.0)
            .collect();

        assert_eq!(colors.len(), 1);
        let assigned = colors[0].to_srgba();
        let is_palette_color = palette_colors.iter().any(|c| {
            let s = c.to_srgba();
            (s.red - assigned.red).abs() < 0.01
                && (s.green - assigned.green).abs() < 0.01
                && (s.blue - assigned.blue).abs() < 0.01
        });
        assert!(is_palette_color, "Assigned color should be from palette");
    }

    #[test]
    fn force_color_overrides_profile() {
        let mut app = setup_app();

        app.world_mut().spawn((
            ParticleType::new("TestForce"),
            ColorProfile::palette(vec![Color::srgba(1.0, 0.0, 0.0, 1.0)]),
        ));

        let forced = Color::srgba(0.0, 1.0, 1.0, 1.0);
        app.world_mut()
            .resource_mut::<Messages<SpawnParticleSignal>>()
            .write(
                SpawnParticleSignal::new(Particle::new("TestForce"), IVec2::new(0, 0))
                    .with_on_spawn({
                        move |cmd| {
                            cmd.insert(ForceColor(forced));
                        }
                    }),
            );

        for _ in 0..5 {
            app.update();
        }

        let colors: Vec<Color> = app
            .world_mut()
            .query::<&ParticleColor>()
            .iter(app.world())
            .map(|pc| pc.0)
            .collect();

        assert_eq!(colors.len(), 1);
        let assigned = colors[0].to_srgba();
        let expected = forced.to_srgba();
        assert!(
            (assigned.red - expected.red).abs() < 0.01
                && (assigned.green - expected.green).abs() < 0.01
                && (assigned.blue - expected.blue).abs() < 0.01,
            "ForceColor should override the profile"
        );
    }

    #[test]
    fn palette_mutation_methods() {
        let mut profile = ColorProfile::palette(vec![
            Color::srgba(1.0, 0.0, 0.0, 1.0),
            Color::srgba(0.0, 1.0, 0.0, 1.0),
        ]);

        assert_eq!(profile.colors().unwrap().len(), 2);

        profile.add_color(Color::srgba(0.0, 0.0, 1.0, 1.0));
        assert_eq!(profile.colors().unwrap().len(), 3);

        let new_color = Color::srgba(1.0, 1.0, 0.0, 1.0);
        profile.edit_color(0, new_color);
        assert_eq!(profile.colors().unwrap()[0].to_srgba().green, 1.0);

        assert_eq!(profile.remove_color(2), Some(true));
        assert_eq!(profile.colors().unwrap().len(), 2);

        assert_eq!(profile.remove_color(0), Some(true));
        assert_eq!(profile.colors().unwrap().len(), 1);

        assert_eq!(
            profile.remove_color(0),
            Some(false),
            "Should not remove last color"
        );
        assert_eq!(profile.colors().unwrap().len(), 1);
    }

    #[test]
    fn colors_returns_all_for_gradient() {
        let profile = ColorProfile::gradient(
            Color::srgba(1.0, 0.0, 0.0, 1.0),
            Color::srgba(0.0, 0.0, 1.0, 1.0),
            5,
        );
        let colors = profile.colors().unwrap();
        assert_eq!(colors.len(), 5);
    }
}
