use crate::ui::ConsoleState;
use crate::ui::particle_search::ParticleSearchState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};
use bevy_falling_sand::prelude::*;

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, AppStateDetectionSet)
            .init_state::<AppState>()
            .add_sub_state::<UiState>()
            .add_sub_state::<CanvasState>()
            .init_state::<InitializationState>()
            .add_systems(Startup, remove_debug_overlays)
            .add_systems(
                EguiPrimaryContextPass,
                (
                    detect_ui_interaction,
                    manage_ui_states
                        .run_if(in_state(AppState::Ui))
                        .after(detect_ui_interaction),
                    manage_canvas_states
                        .run_if(in_state(AppState::Canvas))
                        .after(detect_ui_interaction),
                ),
            )
            .add_systems(OnEnter(AppState::Canvas), hide_cursor)
            .add_systems(OnEnter(AppState::Ui), show_cursor);
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppStateDetectionSet;

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Canvas,
    #[default]
    Ui,
}

#[derive(SubStates, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
#[source(AppState = AppState::Ui)]
pub enum UiState {
    #[default]
    Normal,
    Console,
    ParticleSearch,
}

#[derive(SubStates, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
#[source(AppState = AppState::Canvas)]
pub enum CanvasState {
    #[default]
    Interact,
    Edit,
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum InitializationState {
    #[default]
    Initializing,
    Finished,
}

fn detect_ui_interaction(
    mut contexts: EguiContexts,
    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    particle_search_state: Option<Res<ParticleSearchState>>,
    console_state: Option<Res<ConsoleState>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    let is_over_area = ctx.is_pointer_over_area();
    let search_is_active = particle_search_state.map_or(false, |state| state.active);
    let console_is_expanded = console_state.map_or(false, |state| state.expanded);
    let has_keyboard_focus = ctx.wants_keyboard_input();

    let should_be_ui = is_over_area
        || (search_is_active && has_keyboard_focus)
        || (console_is_expanded && has_keyboard_focus);

    match current_state.get() {
        AppState::Ui => {
            if !should_be_ui {
                next_state.set(AppState::Canvas);
            }
        }
        AppState::Canvas => {
            if should_be_ui {
                next_state.set(AppState::Ui);
            }
        }
    }
    Ok(())
}

fn manage_ui_states(
    current_ui_state: Option<Res<State<UiState>>>,
    mut next_ui_state: ResMut<NextState<UiState>>,
    particle_search_state: Option<Res<ParticleSearchState>>,
    console_state: Option<Res<ConsoleState>>,
) {
    let search_is_active = particle_search_state.map_or(false, |state| state.active);
    let console_is_expanded = console_state.map_or(false, |state| state.expanded);

    let desired_state = if search_is_active {
        UiState::ParticleSearch
    } else if console_is_expanded {
        UiState::Console
    } else {
        UiState::Normal
    };

    if let Some(current_state) = current_ui_state {
        if current_state.get() != &desired_state {
            next_ui_state.set(desired_state);
        }
    }
}

fn manage_canvas_states(
    current_canvas_state: Option<Res<State<CanvasState>>>,
    mut next_canvas_state: ResMut<NextState<CanvasState>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if let Some(current_state) = current_canvas_state {
        if keys.pressed(KeyCode::AltLeft) && current_state.get() == &CanvasState::Interact {
            next_canvas_state.set(CanvasState::Edit);
        } else if !keys.pressed(KeyCode::AltLeft) && current_state.get() == &CanvasState::Edit {
            next_canvas_state.set(CanvasState::Interact);
        }
    }
}

pub fn hide_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.single_mut() {
        window.cursor_options.visible = false;
    }
}

pub fn show_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.single_mut() {
        window.cursor_options.visible = true;
    }
}

fn remove_debug_overlays(mut commands: Commands) {
    commands.remove_resource::<DebugParticleMap>();
    commands.remove_resource::<DebugDirtyRects>();
}

pub fn toggle_resource<T: Resource + Default>(mut commands: Commands, resource: Option<Res<T>>) {
    if resource.is_some() {
        commands.remove_resource::<T>();
    } else {
        commands.init_resource::<T>();
    }
}

pub fn toggle_particle_movement_logic(
    particle_movement_state_current: Res<State<MovementSource>>,
    mut particle_movement_state_next: ResMut<NextState<MovementSource>>,
) {
    match particle_movement_state_current.get() {
        MovementSource::Chunks => {
            particle_movement_state_next.set(MovementSource::Particles);
        }
        MovementSource::Particles => {
            particle_movement_state_next.set(MovementSource::Chunks);
        }
    }
}

pub fn toggle_simulation_run(
    mut commands: Commands,
    simulation_pause: Option<Res<ParticleSimulationRun>>,
    app_state: Res<State<AppState>>,
    mut time: ResMut<Time<Physics>>,
) {
    if app_state.get() == &AppState::Canvas {
        if simulation_pause.is_some() {
            commands.remove_resource::<ParticleSimulationRun>();
        } else {
            commands.init_resource::<ParticleSimulationRun>();
        }
        if time.is_paused() {
            time.unpause();
        } else {
            time.pause();
        }
    }
}

pub fn step_simulation(mut evw_simulation_step: EventWriter<SimulationStepEvent>) {
    evw_simulation_step.write(SimulationStepEvent);
}
