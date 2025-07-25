
use bevy::prelude::*;
use bevy_egui::egui;
use bevy_falling_sand::prelude::{LoadSceneEvent, SaveSceneEvent};
use crate::ui::file_browser::{FileBrowser, FileBrowserState};
use std::path::PathBuf;

pub(super) struct ScenesPlugin;

impl bevy::prelude::Plugin for ScenesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<SceneSelectionDialog>()
            .init_resource::<ParticleSceneFilePath>()
            .init_resource::<SceneFileBrowserState>()
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

// We'll keep a separate browser state for scenes
#[derive(Resource)]
pub struct SceneFileBrowserState(pub FileBrowserState);

impl Default for SceneFileBrowserState {
    fn default() -> Self {
        Self(FileBrowserState::new("assets/scenes", "ron", "Scene Files"))
    }
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
        scene_browser_state: &mut ResMut<SceneFileBrowserState>,
        ev_save_scene: &mut EventWriter<SaveSceneEvent>,
        ev_load_scene: &mut EventWriter<LoadSceneEvent>,
    ) {
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

fn handle_scene_dialog_markers(
    mut commands: Commands,
    load_markers: Query<Entity, With<ShowLoadSceneDialogMarker>>,
    save_markers: Query<Entity, With<ShowSaveSceneDialogMarker>>,
    mut scene_browser_state: ResMut<SceneFileBrowserState>,
) {
    for entity in load_markers.iter() {
        scene_browser_state.0.show_load("Load Scene");
        commands.entity(entity).despawn();
    }
    
    for entity in save_markers.iter() {
        scene_browser_state.0.show_save("Save Scene");
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