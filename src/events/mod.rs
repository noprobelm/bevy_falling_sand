mod map;
mod scenes;

pub use map::*;
pub use scenes::*;

pub(super) struct ParticleEventsPlugin;

impl bevy::prelude::Plugin for ParticleEventsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<ClearChunkMapEvent>()
            .add_event::<SaveSceneEvent>()
            .add_event::<ChangeParticleEvent>()
            .add_event::<LoadSceneEvent>();
    }
}
