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
        app.add_observer(on_reset_particle_color)
            .add_observer(on_reset_flows_color)
            .add_observer(on_reset_randomizes_color);
    }
}

#[derive(Clone, PartialEq, Debug, Default, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ParticleColor {
    color_index: usize,
    pub selected: Color,
    pub palette: Vec<Color>,
}

impl ParticleColor {
    pub fn new(selected: Color, palette: Vec<Color>) -> ParticleColor {
        ParticleColor {
            color_index: 0,
            selected,
            palette,
        }
    }

    pub fn new_with_random<R: TurboRand>(&self, rng: &mut R) -> ParticleColor {
        let color_index = rng.index(0..self.palette.len());
        ParticleColor {
            color_index,
            selected: *self.palette.get(color_index).unwrap(),
            palette: self.palette.clone(),
        }
    }

    pub fn randomize(&mut self, rng: &mut ColorRng) {
        self.color_index = rng.index(0..self.palette.len());
        self.selected = *self.palette.get(self.color_index).unwrap();
    }

    pub fn set_next(&mut self) {
        if self.palette.len() - 1 == self.color_index {
            self.color_index = 0;
        } else {
            self.color_index += 1;
        }
        self.selected = *self.palette.get(self.color_index).unwrap();
    }
}

#[derive(Clone, PartialEq, Debug, Default, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ParticleColorBlueprint(pub ParticleColor);

#[derive(Copy, Clone, PartialEq, Debug, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct RandomizesColor {
    pub rate: f64,
}

#[derive(Copy, Clone, PartialEq, Debug, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct RandomizesColorBlueprint(pub RandomizesColor);

impl RandomizesColor {
    pub fn new(chance: f64) -> RandomizesColor {
        RandomizesColor { rate: chance }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct FlowsColor {
    pub rate: f64,
}

impl FlowsColor {
    pub fn new(chance: f64) -> FlowsColor {
        FlowsColor { rate: chance }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct FlowsColorBlueprint(pub FlowsColor);

#[derive(Event)]
pub struct ResetParticleColorEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct ResetRandomizesColorEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct ResetFlowsColorEvent {
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
