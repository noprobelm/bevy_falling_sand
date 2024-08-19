//! These modules are related to particle gizmos
use bevy::prelude::AppGizmoBuilder;

mod debug;

pub use debug::*;

pub(super) struct ParticleGizmosPlugin;

impl bevy::prelude::Plugin for ParticleGizmosPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_gizmo_group::<DebugGizmos>();
    }
}
