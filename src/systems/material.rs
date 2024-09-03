//! Convenience systems for entities with material marker components.
use bevy::prelude::*;

use crate::*;
use crate::components::material::Material;

/// Observer for adding movement priority when a particle is given a new state of matter.
pub fn on_solid_added(
    trigger: Trigger<OnAdd, Solid>,
    mut commands: Commands,
    particle_query: Query<&Solid, With<ParticleType>>,
) {
    let entity = trigger.entity();
    if let Ok(solid) = particle_query.get(entity) {
	commands.entity(entity).insert(solid.into_movement_priority());
    }
}

/// Observer for adding movement priority when a particle is given a new state of matter.
pub fn on_movable_solid_added(
    trigger: Trigger<OnAdd, MovableSolid>,
    mut commands: Commands,
    particle_query: Query<&MovableSolid, With<ParticleType>>,
) {
    let entity = trigger.entity();
    if let Ok(movable_solid) = particle_query.get(entity) {
	commands.entity(entity).insert(movable_solid.into_movement_priority());
    }
}

/// Observer for adding movement priority when a particle is given a new state of matter.
pub fn on_liquid_added(
    trigger: Trigger<OnAdd, Liquid>,
    mut commands: Commands,
    particle_query: Query<&Liquid, With<ParticleType>>,
) {
    let entity = trigger.entity();
    if let Ok(liquid) = particle_query.get(entity) {
	commands.entity(entity).insert(liquid.into_movement_priority());
    }
}

/// Observer for adding movement priority when a particle is given a new state of matter.
pub fn on_gas_added(
    trigger: Trigger<OnAdd, Gas>,
    mut commands: Commands,
    particle_query: Query<&Gas, With<ParticleType>>,
) {
    let entity = trigger.entity();
    if let Ok(gas) = particle_query.get(entity) {
	commands.entity(entity).insert(gas.into_movement_priority());
    }
}

/// Observer for adding movement priority when a particle is given a new state of matter.
pub fn on_wall_added(
    trigger: Trigger<OnAdd, Wall>,
    mut commands: Commands,
    particle_query: Query<&Wall, With<ParticleType>>,
) {
    let entity = trigger.entity();
    if let Ok(gas) = particle_query.get(entity) {
	commands.entity(entity).insert(gas.into_movement_priority());
    }
}
