mod map;
mod scenes;

pub use map::*;
pub use scenes::*;

pub(super) struct ParticleEventsPlugin;

impl bevy::prelude::Plugin for ParticleEventsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<ClearChunkMap>()
            .add_event::<SaveSceneEvent>()
            .add_event::<LoadSceneEvent>();
    }
}
