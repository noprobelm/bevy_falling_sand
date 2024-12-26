use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy_falling_sand::scenes::{LoadSceneEvent, SaveSceneEvent};
use bevy_egui::egui;

/// Scene plugin
pub(super) struct ScenesPlugin;

impl bevy::prelude::Plugin for ScenesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<SceneSelectionDialog>()
            .init_resource::<ParticleSceneFilePath>();
    }
}

/// Manages scene selection dialog boxes.
#[derive(Resource, Default)]
pub struct SceneSelectionDialog {
    pub show_save_dialog: bool,
    pub show_load_dialog: bool,
    pub save_input_text: String,
    pub load_input_text: String,
}

#[derive(Resource)]
pub struct ParticleSceneFilePath(pub PathBuf);

impl Default for ParticleSceneFilePath {
    fn default() -> ParticleSceneFilePath {
        let mut example_path = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
        example_path.push("examples/assets/scenes/hourglass.ron");

        ParticleSceneFilePath(example_path)
    }
}

/// UI for saving/loading particle scenes.
pub struct SceneManagementUI;

impl SceneManagementUI {
    /// Renders the scene management UI
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        dialog_state: &mut ResMut<SceneSelectionDialog>,
        scene_path: &mut ResMut<ParticleSceneFilePath>,
        ev_save_scene: &mut EventWriter<SaveSceneEvent>,
        ev_load_scene: &mut EventWriter<LoadSceneEvent>,
    ) {
        ui.horizontal_wrapped(|ui| {
            if ui.button("SAVE SCENE").clicked() {
                dialog_state.show_save_dialog = true;
            }

            if ui.button("LOAD SCENE").clicked() {
                dialog_state.show_load_dialog = true;
            }
        });

        if dialog_state.show_save_dialog {
            egui::Window::new("Save Scene")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Enter a name to save the current scene:");
                    ui.text_edit_singleline(&mut dialog_state.save_input_text);
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Save").clicked() {
                            let mut file_name = dialog_state.save_input_text.clone();
                            if !file_name.ends_with(".ron") {
                                file_name.push_str(".ron");
                            }
                            scene_path.0.set_file_name(file_name);
                            ev_save_scene.send(SaveSceneEvent(scene_path.0.clone()));
                            dialog_state.show_save_dialog = false; // Close after saving
                        }
                        if ui.button("Cancel").clicked() {
                            dialog_state.show_save_dialog = false;
                        }
                    });
                });
        }

        if dialog_state.show_load_dialog {
            // Fetch all `.ron` files in the directory
            let ron_files: Vec<String> = std::fs::read_dir(&scene_path.0.parent().unwrap())
                .unwrap()
                .filter_map(|entry| {
                    let path = entry.unwrap().path();
                    if path.extension() == Some(std::ffi::OsStr::new("ron")) {
                        path.file_name()
                            .and_then(|name| name.to_str().map(String::from))
                    } else {
                        None
                    }
                })
                .collect();

            egui::Window::new("Load Scene")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Select the scene to load:");

                    egui::ComboBox::from_label("Available Scenes")
                        .selected_text(dialog_state.load_input_text.clone())
                        .show_ui(ui, |ui| {
                            for file_name in &ron_files {
                                let display_name =
                                    file_name.strip_suffix(".ron").unwrap_or(file_name);
                                if ui
                                    .selectable_value(
                                        &mut dialog_state.load_input_text,
                                        file_name.clone(),
                                        display_name,
                                    )
                                    .changed()
                                {
                                    // Automatically update the scene path when a file is selected
                                    scene_path.0.set_file_name(file_name.clone());
                                }
                            }
                        });

                    if ui.button("Load").clicked() {
                        ev_load_scene.send(LoadSceneEvent(scene_path.0.clone()));
                        dialog_state.show_load_dialog = false; // Close after loading
                    }
                    if ui.button("Cancel").clicked() {
                        dialog_state.show_load_dialog = false;
                    }
                });
        }
    }
}
