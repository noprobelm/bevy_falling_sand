use bevy::prelude::{Reflect, GizmoConfigGroup, Resource};

#[derive(Default, Resource)]
pub struct DebugParticles;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct DebugGizmos;
