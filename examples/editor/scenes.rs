
use bevy::prelude::*;
use bevy_egui::egui;
use bevy_falling_sand::prelude::{LoadSceneAssetEvent, LoadSceneEvent, SaveSceneEvent};
use std::path::PathBuf;

pub(super) struct ScenesPlugin;

impl bevy::prelude::Plugin for ScenesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<SceneSelectionDialog>()
            .init_resource::<ParticleSceneFilePath>()
            .add_systems(Update, handle_scene_dialog_markers);
    }
}

#[derive(Resource, Default)]
pub struct SceneSelectionDialog {
    pub show_load_dialog: bool,
    pub show_save_dialog: bool,
    pub load_input_text: String,
    pub save_input_text: String,
    pub selected_scene: Option<String>,
}

#[derive(Resource)]
pub struct ParticleSceneFilePath {
    pub path: PathBuf,
}

impl Default for ParticleSceneFilePath {
    fn default() -> Self {
        Self {
            path: PathBuf::from("assets/scenes/custom_scene.ron"),
        }
    }
}

pub struct SceneManagementUI;

impl SceneManagementUI {
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        dialog_state: &mut ResMut<SceneSelectionDialog>,
        scene_path: &mut ResMut<ParticleSceneFilePath>,
        ev_save_scene: &mut EventWriter<SaveSceneEvent>,
        ev_load_scene: &mut EventWriter<LoadSceneEvent>,
        ev_load_scene_asset: &mut EventWriter<LoadSceneAssetEvent>,
        asset_server: &Res<AssetServer>,
    ) {
        // Only render dialogs, not buttons - buttons are in the File menu

        if dialog_state.show_save_dialog {
            // Get all scenes from assets/scenes directory for the save dialog too
            let all_scenes: Vec<(String, String)> = std::fs::read_dir("assets/scenes")
                .map(|entries| {
                    entries
                        .filter_map(|entry| {
                            let entry = entry.ok()?;
                            let path = entry.path();
                            if path.extension()? == "ron" {
                                let file_name = path.file_name()?.to_str()?;
                                let display_name = file_name.trim_end_matches(".ron");
                                let full_path = format!("assets/scenes/{}", file_name);
                                Some((display_name.to_string(), full_path))
                            } else {
                                None
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            egui::Window::new("Save Scene")
                .collapsible(false)
                .resizable(true)
                .default_size([400.0, 300.0])
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Scene name:");
                        ui.text_edit_singleline(&mut dialog_state.save_input_text);
                    });
                    
                    ui.separator();
                    ui.label("Existing scenes:");
                    
                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .show(ui, |ui| {
                            for (display_name, _) in &all_scenes {
                                if ui.selectable_label(
                                    dialog_state.selected_scene.as_ref() == Some(display_name),
                                    display_name
                                ).clicked() {
                                    dialog_state.selected_scene = Some(display_name.clone());
                                    dialog_state.save_input_text = display_name.clone();
                                }
                            }
                        });
                    
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            if !dialog_state.save_input_text.is_empty() {
                                let mut path = PathBuf::from("assets/scenes");
                                path.push(&dialog_state.save_input_text);
                                path.set_extension("ron");
                                ev_save_scene.write(SaveSceneEvent(path));
                            }
                            dialog_state.show_save_dialog = false;
                            dialog_state.selected_scene = None;
                        }
                        if ui.button("Cancel").clicked() {
                            dialog_state.show_save_dialog = false;
                            dialog_state.selected_scene = None;
                        }
                    });
                });
        }

        if dialog_state.show_load_dialog {
            // Get all scenes from assets/scenes directory
            let all_scenes: Vec<(String, String)> = std::fs::read_dir("assets/scenes")
                .map(|entries| {
                    entries
                        .filter_map(|entry| {
                            let entry = entry.ok()?;
                            let path = entry.path();
                            if path.extension()? == "ron" {
                                let file_name = path.file_name()?.to_str()?;
                                let display_name = file_name.trim_end_matches(".ron");
                                let full_path = format!("assets/scenes/{}", file_name);
                                Some((display_name.to_string(), full_path))
                            } else {
                                None
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            egui::Window::new("Load Scene")
                .collapsible(false)
                .resizable(true)
                .default_size([400.0, 300.0])
                .show(ui.ctx(), |ui| {
                    ui.label("Select a scene to load:");
                    ui.separator();
                    
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for (display_name, file_path) in &all_scenes {
                                let is_selected = dialog_state.selected_scene.as_ref() == Some(display_name);
                                
                                if ui.selectable_label(is_selected, display_name).clicked() {
                                    dialog_state.selected_scene = Some(display_name.clone());
                                    dialog_state.load_input_text = file_path.clone();
                                }
                                
                                // Double-click to load
                                if is_selected && ui.input(|i| i.pointer.button_double_clicked(egui::PointerButton::Primary)) {
                                    ev_load_scene.write(LoadSceneEvent(PathBuf::from(file_path)));
                                    dialog_state.show_load_dialog = false;
                                    dialog_state.selected_scene = None;
                                    break;
                                }
                            }
                        });
                    
                    ui.separator();
                    if let Some(selected) = &dialog_state.selected_scene {
                        ui.label(format!("Selected: {}", selected));
                    }
                    
                    ui.horizontal(|ui| {
                        let load_enabled = dialog_state.selected_scene.is_some();
                        if ui.add_enabled(load_enabled, egui::Button::new("Load")).clicked() {
                            if !dialog_state.load_input_text.is_empty() {
                                ev_load_scene.write(LoadSceneEvent(PathBuf::from(&dialog_state.load_input_text)));
                            }
                            dialog_state.show_load_dialog = false;
                            dialog_state.selected_scene = None;
                        }
                        if ui.button("Cancel").clicked() {
                            dialog_state.show_load_dialog = false;
                            dialog_state.selected_scene = None;
                        }
                    });
                });
        }
    }
}

fn handle_scene_dialog_markers(
    mut commands: Commands,
    load_markers: Query<Entity, With<ShowLoadSceneDialogMarker>>,
    save_markers: Query<Entity, With<ShowSaveSceneDialogMarker>>,
    mut dialog_state: ResMut<SceneSelectionDialog>,
) {
    for entity in load_markers.iter() {
        dialog_state.show_load_dialog = true;
        commands.entity(entity).despawn();
    }
    
    for entity in save_markers.iter() {
        dialog_state.show_save_dialog = true;
        commands.entity(entity).despawn();
    }
}

pub fn spawn_load_scene_dialog(commands: &mut Commands) {
    commands.spawn_empty().insert(ShowLoadSceneDialogMarker);
}

pub fn spawn_save_scene_dialog(commands: &mut Commands) {
    commands.spawn_empty().insert(ShowSaveSceneDialogMarker);
}

#[derive(Component)]
struct ShowLoadSceneDialogMarker;

#[derive(Component)]
struct ShowSaveSceneDialogMarker;