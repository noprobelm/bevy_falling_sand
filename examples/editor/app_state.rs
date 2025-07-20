use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContextPass, EguiContexts};

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_systems(EguiContextPass, detect_ui_interaction)
            .add_systems(OnEnter(AppState::Canvas), hide_cursor)
            .add_systems(OnEnter(AppState::Ui), show_cursor);
    }
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Canvas,
    Ui,
}

fn detect_ui_interaction(
    mut contexts: EguiContexts,
    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let ctx = contexts.ctx_mut();

    let is_over_area = ctx.is_pointer_over_area();

    match current_state.get() {
        AppState::Ui => {
            if !is_over_area {
                next_state.set(AppState::Canvas);
            }
        }
        AppState::Canvas => {
            if is_over_area {
                next_state.set(AppState::Ui);
            }
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
