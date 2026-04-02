//! Scene loading and spawning
//!
//! A [`ParticleScene`] maps pixel colors in an image to particle type names.
//! Load a `.scn.ron` asset to get a `Handle<ParticleScene>`, then send a
//! [`SpawnSceneSignal`] to spawn it into the world at a given center position.
//!
//! # Asset format
//!
//! A scene consists of two files:
//!
//! - A palette-indexed PNG where each distinct color represents a particle type
//!   (transparent pixels are empty).
//! - A `.scn.ron` manifest mapping RGBA byte tuples to particle type names.
//!
//! See [`ParticleScene`] for the manifest format.
use std::borrow::Cow;
use std::collections::HashMap;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::{Particle, SpawnParticleSignal};
#[cfg(feature = "render")]
use crate::render::ForceColor;

/// Plugin for scene loading and spawning.
///
/// Registers the [`ParticleScene`] asset type, its loader, the
/// [`ParticleSceneRegistry`] resource, and the [`SpawnSceneSignal`] handler.
pub struct FallingSandScenesPlugin;

impl Plugin for FallingSandScenesPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ParticleScene>()
            .register_asset_loader(ParticleSceneLoader)
            .init_resource::<ParticleSceneRegistry>()
            .add_message::<SpawnSceneSignal>()
            .add_systems(Update, (populate_registry, handle_spawn_scene));
    }
}

/// A scene asset mapping image pixel colors to particle type names.
///
/// Loaded from a manifest that references a PNG image. The format for this is
///
/// ```ron
/// (
///   image: "scenes/my_scene.png",
///   layers: {
///     (255, 0, 0, 255): "Sand",
///     (0, 128, 0, 255): "Grass Wall",
///   },
/// )
/// ```
///
/// Each distinct RGBA color in the image maps to a particle type name.
/// Transparent (`alpha == 0`) pixels are skipped.
#[derive(Asset, TypePath, Debug)]
pub struct ParticleScene {
    /// Handle to the scene's image asset.
    pub image: Handle<Image>,
    /// Maps RGBA byte values to particle type names.
    pub layers: HashMap<[u8; 4], Cow<'static, str>>,
}

/// Resource for looking up loaded scene handles by name.
///
/// Automatically populated when [`ParticleScene`] assets finish loading.
/// The key is the scene's asset path (e.g. `"scenes/my_scene.scn.ron"`).
#[derive(Resource, Default, Debug)]
pub struct ParticleSceneRegistry {
    /// Maps scene asset paths to their handles.
    pub scenes: HashMap<String, Handle<ParticleScene>>,
}

/// Signal to spawn a [`ParticleScene`] into the world.
///
/// Particles are placed relative to `center`. Positions with existing particles
/// are skipped unless `overwrite_existing` is `true`.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::scenes::{SpawnSceneSignal, ParticleSceneRegistry};
///
/// fn spawn(
///     registry: Res<ParticleSceneRegistry>,
///     mut writer: MessageWriter<SpawnSceneSignal>,
/// ) {
///     if let Some(handle) = registry.scenes.get("scenes/my_scene.scn.ron") {
///         writer.write(SpawnSceneSignal::new(handle.clone(), IVec2::new(0, 0)));
///     }
/// }
/// ```
#[derive(Event, Message, Clone, Debug, Reflect)]
pub struct SpawnSceneSignal {
    /// Handle to the scene asset to spawn.
    pub scene: Handle<ParticleScene>,
    /// World position for the center of the spawned scene.
    pub center: IVec2,
    /// If `true`, overwrite existing particles at occupied positions.
    /// Defaults to `false` (skip occupied positions).
    pub overwrite_existing: bool,
}

impl SpawnSceneSignal {
    /// Create a spawn signal with default settings (skip occupied positions).
    #[must_use]
    pub const fn new(scene: Handle<ParticleScene>, center: IVec2) -> Self {
        Self {
            scene,
            center,
            overwrite_existing: false,
        }
    }

    /// Set whether existing particles should be overwritten.
    #[must_use]
    pub const fn with_overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite_existing = overwrite;
        self
    }
}

#[derive(Deserialize, Serialize)]
struct ParticleSceneManifest {
    image: String,
    layers: HashMap<(u8, u8, u8, u8), String>,
}

#[derive(TypePath)]
struct ParticleSceneLoader;

impl AssetLoader for ParticleSceneLoader {
    type Asset = ParticleScene;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let manifest: ParticleSceneManifest = ron::de::from_bytes(&bytes)?;

        let image_path = if let Some(parent) = load_context.path().parent() {
            parent
                .path()
                .join(&manifest.image)
                .to_string_lossy()
                .to_string()
        } else {
            manifest.image.clone()
        };
        let image = load_context.load(image_path);

        let layers = manifest
            .layers
            .into_iter()
            .map(|(color, name)| (color.into(), Cow::Owned(name)))
            .collect();

        Ok(ParticleScene { image, layers })
    }

    fn extensions(&self) -> &[&str] {
        &["scn.ron"]
    }
}

#[allow(clippy::needless_pass_by_value)]
fn populate_registry(
    mut registry: ResMut<ParticleSceneRegistry>,
    asset_server: Res<AssetServer>,
    mut events: MessageReader<AssetEvent<ParticleScene>>,
    scenes: Res<Assets<ParticleScene>>,
) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            if scenes.get(*id).is_some() {
                if let Some(path) = asset_server.get_path(*id) {
                    let handle: Handle<ParticleScene> = asset_server.load(path.clone());
                    registry.scenes.insert(path.to_string(), handle);
                }
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn handle_spawn_scene(
    mut msgr_spawn_scene_signal: MessageReader<SpawnSceneSignal>,
    scenes: Res<Assets<ParticleScene>>,
    images: Res<Assets<Image>>,
    mut spawn_writer: MessageWriter<SpawnParticleSignal>,
) {
    for signal in msgr_spawn_scene_signal.read() {
        let Some(scene) = scenes.get(&signal.scene) else {
            warn!("SpawnSceneSignal: scene asset not loaded yet");
            continue;
        };

        let Some(image) = images.get(&scene.image) else {
            warn!("SpawnSceneSignal: scene image not loaded yet");
            continue;
        };

        let Some(data) = image.data.as_ref() else {
            warn!("SpawnSceneSignal: asset has no pixel data");
            continue;
        };

        let width = image.width() as i32;
        let height = image.height() as i32;
        let half_w = width / 2;
        let half_h = height / 2;
        let mut positions_by_layer: HashMap<[u8; 4], Vec<IVec2>> = HashMap::new();

        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                let rgba = [data[idx], data[idx + 1], data[idx + 2], data[idx + 3]];

                if rgba[3] == 0 {
                    continue;
                }

                if !scene.layers.contains_key(&rgba) {
                    continue;
                }

                let world_x = signal.center.x + (x - half_w);
                let world_y = signal.center.y + (half_h - y);

                positions_by_layer
                    .entry(rgba)
                    .or_default()
                    .push(IVec2::new(world_x, world_y));
            }
        }

        for (rgba, positions) in positions_by_layer {
            let name = &scene.layers[&rgba];
            let particle = Particle::from(name.to_string());

            #[cfg(feature = "render")]
            let color = Color::srgba(
                f32::from(rgba[0]) / 255.0,
                f32::from(rgba[1]) / 255.0,
                f32::from(rgba[2]) / 255.0,
                f32::from(rgba[3]) / 255.0,
            );

            if signal.overwrite_existing {
                let mut sig = SpawnParticleSignal {
                    particle,
                    positions,
                    overwrite_existing: true,
                    on_spawn: None,
                };
                #[cfg(feature = "render")]
                {
                    sig = sig.with_on_spawn(move |cmd| {
                        cmd.insert(ForceColor(color));
                    });
                }
                spawn_writer.write(sig);
            } else {
                for pos in positions {
                    let mut sig = SpawnParticleSignal {
                        particle: particle.clone(),
                        positions: vec![pos],
                        overwrite_existing: false,
                        on_spawn: None,
                    };
                    #[cfg(feature = "render")]
                    {
                        sig = sig.with_on_spawn(move |cmd| {
                            cmd.insert(ForceColor(color));
                        });
                    }
                    spawn_writer.write(sig);
                }
            }
        }
    }
}
