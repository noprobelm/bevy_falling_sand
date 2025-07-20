pub mod console;
mod layers_panel;
mod particle_editor;
mod top_bar;

use crate::console::{
    core::{ConsoleCache, ConsoleCommandEntered, ConsoleConfiguration, ConsoleState},
    ConsolePlugin,
};
use console::render_console;
use layers_panel::LayersPanel;
use particle_editor::ParticleEditor;
use top_bar::UiTopBar;

use bevy::prelude::*;
pub(super) use bevy_egui::*;

pub struct UiPlugin;

#[derive(Resource, Default)]
struct UiInteractionState {
    mouse_over_ui: bool,
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EguiPlugin {
                enable_multipass_for_primary_context: false,
            },
            ConsolePlugin,
        ))
        .init_state::<AppState>()
        .init_resource::<UiInteractionState>()
        .add_systems(Update, console::receive_console_line)
        .add_systems(
            EguiContextPass,
            (
                render.before(bevy_egui::EguiPreUpdateSet::InitContexts),
                detect_ui_interaction.after(render),
            ),
        );
    }
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Canvas,
    Ui,
}

fn render(
    mut contexts: EguiContexts,
    mut console_state: ResMut<ConsoleState>,
    cache: Res<ConsoleCache>,
    config: Res<ConsoleConfiguration>,
    mut command_writer: EventWriter<ConsoleCommandEntered>,
    mut ui_state: ResMut<UiInteractionState>,
) {
    let ctx = contexts.ctx_mut();

    ui_state.mouse_over_ui = false;

    let top_response = egui::TopBottomPanel::top("Top panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            UiTopBar.render(ui);
        });
    });

    if top_response.response.hovered() {
        ui_state.mouse_over_ui = true;
    }

    let left_response = egui::SidePanel::left("Left panel")
        .resizable(false)
        .show(ctx, |ui| {
            ui.painter().rect_filled(
                ui.available_rect_before_wrap(),
                0.0,
                egui::Color32::from_rgb(30, 30, 30),
            );

            let available_rect = ui.available_rect_before_wrap();
            let spacing = 8.0;
            let panel_height = (available_rect.height() - spacing) / 2.0;
            let panel_bg = egui::Color32::from_rgb(46, 46, 46);

            let top_response = ui.allocate_response(
                egui::vec2(available_rect.width(), panel_height),
                egui::Sense::hover(),
            );

            ui.scope_builder(egui::UiBuilder::new().max_rect(top_response.rect), |ui| {
                ui.set_clip_rect(top_response.rect);
                egui::Frame::NONE
                    .fill(panel_bg)
                    .corner_radius(4.0)
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        ui.set_min_height(panel_height - 16.0);
                        ui.set_max_height(panel_height - 16.0);
                        ParticleEditor.render(ui);
                    });
            });

            ui.add_space(spacing);

            let bottom_response = ui.allocate_response(
                egui::vec2(available_rect.width(), panel_height),
                egui::Sense::hover(),
            );

            ui.scope_builder(
                egui::UiBuilder::new().max_rect(bottom_response.rect),
                |ui| {
                    ui.set_clip_rect(bottom_response.rect);
                    egui::Frame::NONE
                        .fill(panel_bg)
                        .corner_radius(4.0)
                        .inner_margin(egui::Margin::same(8))
                        .show(ui, |ui| {
                            ui.set_min_height(panel_height - 16.0);
                            ui.set_max_height(panel_height - 16.0);
                            LayersPanel.render(ui);
                        });
                },
            );
        });

    if left_response.response.hovered() {
        ui_state.mouse_over_ui = true;
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.painter().rect_filled(
            ui.available_rect_before_wrap(),
            0.0,
            egui::Color32::from_rgb(30, 30, 30),
        );

        let console_height = if console_state.expanded {
            console_state.height
        } else {
            40.0
        };

        let available_rect = ui.available_rect_before_wrap();
        let console_rect = egui::Rect::from_min_size(
            egui::pos2(
                available_rect.left(),
                available_rect.bottom() - console_height,
            ),
            egui::vec2(available_rect.width(), console_height),
        );

        let _response = ui.allocate_rect(console_rect, egui::Sense::hover());
        ui.scope_builder(egui::UiBuilder::new().max_rect(console_rect), |ui| {
            ui.set_clip_rect(console_rect);
            render_console(ui, &mut console_state, &cache, &config, &mut command_writer);
        });
    });
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
