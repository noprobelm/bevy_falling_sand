use bevy::prelude::*;

use crate::cursor::CursorPosition;

pub struct OverlaysPlugin;

impl Plugin for OverlaysPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<OverlaysGizmos>();
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct OverlaysGizmos;

#[derive(Resource, Clone, Default, Debug)]
pub struct DrawCursorGuide;

pub fn draw_cursor_guide(cursor_position: Res<CursorPosition>, mut gizmos: Gizmos<OverlaysGizmos>) {
    gizmos.grid_2d(
        Isometry2d::from_translation(cursor_position.current.round() + 0.5),
        UVec2::splat(32),
        Vec2::splat(1.),
        LinearRgba::gray(0.10),
    );
}
