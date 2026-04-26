use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use bevy_turborand::{DelegatedRng, RngComponent};
use serde::{Deserialize, Serialize};

use crate::impl_particle_rng;

pub(super) struct ComponentsPlugin;

impl Plugin for ComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ColorRng>()
            .register_type::<ColorProfile>()
            .register_type::<ForceColor>()
            .register_type::<WithColor>();
    }
}

impl_particle_rng!(ColorRng, RngComponent);

/// Provides rng for coloring particles.
#[derive(Component, Clone, PartialEq, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ColorRng(pub RngComponent);

/// Particle colors are assigned either randomly or sequentially from their parent
/// [`ParticleType`](crate::core::ParticleType)'s `ColorProfile`.
///
/// A color profile can be defined from a [palette](ColorProfile::palette),
/// [gradient](ColorProfile::gradient), or [texture](ColorProfile::texture).
///
/// By default, palette and gradient color profiles generate colors **sequentially** according to
/// the order of the palette or gradient step reached. Use [`ColorAssignment::Random`] for random
/// color assignment.
///
/// # Palette
///
/// A palette picks from a discrete list of colors:
///
/// ```
/// use bevy::prelude::*;
/// use bevy_falling_sand::prelude::{ParticleType, ColorProfile};
///
/// fn setup(mut commands: Commands) {
///     commands.spawn((
///         ParticleType::new("Dirt"),
///         ColorProfile::palette(vec![
///             Color::Srgba(Srgba::hex("#916B4C").unwrap()),
///             Color::Srgba(Srgba::hex("#73573D").unwrap()),
///         ]),
///     ));
/// }
/// ```
///
/// # Gradient
///
/// A gradient interpolates between two or more colors. Each segment between consecutive
/// colors gets its own configurable number of samples:
///
/// ```
/// use bevy::prelude::*;
/// use bevy_falling_sand::prelude::{ParticleType, ColorProfile};
///
/// fn setup(mut commands: Commands) {
///     commands.spawn((
///         ParticleType::new("Colorful"),
///         ColorProfile::gradient(
///             vec![
///                 Color::hsla(0.0, 1.0, 0.5, 1.0),
///                 Color::hsla(180.0, 1.0, 0.5, 1.0),
///                 Color::hsla(360.0, 1.0, 0.5, 1.0),
///             ],
///             vec![2500, 2500],
///         ),
///     ));
/// }
/// ```
///
/// # Texture
///
/// A texture samples colors from an image file, tiling across the map by world position:
///
/// ```
/// use bevy::prelude::*;
/// use bevy_falling_sand::prelude::{ParticleType, ColorProfile};
///
/// fn setup(mut commands: Commands) {
///     commands.spawn((
///         ParticleType::new("Wood"),
///         ColorProfile::texture("textures/wood.png"),
///     ));
/// }
/// ```
#[derive(Component, Clone, PartialEq, Debug, Default, Reflect, Serialize, Deserialize)]
#[component(on_add = ColorProfile::on_add)]
#[reflect(Component, Default)]
#[type_path = "bfs_color::particle"]
pub struct ColorProfile {
    /// Source of colors (palette, gradient, or texture)
    pub source: ColorSource,
    /// Logic for color assignment
    pub assignment: ColorAssignment,
}

impl ColorProfile {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        if !world.entity(context.entity).contains::<ColorRng>() {
            world
                .commands()
                .entity(context.entity)
                .insert(ColorRng::default());
        }
    }
}

impl ColorProfile {
    /// Creates a color profile with a palette of colors
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::prelude::{ParticleType, ColorProfile};
    ///
    /// fn setup(mut commands: Commands) {
    ///     commands.spawn((
    ///         ParticleType::new("Dirt"),
    ///         ColorProfile::palette(vec![
    ///             Color::Srgba(Srgba::hex("#916B4C").unwrap()),
    ///             Color::Srgba(Srgba::hex("#73573D").unwrap()),
    ///         ]),
    ///     ));
    /// }
    /// ```
    #[must_use]
    pub fn palette(colors: Vec<Color>) -> Self {
        Self {
            source: ColorSource::Palette(Palette { index: 0, colors }),
            ..default()
        }
    }

    #[must_use]
    /// Creates a color profile with a multi-stop gradient.
    ///
    /// `colors` must have at least 2 entries, and `steps` must have exactly
    /// `colors.len() - 1` entries — one per segment between consecutive colors.
    /// Each `steps[k]` defines how many color samples span the segment from
    /// `colors[k]` to `colors[k + 1]`.
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::prelude::{ParticleType, ColorProfile};
    ///
    /// fn setup(mut commands: Commands) {
    ///     commands.spawn((
    ///         ParticleType::new("Colorful"),
    ///         ColorProfile::gradient(
    ///             vec![
    ///                 Color::hsla(0.0, 1.0, 0.5, 1.0),
    ///                 Color::hsla(180.0, 1.0, 0.5, 1.0),
    ///                 Color::hsla(360.0, 1.0, 0.5, 1.0),
    ///             ],
    ///             vec![2500, 2500],
    ///         ),
    ///     ));
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `colors.len() < 2`, if `steps.len() != colors.len() - 1`,
    /// or if any segment has zero steps.
    pub fn gradient(colors: Vec<Color>, steps: Vec<usize>) -> Self {
        assert!(
            colors.len() >= 2,
            "ColorProfile::gradient requires at least 2 colors"
        );
        assert!(
            steps.len() == colors.len() - 1,
            "ColorProfile::gradient requires steps.len() == colors.len() - 1"
        );
        assert!(
            steps.iter().all(|s| *s > 0),
            "ColorProfile::gradient requires every segment to have at least 1 step"
        );
        Self {
            source: ColorSource::Gradient(ColorGradient {
                colors,
                steps,
                index: 0,
                hsv_interpolation: false,
            }),
            ..default()
        }
    }

    /// Creates a color profile that samples colors from a texture image.
    ///
    /// Particles are colored by sampling the texture at their world position,
    /// tiling seamlessly across the map.
    ///
    /// The image is loaded asynchronously at runtime. Until it loads, particles
    /// will not receive a color.
    #[must_use]
    pub fn texture(path: impl Into<String>) -> Self {
        Self {
            source: ColorSource::Texture(TextureSource {
                path: path.into(),
                handle: None,
            }),
            ..default()
        }
    }

    /// Gets a random color from the profile along with its index.
    ///
    /// Returns `None` for texture-based profiles, which are colored by world position instead.
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::prelude::ColorProfile;
    /// use bevy_turborand::prelude::*;
    ///
    /// let profile = ColorProfile::palette(vec![
    ///     Color::WHITE,
    ///     Color::BLACK,
    /// ]);
    /// let mut rng = RngComponent::default();
    /// let (color, index) = profile.random_with_index(&mut rng).unwrap();
    /// assert!(index < 2);
    /// ```
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn random_with_index<R: DelegatedRng>(&self, rng: &mut R) -> Option<(Color, usize)> {
        match &self.source {
            ColorSource::Palette(palette) => {
                let color_index = rng.index(0..palette.colors.len());
                Some((palette.colors[color_index], color_index))
            }
            ColorSource::Gradient(gradient) => {
                let total = gradient.total_steps();
                let random_step = rng.index(0..total);
                Some((gradient.sample(random_step), random_step))
            }
            ColorSource::Texture(_) => {
                warn!("random_with_index is not supported for texture-based ColorProfiles");
                None
            }
        }
    }

    /// Gets a color at a specific index.
    ///
    /// Returns `None` for texture-based profiles, which are colored by world position instead.
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::prelude::ColorProfile;
    ///
    /// let profile = ColorProfile::palette(vec![
    ///     Color::WHITE,
    ///     Color::BLACK,
    /// ]);
    /// assert_eq!(profile.index(0), Some(Color::WHITE));
    /// assert_eq!(profile.index(1), Some(Color::BLACK));
    /// ```
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn index(&self, index: usize) -> Option<Color> {
        match &self.source {
            ColorSource::Palette(palette) => Some(palette.colors[index]),
            ColorSource::Gradient(gradient) => Some(gradient.sample(index)),
            ColorSource::Texture(_) => {
                warn!("index is not supported for texture-based ColorProfiles");
                None
            }
        }
    }

    /// Get the next particle color in the profile.
    ///
    /// Returns `None` for texture-based profiles, which are colored by world position instead.
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::prelude::ColorProfile;
    ///
    /// let mut profile = ColorProfile::palette(vec![
    ///     Color::WHITE,
    ///     Color::BLACK,
    /// ]);
    /// let first = profile.next();
    /// let second = profile.next();
    /// assert_eq!(first, Some(Color::BLACK));
    /// assert_eq!(second, Some(Color::WHITE));
    /// ```
    #[allow(clippy::cast_precision_loss, clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Color> {
        match &mut self.source {
            ColorSource::Palette(palette) => {
                palette.index = (palette.index + 1) % palette.colors.len();
                Some(palette.colors[palette.index])
            }
            ColorSource::Gradient(gradient) => {
                let total = gradient.total_steps();
                gradient.index = (gradient.index + 1) % total;
                Some(gradient.sample(gradient.index))
            }
            ColorSource::Texture(_) => {
                warn!("next is not supported for texture-based ColorProfiles");
                None
            }
        }
    }

    /// Adds a color to the palette.
    ///
    /// Returns `None` for non-palette profiles.
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::prelude::ColorProfile;
    ///
    /// let mut profile = ColorProfile::palette(vec![Color::WHITE]);
    /// profile.add_color(Color::BLACK);
    /// assert_eq!(profile.colors().unwrap().len(), 2);
    /// ```
    pub fn add_color(&mut self, color: Color) -> Option<()> {
        match &mut self.source {
            ColorSource::Palette(palette) => {
                palette.colors.push(color);
                Some(())
            }
            ColorSource::Gradient(_) | ColorSource::Texture(_) => {
                warn!("add_color is only supported for palette-based ColorProfiles");
                None
            }
        }
    }

    /// Removes a color from the palette at the given index.
    ///
    /// Returns `Some(true)` if the color was removed, `Some(false)` if the palette
    /// only has one color remaining, or `None` for non-palette profiles.
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::prelude::ColorProfile;
    ///
    /// let mut profile = ColorProfile::palette(vec![
    ///     Color::WHITE,
    ///     Color::BLACK,
    /// ]);
    /// assert_eq!(profile.remove_color(1), Some(true));
    /// assert_eq!(profile.colors().unwrap().len(), 1);
    /// assert_eq!(profile.remove_color(0), Some(false));
    /// ```
    pub fn remove_color(&mut self, index: usize) -> Option<bool> {
        match &mut self.source {
            ColorSource::Palette(palette) => {
                if palette.colors.len() <= 1 {
                    return Some(false);
                }
                palette.colors.remove(index);
                if palette.index >= palette.colors.len() {
                    palette.index = palette.colors.len() - 1;
                }
                Some(true)
            }
            ColorSource::Gradient(_) | ColorSource::Texture(_) => {
                warn!("remove_color is only supported for palette-based ColorProfiles");
                None
            }
        }
    }

    /// Edits the color at the given index and updates current color if needed.
    ///
    /// Returns `None` for non-palette profiles.
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::prelude::ColorProfile;
    ///
    /// let mut profile = ColorProfile::palette(vec![Color::WHITE]);
    /// profile.edit_color(0, Color::BLACK);
    /// assert_eq!(profile.index(0), Some(Color::BLACK));
    /// ```
    pub fn edit_color(&mut self, index: usize, new_color: Color) -> Option<()> {
        match &mut self.source {
            ColorSource::Palette(palette) => {
                palette.colors[index] = new_color;
                Some(())
            }
            ColorSource::Gradient(_) | ColorSource::Texture(_) => {
                warn!("edit_color is only supported for palette-based ColorProfiles");
                None
            }
        }
    }

    /// Get all colors from the profile. For palettes, returns the color list directly.
    /// For gradients, returns the full set of interpolated colors.
    ///
    /// Returns `None` for texture-based profiles, which are colored by world position instead.
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::prelude::ColorProfile;
    ///
    /// let profile = ColorProfile::palette(vec![
    ///     Color::WHITE,
    ///     Color::BLACK,
    /// ]);
    /// assert_eq!(profile.colors().unwrap().len(), 2);
    ///
    /// let gradient = ColorProfile::gradient(
    ///     vec![Color::BLACK, Color::WHITE],
    ///     vec![10],
    /// );
    /// assert_eq!(gradient.colors().unwrap().len(), 10);
    /// ```
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn colors(&self) -> Option<Vec<Color>> {
        match &self.source {
            ColorSource::Palette(palette) => Some(palette.colors.clone()),
            ColorSource::Gradient(gradient) => {
                let total = gradient.total_steps();
                let mut colors = Vec::with_capacity(total);
                for i in 0..total {
                    colors.push(gradient.sample(i));
                }
                Some(colors)
            }
            ColorSource::Texture(_) => {
                warn!("colors is not supported for texture-based ColorProfiles");
                None
            }
        }
    }
}

/// Overrides the parent [`ParticleType`](crate::prelude::ParticleType)'s [`ColorProfile`]
/// assignment.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::prelude::*;
///
/// fn spawn_cyan_particle(mut writer: MessageWriter<SpawnParticleSignal>) {
///     let forced = Color::srgba(0.0, 1.0, 1.0, 1.0);
///     writer.write(
///         SpawnParticleSignal::new(Particle::new("Sand"), IVec2::new(5, 5))
///             .with_on_spawn(move |cmd| {
///                 cmd.insert(ForceColor(forced));
///             }),
///     );
/// }
/// ```
#[derive(Component, Copy, Clone, Default, PartialEq, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct ForceColor(pub Color);

/// Component that allows particles to change color based on an input chance.
#[derive(
    Component,
    Copy,
    Clone,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Reflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component)]
pub struct ColorIndex(pub usize);

/// Component that stores a color index for scene preservation.
/// When present on a particle, the scene save system will preserve this color index
/// and restore it when loading the scene.
#[derive(
    Component,
    Copy,
    Clone,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Reflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component)]
pub struct WithColor(pub usize);

/// Color assignment logic
#[derive(Clone, Eq, PartialEq, Debug, Default, Reflect, Serialize, Deserialize)]
#[type_path = "bfs_color::particle"]
pub enum ColorAssignment {
    /// Colors are assigned sequentially from the palette or gradient
    Sequential,
    /// Colors are assigned randomly from the palette or gradient
    #[default]
    Random,
}

/// Palette color configuration for particles
#[derive(Clone, PartialEq, Debug, Reflect, Serialize, Deserialize)]
#[type_path = "bfs_color::particle"]
pub struct Palette {
    /// Current index in the palette
    pub index: usize,
    /// List of colors in the palette
    pub colors: Vec<Color>,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            index: 0,
            colors: vec![Color::WHITE],
        }
    }
}

/// Color gradient configuration for particles.
///
/// A gradient is composed of one or more segments. Each segment connects two consecutive
/// entries in [`ColorGradient::colors`] and produces [`ColorGradient::steps`]`[k]` color
/// samples, with `t` interpolating from `0.0` to `1.0` within the segment. Boundary colors
/// between adjacent segments may appear as samples in both segments.
#[derive(Clone, PartialEq, Debug, Reflect, Serialize, Deserialize)]
#[type_path = "bfs_color::particle"]
pub struct ColorGradient {
    /// Color stops. Must contain at least 2 entries.
    pub colors: Vec<Color>,
    /// Number of color samples in each segment between consecutive colors.
    /// Must satisfy `steps.len() == colors.len() - 1` and every entry must be > 0.
    pub steps: Vec<usize>,
    /// Current sample index in the flattened gradient.
    pub index: usize,
    /// If true, interpolate in HSV space for rainbow effects.
    pub hsv_interpolation: bool,
}

impl ColorGradient {
    /// Total number of color samples across all segments.
    #[must_use]
    pub fn total_steps(&self) -> usize {
        self.steps.iter().sum()
    }

    /// Samples the gradient at the given flattened index.
    ///
    /// The index is clamped to `[0, total_steps - 1)`. Returns the first stop's color
    /// when the gradient is degenerate (no segments).
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn sample(&self, index: usize) -> Color {
        if self.steps.is_empty() || self.colors.len() < 2 {
            return self.colors.first().copied().unwrap_or(Color::WHITE);
        }

        let total = self.total_steps();
        let clamped = index.min(total.saturating_sub(1));

        let mut remaining = clamped;
        let mut seg = 0;
        while seg + 1 < self.steps.len() && remaining >= self.steps[seg] {
            remaining -= self.steps[seg];
            seg += 1;
        }

        let segment_steps = self.steps[seg];
        let start = self.colors[seg];
        let end = self.colors[seg + 1];
        let t = if segment_steps <= 1 {
            0.0
        } else {
            remaining as f32 / (segment_steps - 1) as f32
        };

        if self.hsv_interpolation {
            let start_hsl: Hsla = start.into();
            let end_hsl: Hsla = end.into();

            let h = (end_hsl.hue - start_hsl.hue).mul_add(t, start_hsl.hue);
            let s = (end_hsl.saturation - start_hsl.saturation).mul_add(t, start_hsl.saturation);
            let l = (end_hsl.lightness - start_hsl.lightness).mul_add(t, start_hsl.lightness);

            Color::hsl(h, s, l)
        } else {
            start.mix(&end, t)
        }
    }
}

impl Default for ColorGradient {
    fn default() -> Self {
        Self {
            colors: vec![Color::WHITE, Color::BLACK],
            steps: vec![10],
            index: 0,
            hsv_interpolation: false,
        }
    }
}

/// Texture-based color source for particles.
///
/// Stores the asset path and a handle to a texture image. Each particle's color
/// is sampled directly from the [`Image`] asset at its world position, tiling
/// across the map.
#[derive(Clone, Debug, Reflect, Serialize, Deserialize)]
#[type_path = "bfs_color::particle"]
pub struct TextureSource {
    /// Asset path to the texture image (e.g. `"textures/wood.png"`).
    pub path: String,
    #[serde(skip)]
    #[reflect(ignore)]
    pub(crate) handle: Option<Handle<Image>>,
}

impl PartialEq for TextureSource {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for TextureSource {}

/// Color source configuration for particles
#[derive(Clone, PartialEq, Debug, Reflect, Serialize, Deserialize)]
#[type_path = "bfs_color::particle"]
pub enum ColorSource {
    /// Use a palette of discrete colors
    Palette(Palette),
    /// Use a gradient between colors
    Gradient(ColorGradient),
    /// Use a texture image, tiling by world position
    Texture(TextureSource),
}

impl Default for ColorSource {
    fn default() -> Self {
        Self::Palette(Palette::default())
    }
}

/// Stores the current rendered color of a particle.
#[derive(Component, Clone, Default, PartialEq, Debug, Reflect, Serialize, Deserialize)]
pub struct ParticleColor(pub Color);
