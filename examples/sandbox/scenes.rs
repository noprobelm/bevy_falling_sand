use bevy::prelude::*;
use bevy_egui::egui;
use bevy_falling_sand::prelude::{LoadSceneEvent, SaveSceneEvent};
use crate::ui::file_browser::{FileBrowser, FileBrowserState};
use std::path::PathBuf;

pub(super) struct ScenesPlugin;

impl bevy::prelude::Plugin for ScenesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<SceneSelectionDialog>()
            .init_resource::<SceneFileBrowserState>();
    }
}

#[derive(Resource, Default)]
pub struct SceneSelectionDialog {
    pub last_error: Option<String>,
    pub last_success: Option<String>,
}

// Keep a separate browser state for scenes
#[derive(Resource)]
pub struct SceneFileBrowserState(pub FileBrowserState);

impl Default for SceneFileBrowserState {
    fn default() -> Self {
        Self(FileBrowserState::new("assets/scenes", "ron", "Scene Files"))
    }
}

pub struct SceneManagementUI;

impl SceneManagementUI {
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        dialog_state: &mut ResMut<SceneSelectionDialog>,
        scene_browser_state: &mut ResMut<SceneFileBrowserState>,
        ev_save_scene: &mut EventWriter<SaveSceneEvent>,
        ev_load_scene: &mut EventWriter<LoadSceneEvent>,
    ) {
        ui.separator();
        ui.label("Scene Management");
        
        ui.horizontal_wrapped(|ui| {
            if ui.button("SAVE SCENE").clicked() {
                scene_browser_state.0.show_save("Save Scene");
            }
            
            if ui.button("LOAD SCENE").clicked() {
                scene_browser_state.0.show_load("Load Scene");
            }
        });
        
        // Show status messages
        if let Some(ref error) = dialog_state.last_error {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
        }
        
        if let Some(ref success) = dialog_state.last_success {
            ui.colored_label(egui::Color32::GREEN, success);
        }
        
        // Render file browser dialogs
        let file_browser = FileBrowser;
        
        file_browser.render_save_dialog(
            ui,
            &mut scene_browser_state.0,
            |path| {
                ev_save_scene.write(SaveSceneEvent(path));
            },
        );
        
        file_browser.render_load_dialog(
            ui,
            &mut scene_browser_state.0,
            |path| {
                ev_load_scene.write(LoadSceneEvent(path));
            },
        );
    }
}