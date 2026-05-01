//! Scene loading and spawning.
//!
//! A [`ParticleScene`] is an ordered list of [`SceneLayer`]s. Each layer carries
//! its own image and is processed independently, so a [`SceneLayer::Particles`]
//! layer can sit visually on top of a [`SceneLayer::Background`] layer without
//! any per-pixel ambiguity.
//!
//! Load a `.scn.ron` asset to get a `Handle<ParticleScene>`, then send a
//! [`SpawnSceneSignal`] to spawn it into the world at a given center position.
//! Send a [`DespawnSceneSignal`] with the instance's [`ParticleSceneRoot`]
//! entity to tear it back down.
//!
//! # Asset format
//!
//! ```ron
//! (
//!   layers: [
//!     Background((image: "scene.sky.png")),
//!     Background((image: "scene.hills.png")),
//!     Particles((
//!       image: "scene.terrain.png",
//!       colors: {
//!         (255, 0, 0, 255): "Sand",
//!         (0, 128, 0, 255): "Grass Wall",
//!       },
//!     )),
//!   ],
//! )
//! ```
//!
//! Layer order is back-to-front: the first entry is drawn first (deepest), each
//! subsequent entry composites on top. [`SceneLayer::Background`] sprites are
//! always placed at `z < 0` so particles (at `z >= 0`) render in front of every
//! background regardless of list order.
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::{DespawnParticleSignal, Particle, SpawnParticleSignal};
#[cfg(feature = "render")]
use crate::render::ForceColor;

/// Plugin for scene loading and spawning.
///
/// Registers the [`ParticleScene`] asset type and its loader, the
/// [`ParticleSceneRegistry`] resource, and the [`SpawnSceneSignal`] /
/// [`DespawnSceneSignal`] handlers.
pub struct FallingSandScenesPlugin;

impl Plugin for FallingSandScenesPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ParticleScene>()
            .register_asset_loader(ParticleSceneLoader)
            .init_resource::<ParticleSceneRegistry>()
            .add_message::<SpawnSceneSignal>()
            .add_message::<DespawnSceneSignal>()
            .register_type::<ParticleSceneRoot>()
            .register_type::<ParticleSceneInstance>()
            .add_systems(
                Update,
                (populate_registry, handle_spawn_scene, handle_despawn_scene),
            );
    }
}

/// A scene asset: an ordered list of layers, each with its own image.
#[derive(Asset, TypePath, Debug)]
pub struct ParticleScene {
    /// Layers in back-to-front render order.
    pub layers: Vec<SceneLayer>,
}

/// A single layer of a [`ParticleScene`].
///
/// New layer kinds (zones, rigid body shapes, …) can be added as additional
/// variants without touching existing variants.
#[derive(Debug)]
pub enum SceneLayer {
    /// Pixels become particles at the matching world positions. Pixels whose
    /// RGBA value isn't a key in `colors` are skipped.
    Particles {
        /// Handle to the layer's image asset.
        image: Handle<Image>,
        /// Maps RGBA byte values in this layer to a particle type name.
        colors: HashMap<[u8; 4], Cow<'static, str>>,
    },
    /// Pixels render as a stock [`Sprite`]. No particles spawned.
    ///
    /// The sprite is placed at `z < 0` so particles always draw in front.
    Background {
        /// Handle to the layer's image asset.
        image: Handle<Image>,
    },
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

/// Marker on the root entity of a spawned [`ParticleScene`] instance.
///
/// One root is created per [`SpawnSceneSignal`]. Every particle and every
/// background sprite belonging to that instance carry a
/// [`ParticleSceneInstance`] pointing at this root.
#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component, Default, Debug)]
pub struct ParticleSceneRoot;

/// Links a spawned entity to its [`ParticleSceneRoot`].
///
/// Inserted on every particle and on each background sprite produced by a
/// [`SpawnSceneSignal`]. Use it to query, filter, or batch-despawn the members
/// of a scene instance — see [`DespawnSceneSignal`].
#[derive(Component, Copy, Clone, Debug, Reflect)]
#[reflect(Component, Debug)]
pub struct ParticleSceneInstance(pub Entity);

/// Signal to spawn a [`ParticleScene`] into the world.
///
/// Particles and background sprites are placed relative to `center`. Particle
/// positions already occupied are skipped unless `overwrite_existing` is
/// `true`.
///
/// Pre-allocate a [`ParticleSceneRoot`] entity and pass it via
/// [`SpawnSceneSignal::with_root`] when you want to retain a handle for
/// despawning later. Otherwise a fresh root is spawned automatically and you'll
/// have to query for it (e.g. via [`Added<ParticleSceneRoot>`](Added)).
#[derive(Event, Message, Clone, Debug, Reflect)]
pub struct SpawnSceneSignal {
    /// Handle to the scene asset to spawn.
    pub scene: Handle<ParticleScene>,
    /// World position for the center of the spawned scene.
    pub center: IVec2,
    /// If `true`, overwrite existing particles at occupied positions.
    /// Defaults to `false` (skip occupied positions).
    pub overwrite_existing: bool,
    /// Pre-allocated [`ParticleSceneRoot`] entity. If `None`, a new one is
    /// spawned and tagged automatically.
    pub root: Option<Entity>,
}

impl SpawnSceneSignal {
    /// Create a spawn signal with default settings (skip occupied positions,
    /// auto-spawn the root entity).
    #[must_use]
    pub const fn new(scene: Handle<ParticleScene>, center: IVec2) -> Self {
        Self {
            scene,
            center,
            overwrite_existing: false,
            root: None,
        }
    }

    /// Set whether existing particles should be overwritten.
    #[must_use]
    pub const fn with_overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite_existing = overwrite;
        self
    }

    /// Use an existing entity as the [`ParticleSceneRoot`] for this instance.
    ///
    /// The [`ParticleSceneRoot`] component is inserted on `root` if absent, so
    /// callers can spawn an empty entity and pass its id here.
    #[must_use]
    pub const fn with_root(mut self, root: Entity) -> Self {
        self.root = Some(root);
        self
    }
}

/// Despawn every entity belonging to a scene instance.
///
/// Walks all entities carrying a [`ParticleSceneInstance`] referring to
/// `root`. Particle members are routed through [`DespawnParticleSignal`] so the
/// simulation map stays consistent; non-particle members (background sprites)
/// and the root itself are despawned directly.
#[derive(Event, Message, Clone, Copy, Debug, Reflect)]
pub struct DespawnSceneSignal {
    /// The [`ParticleSceneRoot`] entity identifying the instance to despawn.
    pub root: Entity,
}

impl DespawnSceneSignal {
    /// Create a despawn signal targeting `root`.
    #[must_use]
    pub const fn new(root: Entity) -> Self {
        Self { root }
    }
}

#[derive(Deserialize, Serialize)]
struct ParticleSceneManifest {
    layers: Vec<SceneLayerManifest>,
}

#[derive(Deserialize, Serialize)]
enum SceneLayerManifest {
    Particles(ParticlesLayerManifest),
    Background(BackgroundLayerManifest),
}

#[derive(Deserialize, Serialize)]
struct ParticlesLayerManifest {
    image: String,
    colors: HashMap<(u8, u8, u8, u8), String>,
}

#[derive(Deserialize, Serialize)]
struct BackgroundLayerManifest {
    image: String,
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

        let parent_path = load_context.path().parent().map(|p| p.path().to_path_buf());
        let resolve = |image: &str| -> String {
            parent_path.as_ref().map_or_else(
                || image.to_string(),
                |parent| parent.join(image).to_string_lossy().to_string(),
            )
        };

        let layers = manifest
            .layers
            .into_iter()
            .map(|layer| match layer {
                SceneLayerManifest::Particles(ParticlesLayerManifest { image, colors }) => {
                    let handle = load_context.load(resolve(&image));
                    let colors = colors
                        .into_iter()
                        .map(|(rgba, name)| (rgba.into(), Cow::Owned(name)))
                        .collect();
                    SceneLayer::Particles {
                        image: handle,
                        colors,
                    }
                }
                SceneLayerManifest::Background(BackgroundLayerManifest { image }) => {
                    SceneLayer::Background {
                        image: load_context.load(resolve(&image)),
                    }
                }
            })
            .collect();

        Ok(ParticleScene { layers })
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
        if let AssetEvent::LoadedWithDependencies { id } = event
            && scenes.get(*id).is_some()
            && let Some(path) = asset_server.get_path(*id)
        {
            let handle: Handle<ParticleScene> = asset_server.load(path.clone());
            registry.scenes.insert(path.to_string(), handle);
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn handle_spawn_scene(
    mut msgr_spawn_scene_signal: MessageReader<SpawnSceneSignal>,
    scenes: Res<Assets<ParticleScene>>,
    images: Res<Assets<Image>>,
    mut spawn_writer: MessageWriter<SpawnParticleSignal>,
    mut commands: Commands,
) {
    for signal in msgr_spawn_scene_signal.read() {
        let Some(scene) = scenes.get(&signal.scene) else {
            warn!("SpawnSceneSignal: scene asset not loaded yet");
            continue;
        };

        let root = signal.root.unwrap_or_else(|| commands.spawn_empty().id());
        commands.entity(root).try_insert(ParticleSceneRoot);

        for (layer_index, layer) in scene.layers.iter().rev().enumerate() {
            match layer {
                SceneLayer::Particles { image, colors } => {
                    if !spawn_particles_layer(
                        &mut spawn_writer,
                        &images,
                        image,
                        colors,
                        signal.center,
                        signal.overwrite_existing,
                        root,
                    ) {
                        warn!("SpawnSceneSignal: particle layer image not loaded yet");
                    }
                }
                #[cfg(feature = "render")]
                SceneLayer::Background { image } => {
                    if !spawn_background_layer(
                        &mut commands,
                        &images,
                        image,
                        signal.center,
                        layer_index,
                        root,
                    ) {
                        warn!("SpawnSceneSignal: background layer image not loaded yet");
                    }
                }
                #[cfg(not(feature = "render"))]
                SceneLayer::Background { .. } => {
                    let _ = layer_index;
                }
            }
        }
    }
}

fn spawn_particles_layer(
    writer: &mut MessageWriter<SpawnParticleSignal>,
    images: &Assets<Image>,
    handle: &Handle<Image>,
    colors: &HashMap<[u8; 4], Cow<'static, str>>,
    center: IVec2,
    overwrite_existing: bool,
    root: Entity,
) -> bool {
    let Some(image) = images.get(handle) else {
        return false;
    };
    let Some(data) = image.data.as_ref() else {
        return false;
    };

    let width = image.width() as i32;
    let height = image.height() as i32;
    let half_w = width / 2;
    let half_h = height / 2;

    let mut positions_by_color: HashMap<[u8; 4], Vec<IVec2>> = HashMap::new();
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let rgba = [data[idx], data[idx + 1], data[idx + 2], data[idx + 3]];
            if rgba[3] == 0 || !colors.contains_key(&rgba) {
                continue;
            }
            let world_x = center.x + (x - half_w);
            let world_y = center.y + (half_h - y);
            positions_by_color
                .entry(rgba)
                .or_default()
                .push(IVec2::new(world_x, world_y));
        }
    }

    for (rgba, positions) in positions_by_color {
        let name = &colors[&rgba];
        write_particle_signals(
            writer,
            Particle::from(name.to_string()),
            positions,
            rgba,
            overwrite_existing,
            root,
        );
    }
    true
}

fn write_particle_signals(
    writer: &mut MessageWriter<SpawnParticleSignal>,
    particle: Particle,
    positions: Vec<IVec2>,
    rgba: [u8; 4],
    overwrite_existing: bool,
    root: Entity,
) {
    #[cfg(feature = "render")]
    let color = Color::srgba(
        f32::from(rgba[0]) / 255.0,
        f32::from(rgba[1]) / 255.0,
        f32::from(rgba[2]) / 255.0,
        f32::from(rgba[3]) / 255.0,
    );
    #[cfg(not(feature = "render"))]
    let _ = rgba;

    let decorate = move |sig: SpawnParticleSignal| {
        let sig = sig.with_on_spawn(move |cmd| {
            cmd.insert(ParticleSceneInstance(root));
        });
        #[cfg(feature = "render")]
        let sig = sig.with_on_spawn(move |cmd| {
            cmd.insert(ForceColor(color));
        });
        sig
    };

    if overwrite_existing {
        writer.write(decorate(SpawnParticleSignal {
            particle,
            positions,
            overwrite_existing: true,
            on_spawn: None,
        }));
    } else {
        for pos in positions {
            writer.write(decorate(SpawnParticleSignal {
                particle: particle.clone(),
                positions: vec![pos],
                overwrite_existing: false,
                on_spawn: None,
            }));
        }
    }
}

#[cfg(feature = "render")]
fn spawn_background_layer(
    commands: &mut Commands,
    images: &Assets<Image>,
    handle: &Handle<Image>,
    center: IVec2,
    layer_index: usize,
    root: Entity,
) -> bool {
    let Some(image) = images.get(handle) else {
        return false;
    };
    let width = image.width() as i32;
    let height = image.height() as i32;
    let half_w = width / 2;
    let half_h = height / 2;
    // Align the image's pixel grid to the particle grid: an integer pixel
    // column maps to a 1x1 world-unit cell centered on a particle position.
    // The shifts below cancel the half-pixel offset that arises whenever width
    // or height is even.
    let sprite_x = center.x as f32 - half_w as f32 + width as f32 / 2.0 - 0.5;
    let sprite_y = center.y as f32 + half_h as f32 - height as f32 / 2.0 + 0.5;
    // Negative z keeps every background behind the particle render layer; the
    // small per-index step preserves back-to-front order between backgrounds.
    let z = (layer_index as f32).mul_add(0.001, -1.0);
    commands.spawn((
        Sprite::from_image(handle.clone()),
        Transform::from_xyz(sprite_x, sprite_y, z),
        ParticleSceneInstance(root),
    ));
    true
}

#[allow(clippy::needless_pass_by_value)]
fn handle_despawn_scene(
    mut msgr_despawn_scene: MessageReader<DespawnSceneSignal>,
    members: Query<(Entity, &ParticleSceneInstance, Has<Particle>)>,
    mut despawn_particle_writer: MessageWriter<DespawnParticleSignal>,
    mut commands: Commands,
) {
    if msgr_despawn_scene.is_empty() {
        return;
    }

    let targets: HashSet<Entity> = msgr_despawn_scene.read().map(|s| s.root).collect();

    for (entity, instance, is_particle) in &members {
        if !targets.contains(&instance.0) {
            continue;
        }
        if is_particle {
            despawn_particle_writer.write(DespawnParticleSignal::from_entity(entity));
        } else {
            commands.entity(entity).try_despawn();
        }
    }

    for root in targets {
        commands.entity(root).try_despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_round_trips_plugin_output() {
        let ron = r#"(
  layers: [
    Background((image: "scene.sky.png")),
    Particles((
      image: "scene.terrain.png",
      colors: {
        (255, 0, 0, 255): "Sand",
        (0, 128, 0, 255): "Grass Wall",
      },
    )),
  ],
)"#;
        let manifest: ParticleSceneManifest = ron::de::from_str(ron).unwrap();
        assert_eq!(manifest.layers.len(), 2);
        match &manifest.layers[0] {
            SceneLayerManifest::Background(BackgroundLayerManifest { image }) => {
                assert_eq!(image, "scene.sky.png");
            }
            SceneLayerManifest::Particles(_) => panic!("expected Background first"),
        }
        match &manifest.layers[1] {
            SceneLayerManifest::Particles(ParticlesLayerManifest { image, colors }) => {
                assert_eq!(image, "scene.terrain.png");
                assert_eq!(
                    colors.get(&(255, 0, 0, 255)).map(String::as_str),
                    Some("Sand"),
                );
                assert_eq!(
                    colors.get(&(0, 128, 0, 255)).map(String::as_str),
                    Some("Grass Wall"),
                );
            }
            SceneLayerManifest::Background(_) => panic!("expected Particles second"),
        }
    }
}
