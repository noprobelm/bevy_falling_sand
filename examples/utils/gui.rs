use super::states::AppState;
use bevy::prelude::*;
use bevy_egui::EguiContexts;

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_app_state);
    }
}

pub fn update_app_state(
    mut contexts: EguiContexts,
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) -> Result {
    match app_state.get() {
        AppState::Ui => {
            let ctx = contexts.ctx_mut()?;
            if !ctx.is_pointer_over_area() {
                next_app_state.set(AppState::Canvas);
            }
        }
        AppState::Canvas => {
            let ctx = contexts.ctx_mut()?;
            if ctx.is_pointer_over_area() {
                next_app_state.set(AppState::Ui);
            }
        }
    }
    Ok(())
}
