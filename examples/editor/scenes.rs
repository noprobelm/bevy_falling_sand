use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy_egui::egui;
use bevy_falling_sand::prelude::{LoadSceneEvent, SaveSceneEvent, LoadSceneAssetEvent};
use bfs_assets::ParticleSceneAsset;

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
        example_path.push("assets/scenes/hourglass.ron");

        ParticleSceneFilePath(example_path)
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
                            ev_save_scene.write(SaveSceneEvent(scene_path.0.clone()));
                            dialog_state.show_save_dialog = false; // Close after saving
                        }
                        if ui.button("Cancel").clicked() {
                            dialog_state.show_save_dialog = false;
                        }
                    });
                });
        }

        if dialog_state.show_load_dialog {
            // Built-in scenes (loaded from assets)
            let built_in_scenes = vec![
                ("Hourglass", "scenes/hourglass.ron"),
                ("Box", "scenes/box.ron"),
                ("Smaller Box", "scenes/smaller_box.ron"),
                ("Platforms", "scenes/platforms.ron"),
                ("Dividers", "scenes/dividers.ron"),
                ("Tree", "scenes/tree.ron"),
                ("Benchmark", "scenes/benchmark.ron"),
                ("Bevy Falling Sand", "scenes/bevy_falling_sand.ron"),
            ];

            // Also check for user-created scenes in the current directory
            let user_files: Vec<String> = std::fs::read_dir(&scene_path.0.parent().unwrap())
                .unwrap_or_else(|_| std::fs::read_dir(".").unwrap())
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
                            // Built-in scenes
                            ui.label("Built-in Scenes:");
                            for (display_name, asset_path) in &built_in_scenes {
                                if ui
                                    .selectable_value(
                                        &mut dialog_state.load_input_text,
                                        format!("asset:{}", asset_path),
                                        *display_name,
                                    )
                                    .changed()
                                {}
                            }

                            if !user_files.is_empty() {
                                ui.separator();
                                ui.label("User Scenes:");
                                for file_name in &user_files {
                                    let display_name =
                                        file_name.strip_suffix(".ron").unwrap_or(file_name);
                                    if ui
                                        .selectable_value(
                                            &mut dialog_state.load_input_text,
                                            format!("file:{}", file_name),
                                            display_name,
                                        )
                                        .changed()
                                    {
                                        scene_path.0.set_file_name(file_name.clone());
                                    }
                                }
                            }
                        });

                    if ui.button("Load").clicked() {
                        if dialog_state.load_input_text.starts_with("asset:") {
                            // Load from asset
                            let asset_path = dialog_state.load_input_text.strip_prefix("asset:").unwrap();
                            let handle: Handle<ParticleSceneAsset> = asset_server.load(asset_path);
                            ev_load_scene_asset.write(LoadSceneAssetEvent(handle));
                        } else if dialog_state.load_input_text.starts_with("file:") {
                            // Load from file
                            ev_load_scene.write(LoadSceneEvent(scene_path.0.clone()));
                        }
                        dialog_state.show_load_dialog = false; // Close after loading
                    }
                    if ui.button("Cancel").clicked() {
                        dialog_state.show_load_dialog = false;
                    }
                });
        }
    }
}

fn handle_scene_dialog_markers(
    mut commands: Commands,
    save_markers: Query<Entity, With<ShowSaveSceneDialogMarker>>,
    load_markers: Query<Entity, With<ShowLoadSceneDialogMarker>>,
    mut dialog_state: ResMut<SceneSelectionDialog>,
) {
    for entity in save_markers.iter() {
        dialog_state.show_save_dialog = true;
        commands.entity(entity).despawn();
    }
    
    for entity in load_markers.iter() {
        dialog_state.show_load_dialog = true;
        commands.entity(entity).despawn();
    }
}

pub fn spawn_save_scene_dialog(commands: &mut Commands) {
    commands.spawn_empty().insert(ShowSaveSceneDialogMarker);
}

pub fn spawn_load_scene_dialog(commands: &mut Commands) {
    commands.spawn_empty().insert(ShowLoadSceneDialogMarker);
}

#[derive(Component)]
struct ShowSaveSceneDialogMarker;

#[derive(Component)]
struct ShowLoadSceneDialogMarker;