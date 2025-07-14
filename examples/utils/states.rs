use bevy::prelude::*;

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, exit_on_key)
            .init_state::<AppState>();
    }
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Canvas,
    Ui,
}

fn exit_on_key(keyboard_input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
    }
}
