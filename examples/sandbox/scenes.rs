use bevy::prelude::*;
use bevy_egui::egui;
use bevy_falling_sand::prelude::LoadSceneAssetEvent;
use bfs_assets::ParticleSceneAsset;

pub(super) struct ScenesPlugin;

impl bevy::prelude::Plugin for ScenesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<SceneSelectionDialog>();
    }
}

#[derive(Resource, Default)]
pub struct SceneSelectionDialog {
    pub show_load_dialog: bool,
    pub load_input_text: String,
}

pub struct SceneManagementUI;

impl SceneManagementUI {
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        dialog_state: &mut ResMut<SceneSelectionDialog>,
        ev_load_scene_asset: &mut EventWriter<LoadSceneAssetEvent>,
        asset_server: &Res<AssetServer>,
    ) {
        ui.horizontal_wrapped(|ui| {
            if ui.button("LOAD SCENE").clicked() {
                dialog_state.show_load_dialog = true;
            }
        });

        if dialog_state.show_load_dialog {
            let scenes = vec![
                ("Hourglass", "scenes/hourglass.ron"),
                ("Box", "scenes/box.ron"),
                ("Smaller Box", "scenes/smaller_box.ron"),
                ("Platforms", "scenes/platforms.ron"),
                ("Dividers", "scenes/dividers.ron"),
                ("Tree", "scenes/tree.ron"),
                ("Benchmark", "scenes/benchmark.ron"),
                ("Bevy Falling Sand", "scenes/bevy_falling_sand.ron"),
            ];

            egui::Window::new("Load Scene")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Select the scene to load:");

                    egui::ComboBox::from_label("Available Scenes")
                        .selected_text(dialog_state.load_input_text.clone())
                        .show_ui(ui, |ui| {
                            for (display_name, asset_path) in &scenes {
                                if ui
                                    .selectable_value(
                                        &mut dialog_state.load_input_text,
                                        asset_path.to_string(),
                                        *display_name,
                                    )
                                    .changed()
                                {}
                            }
                        });

                    if ui.button("Load").clicked() {
                        if !dialog_state.load_input_text.is_empty() {
                            let handle: Handle<ParticleSceneAsset> = asset_server.load(&dialog_state.load_input_text);
                            ev_load_scene_asset.write(LoadSceneAssetEvent(handle));
                        }
                        dialog_state.show_load_dialog = false;
                    }
                    if ui.button("Cancel").clicked() {
                        dialog_state.show_load_dialog = false;
                    }
                });
        }
    }
}
