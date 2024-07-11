use bevy::prelude::{Reflect, GizmoConfigGroup, Resource};

#[derive(Default, Resource)]
pub struct DebugParticlesMode;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct DebugGizmos;
