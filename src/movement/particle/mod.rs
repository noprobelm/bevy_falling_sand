mod components;
mod despawn;

use bevy::prelude::*;

pub use components::*;
pub use despawn::*;

use components::ComponentsPlugin;
use despawn::DespawnPlugin;

pub(super) struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ComponentsPlugin, DespawnPlugin))
            .add_systems(PreUpdate, sync_air_resistance);
    }
}

fn sync_air_resistance(mut query: Query<(&Movement, &mut AirResistance), Changed<Movement>>) {
    for (movement, mut air_resistance) in &mut query {
        let target_len = movement.neighbor_groups.len();
        if air_resistance.len() != target_len {
            air_resistance.resize(target_len);
        }
    }
}
