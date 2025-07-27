use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_falling_sand::prelude::*;

pub(super) struct QuickActionsPlugin;

impl Plugin for QuickActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                toggle_particle_map_overlay.run_if(input_just_pressed(KeyCode::F1)),
                toggle_dirty_rects_overlay.run_if(input_just_pressed(KeyCode::F2)),
                toggle_particle_movement_logic.run_if(input_just_pressed(KeyCode::F3)),
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

fn toggle_particle_movement_logic(
    movement_state_current: Res<State<MovementSource>>,
    mut movement_state_next: ResMut<NextState<MovementSource>>,
) {
    match movement_state_current.get() {
        MovementSource::Chunks => {
            movement_state_next.set(MovementSource::Particles);
        }
        MovementSource::Particles => {
            movement_state_next.set(MovementSource::Chunks);
        }
    }
}
