use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct ParticleDefinitionsPlugin;

impl Plugin for ParticleDefinitionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MutateParticleEvent>()
            .register_type::<Coordinates>()
            .register_type::<Particle>()
            .add_event::<ResetParticleEvent>()
            .add_event::<RemoveParticleEvent>()
            .add_observer(on_reset_particle);
    }
}

#[derive(Component, Clone, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Particle {
    pub name: String,
}

impl Particle {
    pub fn new(name: &str) -> Particle {
        Particle {
            name: name.to_string(),
        }
    }
}

#[derive(Component, Clone, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ParticleBlueprint(pub Particle);

#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect, Serialize, Deserialize,
)]
#[reflect(Component)]
pub struct Coordinates(pub IVec2);

#[derive(Event)]
pub struct MutateParticleEvent {
    pub entity: Entity,
    pub particle: Particle,
}

#[derive(Event)]
pub struct RemoveParticleEvent {
    pub coordinates: IVec2,
    pub despawn: bool,
}

#[derive(Event)]
pub struct ResetParticleEvent {
    pub entity: Entity,
}

pub fn on_reset_particle(
    trigger: Trigger<ResetParticleEvent>,
    mut particle_query: Query<&mut Particle>,
) {
    particle_query
        .get_mut(trigger.event().entity)
        .unwrap()
        .into_inner();
}
