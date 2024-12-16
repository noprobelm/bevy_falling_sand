//! Defines additional components for particle types to be used as blueprint data when spawning or
//! resetting particles.
//!
//! This module is a standard template that can be followed when extending particle types. Its
//! structure is as follows:
//!   - Defines new components which will be associated with particle types as blueprint information
//!     for child particles.
//!   - Adds events for each new component which manage resetting information for child particles
//!   - Adds observers for each event to specify granular logic through which a particle should have
//!     its information reset. This usually involves referencing the parent `ParticleType`.
//!
//! When a particle should have its information reset (e.g., when spawning or resetting), we can
//! trigger the events defined in this module and communicate with higher level systems that
//! something needs to happen with a given particle.

use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng, TurboRand};
use serde::{Deserialize, Serialize};

use super::ColorRng;
use bfs_core::{Particle, ParticleType};

pub struct ParticleDefinitionsPlugin;

impl Plugin for ParticleDefinitionsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ColorRng>()
            .register_type::<ParticleColor>()
            .register_type::<FlowsColor>()
            .register_type::<RandomizesColor>();
        app.add_event::<ResetParticleColorEvent>()
            .add_event::<ResetRandomizesColorEvent>()
            .add_event::<ResetFlowsColorEvent>();
        app.observe(on_reset_particle_color)
            .observe(on_reset_flows_color)
            .observe(on_reset_randomizes_color);
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

/// The ParticleColor blueprint.
#[derive(Clone, PartialEq, Debug, Default, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ParticleColorBlueprint(pub ParticleColor);

/// Component for particles that randomly change colors from its palette.
#[derive(Copy, Clone, PartialEq, Debug, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct RandomizesColor {
    /// The chance a particle's color will change.
    pub rate: f64,
}

/// The RandomizesColor blueprint.
#[derive(Copy, Clone, PartialEq, Debug, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct RandomizesColorBlueprint(pub RandomizesColor);

impl RandomizesColor {
    /// Creates a new RandomizesColors
    pub fn new(chance: f64) -> RandomizesColor {
        RandomizesColor { rate: chance }
    }
}

/// Component for particlce whose colors flows sequientally through its palette.
#[derive(Copy, Clone, PartialEq, Debug, Component, Reflect, Serialize, Deserialize)]
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

#[derive(Copy, Clone, PartialEq, Debug, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct FlowsColorBlueprint(pub FlowsColor);

/// Triggers a particle to reset its ParticleColor information to its parent's.
#[derive(Event)]
pub struct ResetParticleColorEvent {
    /// The entity to reset data for.
    pub entity: Entity,
}

/// Triggers a particle to reset its RandomizesColor information to its parent's.
#[derive(Event)]
pub struct ResetRandomizesColorEvent {
    /// The entity to reset data for.
    pub entity: Entity,
}

/// Triggers a particle to reset its FlowsColor information to its parent's.
#[derive(Event)]
pub struct ResetFlowsColorEvent {
    /// The entity to reset data for.
    pub entity: Entity,
}

pub fn on_reset_particle_color(
    trigger: Trigger<ResetParticleColorEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&ParticleColorBlueprint>, With<ParticleType>>,
    mut rng: ResMut<GlobalRng>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        let rng = rng.get_mut();
        if let Some(particle_color) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(particle_color.0.new_with_random(rng));
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
    parent_query: Query<Option<&RandomizesColorBlueprint>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(color) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(color.0);
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
    parent_query: Query<Option<&FlowsColorBlueprint>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(color) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(color.0);
        } else {
            commands
                .entity(trigger.event().entity)
                .remove::<FlowsColor>();
        }
    }
}
