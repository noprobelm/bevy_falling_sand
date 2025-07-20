use bevy::prelude::*;
use bevy_egui::egui;

use crate::console::core::{
    ConsoleCache, ConsoleCommandEntered, ConsoleConfiguration, ConsoleState, PrintConsoleLine,
};

pub fn render_console(
    ui: &mut egui::Ui,
    console_state: &mut ConsoleState,
    cache: &ConsoleCache,
    config: &ConsoleConfiguration,
    command_writer: &mut EventWriter<ConsoleCommandEntered>,
) {
    let backtick_pressed = ui.input(|i| i.key_pressed(egui::Key::Backtick));
    if backtick_pressed {
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
                                ui.label(egui::RichText::new(message).monospace().color(color));
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

                    // Create the input field with inline autocompletion
                    let current_suggestion = if !console_state.suggestions.is_empty() {
                        console_state
                            .suggestion_index
                            .and_then(|i| console_state.suggestions.get(i))
                            .or_else(|| console_state.suggestions.first())
                            .cloned()
                    } else {
                        None
                    };

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut console_state.input)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(ui.available_width()),
                    );

                    // Auto-focus when console is opened with backtick or on initial load
                    if (backtick_pressed && console_state.expanded) || console_state.needs_initial_focus {
                        response.request_focus();
                        console_state.needs_initial_focus = false;
                    }

                    // Render inline autocompletion suggestion
                    if let Some(suggestion) = &current_suggestion {
                        if suggestion.starts_with(&console_state.input)
                            && !console_state.input.is_empty()
                        {
                            let remaining_text = &suggestion[console_state.input.len()..];
                            if !remaining_text.is_empty() {
                                // Calculate position for the suggestion text more accurately
                                let font_id = egui::FontId::monospace(14.0);
                                let text_galley = ui.fonts(|f| {
                                    f.layout_no_wrap(
                                        console_state.input.clone(),
                                        font_id.clone(),
                                        egui::Color32::WHITE,
                                    )
                                });

                                // Position relative to the text edit field's content area
                                let text_edit_content_rect = response.rect;
                                let text_start_x = text_edit_content_rect.left() + 4.0; // Small padding inside text edit
                                let text_y = text_edit_content_rect.center().y
                                    - (text_galley.size().y / 2.0);

                                let suggestion_pos =
                                    egui::Pos2::new(text_start_x + text_galley.size().x, text_y);

                                ui.painter().text(
                                    suggestion_pos,
                                    egui::Align2::LEFT_TOP,
                                    remaining_text,
                                    font_id,
                                    egui::Color32::from_rgb(120, 120, 120), // Grayed out
                                );
                            }
                        }
                    }

                    if response.changed() {
                        console_state.history_index = 0;
                        console_state.update_suggestions(cache);
                    }

                    // Handle Enter key submission - check before focus is lost
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        // Auto-complete with current suggestion if available
                        if let Some(suggestion) = &current_suggestion {
                            if suggestion.starts_with(&console_state.input)
                                && !console_state.input.is_empty()
                            {
                                console_state.input = suggestion.clone();
                            }
                        }

                        if !console_state.input.trim().is_empty() {
                            let command = console_state.input.clone();
                            console_state.input.clear();
                            console_state.suggestions.clear(); // Clear suggestions after command
                            console_state.suggestion_index = None;
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

                    // Handle Tab key for cycling through suggestions
                    if response.has_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Tab))
                        && !console_state.suggestions.is_empty()
                    {
                        match &mut console_state.suggestion_index {
                            Some(index) => {
                                *index = (*index + 1) % console_state.suggestions.len();
                            }
                            None => {
                                console_state.suggestion_index = Some(0);
                            }
                        }
                        // Don't auto-complete on Tab, just cycle through suggestions for inline display
                        response.request_focus(); // Keep focus
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

// System to receive console output
pub fn receive_console_line(
    mut console_state: ResMut<ConsoleState>,
    mut events: EventReader<PrintConsoleLine>,
) {
    for event in events.read() {
        console_state.add_message(event.line.clone());
    }
}

