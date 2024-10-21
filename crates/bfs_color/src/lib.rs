//! This crate provides coloring for particles in the simulation.

mod events;

use std::ops::RangeBounds;
use serde::{Deserialize, Serialize};
use bevy::prelude::*;
use bevy_turborand::GlobalRng;

use bevy_turborand::{prelude::RngComponent, DelegatedRng, TurboRand};
use bfs_core::{Particle, ParticleSimulationSet, ParticleType};

pub use events::*;

pub struct FallingSandColorPlugin;

impl Plugin for FallingSandColorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EventsPlugin);
        app.register_type::<ColorRng>()
            .register_type::<ParticleColor>()
            .register_type::<FlowsColor>()
            .register_type::<RandomizesColor>();
        app.add_systems(
            Update,
            (
                color_particles,
                color_flowing_particles,
                color_randomizing_particles,
            )
                .in_set(ParticleSimulationSet)
        );
        app.observe(on_reset_particle_color)
            .observe(on_reset_randomizes_color)
            .observe(on_reset_flows_color);
    }
}

/// RNG to use when dealing with any entity that needs random coloring behaviors.
#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct ColorRng(pub RngComponent);

impl ColorRng {
    /// Shuffles a given slice.
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        self.0.shuffle(slice);
    }

    /// Returns a boolean value based on a rate. rate represents the chance to return a true value, with 0.0 being no
    /// chance and 1.0 will always return true.
    pub fn chance(&mut self, rate: f64) -> bool {
        self.0.chance(rate)
    }

    /// Samples a random item from a slice of values.
    pub fn sample<'a, T>(&mut self, list: &'a [T]) -> Option<&'a T> {
        self.0.sample(&list)
    }

    /// Returns a usize value for stable indexing across different word size platforms.
    pub fn index(&mut self, bound: impl RangeBounds<usize>) -> usize {
        self.0.index(bound)
    }
}

/// Provides a range of possible colors for a particle. Child particles will access
/// this component from their parent particle when spawning to select a color for themselves at
/// random.
#[derive(Clone, PartialEq, Debug, Default, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ParticleColor {
    /// The index to reference when changing through colors sequentially.
    color_index: usize,
    /// The current color.
    pub selected: Color,
    /// The possible range of colors.
    pub palette: Vec<Color>,
}

impl ParticleColor {
    /// Creates a new ParticleColors component with the specified colors.
    pub fn new(selected: Color, palette: Vec<Color>) -> ParticleColor {
        ParticleColor {
            color_index: 0,
            selected,
            palette,
        }
    }

    /// Select a random color from the colors sequence.
    pub fn new_with_random<R: TurboRand>(&self, rng: &mut R) -> ParticleColor {
        let color_index = rng.index(0..self.palette.len());
        ParticleColor {
            color_index,
            selected: *self.palette.get(color_index).unwrap(),
            palette: self.palette.clone(),
        }
    }

    /// Randomize the current color.
    pub fn randomize(&mut self, rng: &mut ColorRng) {
        self.color_index = rng.index(0..self.palette.len());
        self.selected = *self.palette.get(self.color_index).unwrap();
    }

    /// Change to the next color in the palette
    pub fn set_next(&mut self) {
        if self.palette.len() - 1 == self.color_index {
            self.color_index = 0;
        } else {
            self.color_index += 1;
        }
        self.selected = *self.palette.get(self.color_index).unwrap();
    }
}

/// Component for particles that randomly change colors from its palette.
#[derive(Clone, PartialEq, Debug, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct RandomizesColor {
    /// The chance a particle's color will change.
    pub rate: f64,
}

impl RandomizesColor {
    /// Creates a new RandomizesColors
    pub fn new(chance: f64) -> RandomizesColor {
        RandomizesColor { rate: chance }
    }
}

/// Component for particlce whose colors flows sequientally through its palette.
#[derive(Clone, PartialEq, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct FlowsColor {
    /// The chance a particle's color will change.
    pub rate: f64,
}

impl FlowsColor {
    /// Creates a new RandomizesColors
    pub fn new(chance: f64) -> FlowsColor {
        FlowsColor { rate: chance }
    }
}

/// Colors newly added or changed particles
pub fn color_particles(
    mut particle_query: Query<(&mut Sprite, &ParticleColor), Changed<ParticleColor>>,
) {
    particle_query.iter_mut().for_each(|(mut sprite, color)| {
        sprite.color = color.selected;
    });
}

/// Changes the color of particles with the ChangesColor component
pub fn color_flowing_particles(
    mut particles_query: Query<(&mut ParticleColor, &mut ColorRng, &FlowsColor), With<Particle>>,
) {
    particles_query
        .iter_mut()
        .for_each(|(mut particle_color, mut rng, flows_color)| {
            if rng.chance(flows_color.rate) {
                particle_color.set_next();
            }
        })
}

/// Randomizes the color of particles with the ChangesColor component
pub fn color_randomizing_particles(
    mut particles_query: Query<
        (&mut ParticleColor, &mut ColorRng, &RandomizesColor),
        With<Particle>,
    >,
) {
    particles_query
        .iter_mut()
        .for_each(|(mut particle_color, mut rng, randomizes_color)| {
            if rng.chance(randomizes_color.rate) {
                particle_color.randomize(&mut rng);
            }
        })
}

/// Observer for resetting a particle's Velocity information to its parent's.
pub fn on_reset_particle_color(
    trigger: Trigger<ResetParticleColorEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&ParticleColor>, With<ParticleType>>,
    mut rng: ResMut<GlobalRng>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        let rng = rng.get_mut();
        if let Some(particle_color) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(particle_color.new_with_random(rng));
        } else {
            commands
                .entity(trigger.event().entity)
                .remove::<ParticleColor>();
        }
    }
}

/// Observer for resetting a particle's RandomizesColor information to its parent's.
pub fn on_reset_randomizes_color(
    trigger: Trigger<ResetRandomizesColorEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&RandomizesColor>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(color) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(color.clone());
        } else {
            commands
                .entity(trigger.event().entity)
                .remove::<RandomizesColor>();
        }
    }
}

/// Observer for resetting a particle's FlowsColor information to its parent's.
pub fn on_reset_flows_color(
    trigger: Trigger<ResetFlowsColorEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&FlowsColor>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(color) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(color.clone());
        } else {
            commands
                .entity(trigger.event().entity)
                .remove::<FlowsColor>();
        }
    }
}
