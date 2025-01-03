use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng, TurboRand};
use serde::{Deserialize, Serialize};

use super::ColorRng;
use bfs_core::{Particle, ParticleRegistrationEvent, ParticleType};

pub struct ParticleDefinitionsPlugin;

impl Plugin for ParticleDefinitionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_particle_registration);
        app.add_event::<ResetParticleColorEvent>();
        app.register_type::<ColorRng>()
            .register_type::<ParticleColor>()
            .register_type::<FlowsColor>()
            .register_type::<RandomizesColor>();
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

#[derive(
    Clone, Hash, Debug, Default, Eq, PartialEq, PartialOrd, Event, Reflect, Serialize, Deserialize,
)]
pub struct ResetParticleColorEvent {
    pub entities: Vec<Entity>,
}

#[derive(Copy, Clone, PartialEq, Debug, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct FlowsColorBlueprint(pub FlowsColor);

fn handle_particle_components(
    commands: &mut Commands,
    rng: &mut ResMut<GlobalRng>,
    parent_query: &Query<
        (
            Option<&ParticleColorBlueprint>,
            Option<&FlowsColorBlueprint>,
            Option<&RandomizesColorBlueprint>,
        ),
        With<ParticleType>,
    >,
    particle_query: &Query<&Parent, With<Particle>>,
    entities: &Vec<Entity>,
) {
    entities.iter().for_each(|entity| {
        if let Ok(parent) = particle_query.get(*entity) {
            commands.entity(*entity).insert(ColorRng::default());
            if let Ok((particle_color, flows_color, randomizes_color)) =
                parent_query.get(parent.get())
            {
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
                    commands.entity(*entity).remove::<ParticleColor>();
                }
                if let Some(flows_color) = flows_color {
                    commands.entity(*entity).insert(flows_color.0.clone());
                } else {
                    commands.entity(*entity).remove::<FlowsColor>();
                }
                if let Some(randomizes_color) = randomizes_color {
                    commands.entity(*entity).insert(randomizes_color.0.clone());
                } else {
                    commands.entity(*entity).remove::<RandomizesColor>();
                }
            }
        }
    });
}

fn handle_particle_registration(
    mut commands: Commands,
    mut rng: ResMut<GlobalRng>,
    parent_query: Query<
        (
            Option<&ParticleColorBlueprint>,
            Option<&FlowsColorBlueprint>,
            Option<&RandomizesColorBlueprint>,
        ),
        With<ParticleType>,
    >,
    particle_query: Query<&Parent, With<Particle>>,
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
