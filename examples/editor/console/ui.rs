use bevy::prelude::*;
use bevy_egui::egui;

use crate::console::core::{ConsoleCache, ConsoleCommandEntered, ConsoleConfiguration, ConsoleState, PrintConsoleLine};

pub struct ConsoleUiPlugin;

impl Plugin for ConsoleUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_console_system)
            .add_systems(Update, receive_console_line);
    }
}

#[derive(Component)]
pub struct ConsoleUi;

fn render_console_system(
    mut contexts: bevy_egui::EguiContexts,
    mut console_state: ResMut<ConsoleState>,
    console_cache: Res<ConsoleCache>,
    config: Res<ConsoleConfiguration>,
    mut command_writer: EventWriter<ConsoleCommandEntered>,
) {
    let ctx = contexts.ctx_mut();
    
    egui::TopBottomPanel::bottom("console_panel")
        .resizable(false)
        .show(ctx, |ui| {
            render_console(ui, &mut console_state, &console_cache, &config, &mut command_writer);
        });
}

pub fn render_console(
    ui: &mut egui::Ui,
    console_state: &mut ConsoleState,
    cache: &ConsoleCache,
    config: &ConsoleConfiguration,
    command_writer: &mut EventWriter<ConsoleCommandEntered>,
) {
    if ui.input(|i| i.key_pressed(egui::Key::Backtick)) {
        console_state.toggle();
    }

    let available_height = ui.available_height();

    let _frame_response = egui::Frame::new()
        .fill(egui::Color32::from_rgb(46, 46, 46))
        .corner_radius(4.0)
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            let console_is_hovered = ui.rect_contains_pointer(ui.max_rect());

            ui.vertical(|ui| {
                if console_state.expanded {
                    let resize_response = ui.allocate_response(
                        egui::Vec2::new(ui.available_width(), 8.0),
                        egui::Sense::drag(),
                    );

                    if resize_response.dragged() {
                        let drag_delta = resize_response.drag_delta().y;
                        console_state.height =
                            (console_state.height - drag_delta).clamp(80.0, 600.0);
                    }

                    if resize_response.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
                    }

                    let handle_rect = resize_response.rect;
                    let handle_center = handle_rect.center();
                    ui.painter().hline(
                        handle_center.x - 20.0..=handle_center.x + 20.0,
                        handle_center.y - 1.0,
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)),
                    );
                    ui.painter().hline(
                        handle_center.x - 20.0..=handle_center.x + 20.0,
                        handle_center.y + 1.0,
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)),
                    );

                    let text_height = available_height - 50.0;

                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .max_height(text_height)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for message in &console_state.messages {
                                let color = if message.starts_with("error:") {
                                    egui::Color32::from_rgb(255, 100, 100)
                                } else {
                                    egui::Color32::from_rgb(200, 200, 200)
                                };
                                ui.label(
                                    egui::RichText::new(message)
                                        .monospace()
                                        .color(color),
                                );
                            }
                        });

                    ui.separator();
                }

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(">")
                            .monospace()
                            .color(egui::Color32::from_rgb(100, 200, 100)),
                    );

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut console_state.input)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(ui.available_width()),
                    );

                    if response.changed() {
                        console_state.history_index = 0;
                        console_state.update_suggestions(cache);
                    }

                    // Handle Enter key submission - check this before focus is lost
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if !console_state.input.trim().is_empty() {
                            let command = console_state.input.clone();
                            console_state.input.clear();
                            console_state.execute_command(command, config, command_writer);
                            console_state.history_index = 0;
                            // Auto-expand when command is executed
                            if !console_state.expanded {
                                console_state.expanded = true;
                            }
                        }
                        // Re-focus the input for next command
                        response.request_focus();
                    }

                    if response.has_focus() {
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                            console_state.navigate_history(true);
                        }

                        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                            console_state.navigate_history(false);
                        }
                    }

                    if console_is_hovered && !response.has_focus() {
                        response.request_focus();
                    }
                });
            });
        });
}

fn receive_console_line(
    mut console_state: ResMut<ConsoleState>,
    mut events: EventReader<PrintConsoleLine>,
) {
    for event in events.read() {
        console_state.add_message(event.line.clone());
    }
}