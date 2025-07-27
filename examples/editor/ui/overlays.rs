use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_falling_sand::prelude::*;

pub(super) struct OverlaysPlugin;

impl Plugin for OverlaysPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                toggle_particle_map_overlay.run_if(input_just_pressed(KeyCode::F1)),
                toggle_dirty_rects_overlay.run_if(input_just_pressed(KeyCode::F2)),
            ),
        );
    }
}

fn toggle_particle_map_overlay(
    mut commands: Commands,
    debug_particle_map: Option<Res<DebugParticleMap>>,
) {
    if debug_particle_map.is_some() {
        commands.remove_resource::<DebugParticleMap>();
    } else {
        commands.init_resource::<DebugParticleMap>();
    }
}

fn toggle_dirty_rects_overlay(
    mut commands: Commands,
    debug_dirty_rects: Option<Res<DebugDirtyRects>>,
) {
    if debug_dirty_rects.is_some() {
        commands.remove_resource::<DebugDirtyRects>();
    } else {
        commands.init_resource::<DebugDirtyRects>();
    }
}
