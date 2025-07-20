use bevy::prelude::*;

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_resource::<UiInteractionState>()
            .add_systems(Update, detect_ui_interaction);
    }
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Canvas,
    Ui,
}

#[derive(Resource, Default)]
pub struct UiInteractionState {
    pub mouse_over_ui: bool,
}

fn detect_ui_interaction(
    ui_state: Res<UiInteractionState>,
    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    match (current_state.get(), ui_state.mouse_over_ui) {
        (AppState::Canvas, true) => next_state.set(AppState::Ui),
        (AppState::Ui, false) => next_state.set(AppState::Canvas),
        _ => {}
    }
}

