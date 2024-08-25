mod map;
mod scenes;
mod particle_deserializer;

pub use map::*;
pub use scenes::*;
pub use particle_deserializer::*;

pub(super) struct ParticleEventsPlugin;

impl bevy::prelude::Plugin for ParticleEventsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<ClearChunkMapEvent>()
            .add_event::<SaveSceneEvent>()
            .add_event::<LoadSceneEvent>()
            .add_event::<DeserializeParticleTypesEvent>();
    }
}
