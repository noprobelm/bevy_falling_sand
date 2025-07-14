use bevy::prelude::*;
use bevy_falling_sand::prelude::*;
use super::brush::{BrushState, BrushType, SelectedBrushParticle};

#[derive(Component)]
pub struct TotalParticleCountText;

#[derive(Component)]
pub struct BrushStateText;

#[derive(Component)]
pub struct SelectedParticleText;

#[derive(Component)]
pub struct BrushTypeText;

#[derive(Component)]
pub struct MovementSourceText;

pub struct StatusUIPlugin;

impl Plugin for StatusUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_total_particle_count_text.run_if(resource_exists::<TotalParticleCount>),
                update_brush_state_text,
                update_selected_particle_text,
                update_brush_type_text,
                update_movement_source_text,
            ),
        );
    }
}

// Component spawning will be done directly in basic.rs
// This module just provides the plugin and component definitions

fn update_total_particle_count_text(
    debug_total_particle_count: Res<TotalParticleCount>,
    mut total_particle_count_text: Query<&mut Text, With<TotalParticleCountText>>,
) -> Result {
    let new_text = format!("Total Particles: {:?}", debug_total_particle_count.0);
    for mut total_particle_count_text in total_particle_count_text.iter_mut() {
        (**total_particle_count_text).clone_from(&new_text);
    }
    Ok(())
}

fn update_brush_state_text(
    brush_state: Res<State<BrushState>>,
    mut brush_state_text: Query<&mut Text, With<BrushStateText>>,
) {
    let state_text = match brush_state.get() {
        BrushState::Spawn => "Brush Mode: Spawn",
        BrushState::Despawn => "Brush Mode: Despawn",
    };

    for mut text in brush_state_text.iter_mut() {
        **text = state_text.to_string();
    }
}

fn update_selected_particle_text(
    selected_particle: Res<SelectedBrushParticle>,
    mut selected_particle_text: Query<&mut Text, With<SelectedParticleText>>,
) {
    let particle_text = format!("Selected Particle: {}", selected_particle.0.name);

    for mut text in selected_particle_text.iter_mut() {
        **text = particle_text.clone();
    }
}

fn update_brush_type_text(
    brush_type: Res<State<BrushType>>,
    mut brush_type_text: Query<&mut Text, With<BrushTypeText>>,
) {
    let type_text = format!("Brush Type: {:?}", brush_type.get());

    for mut text in brush_type_text.iter_mut() {
        **text = type_text.clone();
    }
}

fn update_movement_source_text(
    movement_source: Res<State<MovementSource>>,
    mut movement_source_text: Query<&mut Text, With<MovementSourceText>>,
) {
    let source_text = format!("Movement Source: {:?}", movement_source.get());

    for mut text in movement_source_text.iter_mut() {
        **text = source_text.clone();
    }
}