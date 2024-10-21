use bevy::prelude::{Entity, Event, App, Plugin};

pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ResetParticleColorEvent>()
            .add_event::<ResetRandomizesColorEvent>()
            .add_event::<ResetFlowsColorEvent>();
    }
}
