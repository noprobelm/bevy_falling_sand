use bevy::{input::common_conditions::input_just_pressed, prelude::*};

/// Standalone instructions setup that doesn't require the plugin system
/// Useful for examples that don't want to include the full utils ecosystem
pub fn setup_standalone_instructions(
    commands: &mut Commands,
    instructions_text: &str,
    toggle_key: KeyCode,
) -> Entity {
    let panel_id = spawn_instructions_panel(commands, instructions_text);

    // Add a marker component to track which key toggles this panel
    commands
        .entity(panel_id)
        .insert(StandaloneInstructionsToggle(toggle_key));

    panel_id
}

#[derive(Component)]
pub struct StandaloneInstructionsToggle(KeyCode);

/// System function for toggling standalone instructions
/// Add this to your app's Update schedule manually
pub fn toggle_standalone_instructions(
    keys: Res<ButtonInput<KeyCode>>,
    mut instructions_query: Query<
        (&mut Visibility, &StandaloneInstructionsToggle),
        With<InstructionsPanel>,
    >,
) {
    for (mut visibility, toggle) in instructions_query.iter_mut() {
        if keys.just_pressed(toggle.0) {
            *visibility = match *visibility {
                Visibility::Visible => Visibility::Hidden,
                Visibility::Hidden => Visibility::Visible,
                Visibility::Inherited => Visibility::Hidden,
            };
        }
    }
}

#[derive(Component)]
pub struct InstructionsPanel;

pub struct InstructionsPlugin {
    pub toggle_key: KeyCode,
}

impl Default for InstructionsPlugin {
    fn default() -> Self {
        Self {
            toggle_key: KeyCode::KeyH,
        }
    }
}

impl InstructionsPlugin {
    pub fn with_toggle_key(mut self, toggle_key: KeyCode) -> Self {
        self.toggle_key = toggle_key;
        self
    }
}

impl Plugin for InstructionsPlugin {
    fn build(&self, app: &mut App) {
        let toggle_key = self.toggle_key;
        app.add_systems(
            Update,
            toggle_instructions_panel.run_if(input_just_pressed(toggle_key)),
        );
    }
}

pub fn spawn_instructions_panel(commands: &mut Commands, instructions_text: &str) -> Entity {
    commands
        .spawn((
            InstructionsPanel,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::Srgba(Srgba::new(0.0, 0.0, 0.0, 0.8))),
            BorderColor(Color::Srgba(Srgba::new(0.4, 0.4, 0.4, 1.0))),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|parent| {
            let style = TextFont::default();
            parent.spawn((Text::new(instructions_text), style));
        })
        .id()
}

fn toggle_instructions_panel(
    mut instructions_query: Query<&mut Visibility, With<InstructionsPanel>>,
) {
    for mut visibility in instructions_query.iter_mut() {
        *visibility = match *visibility {
            Visibility::Visible => Visibility::Hidden,
            Visibility::Hidden => Visibility::Visible,
            Visibility::Inherited => Visibility::Hidden,
        };
    }
}
