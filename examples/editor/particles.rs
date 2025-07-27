use bevy::prelude::*;
use bevy_falling_sand::prelude::*;

use crate::app_state::InitializationState;

pub(crate) struct ParticleSetupPlugin;

impl Plugin for ParticleSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            check_particles_defs_initialized.run_if(in_state(InitializationState::Initializing)),
        );
    }
}

#[derive(Clone, Resource, Debug, Reflect)]
#[reflect(Resource)]
pub struct SelectedParticle(pub Particle);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let particles_handle: Handle<ParticleDefinitionsAsset> =
        asset_server.load("particles/particles.ron");
    commands.spawn(ParticleDefinitionsHandle::new(particles_handle));
}

fn check_particles_defs_initialized(
    mut commands: Commands,
    mut next_state: ResMut<NextState<InitializationState>>,
    map: Res<ParticleTypeMap>,
) {
    if map.is_empty() {
        return;
    }

    if map.contains("Dirt Wall") {
        commands.insert_resource(SelectedParticle(Particle::new("Dirt Wall")));
    } else {
        let first_name = map
            .keys()
            .next()
            .expect("ParticleTypeMap is not empty, so this should never fail");
        let static_name: &'static str = Box::leak(first_name.to_string().into_boxed_str());
        commands.insert_resource(SelectedParticle(Particle::new(static_name)));
    }
    next_state.set(InitializationState::Finished);
}
