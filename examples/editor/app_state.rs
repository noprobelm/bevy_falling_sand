use crate::ui::particle_search::ParticleSearchState;
use crate::ui::ConsoleState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContextPass, EguiContexts};
use bevy_falling_sand::prelude::{DebugDirtyRects, DebugParticleMap};

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_state::<UiState>()
            .init_state::<InitializationState>()
            .add_systems(Startup, remove_debug_overlays)
            .add_systems(EguiContextPass, (detect_ui_interaction, manage_ui_states))
            .add_systems(OnEnter(AppState::Canvas), hide_cursor)
            .add_systems(OnEnter(AppState::Ui), show_cursor);
    }
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Canvas,
    #[default]
    Ui,
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum UiState {
    #[default]
    Normal,
    Console,
    ParticleSearch,
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
) {
    let ctx = contexts.ctx_mut();

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
}

fn manage_ui_states(
    current_ui_state: Res<State<UiState>>,
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

    if current_ui_state.get() != &desired_state {
        next_ui_state.set(desired_state);
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
