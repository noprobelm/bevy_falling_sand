//! DebugGizmos group.
use bevy::prelude::*;

/// Indicates a gizmo used for debugging purposes
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct DebugGizmos;
