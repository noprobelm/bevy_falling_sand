use bevy::prelude::*;
use bevy_turborand::RngComponent;
use bevy_turborand::{DelegatedRng, GlobalRng, TurboRand};
use serde::{Deserialize, Serialize};

use bfs_core::{
    impl_particle_blueprint, impl_particle_rng, Particle, ParticleComponent,
    ParticleRegistrationEvent, ParticleRng, ParticleSimulationSet, ParticleType,
};

pub(super) struct ParticleDefinitionsPlugin;

impl Plugin for ParticleDefinitionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ResetParticleColorEvent>().add_systems(
            Update,
            handle_particle_registration.before(ParticleSimulationSet),
        );
    }
}

impl_particle_blueprint!(ColorProfileBlueprint, ColorProfile);
impl_particle_blueprint!(ChangesColorBlueprint, ChangesColor);
impl_particle_rng!(ColorRng, RngComponent);

/// Provides rng for coloring particles.
#[derive(Clone, PartialEq, Debug, Default, Component)]
pub struct ColorRng(pub RngComponent);

/// Provides a color profile for particles, which can be used to set the color of particles from a
/// predefined palette.
#[derive(Clone, PartialEq, Debug, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ColorProfile {
    index: usize,
    /// The color of the particle.
    pub color: Color,
    /// The possible colors of the particle.
    pub palette: Vec<Color>,
}

impl ColorProfile {
    /// Initialize a new `ColorProfile` with the first color in the palette.
    #[must_use]
    pub fn new(palette: Vec<Color>) -> Self {
        Self {
            index: 0,
            color: palette[0],
            palette,
        }
    }

    /// Intiialize a new `ColorProfile` with a specific index and palette.
    #[must_use]
    pub fn new_with_selected(index: usize, palette: Vec<Color>) -> Self {
        Self {
            index,
            color: palette[index],
            palette,
        }
    }

    /// Initialize a new `ColorProfile` with a random color from the palette.
    ///
    /// # Panics
    ///
    /// Panics if the palette is empty.
    #[must_use]
    pub fn new_with_random<R: TurboRand>(&self, rng: &mut R) -> Self {
        assert!(
            !self.palette.is_empty(),
            "ColorProfile palette cannot be empty when initializing with random color."
        );
        let color_index = rng.index(0..self.palette.len());
        Self {
            index: color_index,
            color: *self.palette.get(color_index).unwrap(), // safe because of assert
            palette: self.palette.clone(),
        }
    }

    /// Set the particle color to a random color from the palette.
    ///
    /// # Panics
    ///
    /// Panics if the palette is empty.
    pub fn set_random(&mut self, rng: &mut ColorRng) {
        assert!(
            !self.palette.is_empty(),
            "ColorProfile palette cannot be empty setting color to random."
        );
        self.index = rng.index(0..self.palette.len());
        self.color = *self.palette.get(self.index).unwrap();
    }

    /// Set the particle color to the next color in the palette, returning to the start if at the end.
    ///
    /// # Panics
    ///
    /// Panics if the palette is empty.
    pub fn set_next(&mut self) {
        assert!(
            !self.palette.is_empty(),
            "Palette cannot be empty if setting to next color."
        );
        if self.index >= self.palette.len() - 1 {
            self.index = 0;
        } else {
            self.index += 1;
        }
        self.color = self.palette[self.index];
    }
}

impl Default for ColorProfile {
    fn default() -> Self {
        Self::new(vec![Color::srgba(255., 255., 255., 255.)])
    }
}

/// Blueprint for a [`ColorProfile`].
#[derive(Clone, PartialEq, Debug, Default, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ColorProfileBlueprint(pub ColorProfile);

/// Component that allows particles to change color based on an input chance.
#[derive(Copy, Clone, PartialEq, Debug, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ChangesColor {
    /// The chance that a particle will change color when provided to rng.
    pub chance: f64,
}

impl ChangesColor {
    #[must_use]
    /// Initialize a new `ChangesColor` with a specific chance.
    pub const fn new(chance: f64) -> Self {
        Self { chance }
    }
}

/// Blueprint for holding a `ChangesColor`.
#[derive(Copy, Clone, PartialEq, Debug, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ChangesColorBlueprint(pub ChangesColor);

/// Triggers a particle to reset its [`ParticleColor`] to its parent's blueprint data.
#[derive(
    Clone, Hash, Debug, Default, Eq, PartialEq, PartialOrd, Event, Reflect, Serialize, Deserialize,
)]
pub struct ResetParticleColorEvent {
    /// The particle entities to reset color for.
    pub entities: Vec<Entity>,
}

fn handle_particle_components(
    commands: &mut Commands,
    rng: &mut ResMut<GlobalRng>,
    parent_query: &Query<
        (
            Option<&ColorProfileBlueprint>,
            Option<&ChangesColorBlueprint>,
        ),
        With<ParticleType>,
    >,
    particle_query: &Query<&ChildOf, With<Particle>>,
    entities: &[Entity],
) {
    for entity in entities {
        if let Ok(child_of) = particle_query.get(*entity) {
            commands.entity(*entity).insert(ColorRng::default());
            if let Ok((particle_color, flows_color)) = parent_query.get(child_of.parent()) {
                commands.entity(*entity).insert((
                    Sprite {
                        color: Color::srgba(0., 0., 0., 0.),
                        ..default()
                    },
                    ColorRng::default(),
                ));
                if let Some(particle_color) = particle_color {
                    let rng = rng.get_mut();
                    commands
                        .entity(*entity)
                        .insert(particle_color.0.new_with_random(rng));
                } else {
                    commands.entity(*entity).remove::<ColorProfile>();
                }
                if let Some(flows_color) = flows_color {
                    commands.entity(*entity).insert(flows_color.0);
                } else {
                    commands.entity(*entity).remove::<ChangesColor>();
                }
            }
        }
    }
}

fn handle_particle_registration(
    mut commands: Commands,
    mut rng: ResMut<GlobalRng>,
    parent_query: Query<
        (
            Option<&ColorProfileBlueprint>,
            Option<&ChangesColorBlueprint>,
        ),
        With<ParticleType>,
    >,
    particle_query: Query<&ChildOf, With<Particle>>,
    mut ev_particle_registered: EventReader<ParticleRegistrationEvent>,
    mut ev_reset_particle_color: EventReader<ResetParticleColorEvent>,
) {
    ev_particle_registered.read().for_each(|ev| {
        handle_particle_components(
            &mut commands,
            &mut rng,
            &parent_query,
            &particle_query,
            &ev.entities,
        );
    });
    ev_reset_particle_color.read().for_each(|ev| {
        handle_particle_components(
            &mut commands,
            &mut rng,
            &parent_query,
            &particle_query,
            &ev.entities,
        );
    });
}
