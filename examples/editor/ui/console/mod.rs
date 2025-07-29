pub mod core;

use bevy::prelude::*;
use bevy_egui::egui;

use core::{
    ConsoleCache, ConsoleCommandEntered, ConsoleConfiguration, ConsoleState, PrintConsoleLine,
};

pub use core::ConsolePlugin;

pub struct Console;

impl Console {
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        console_state: &mut ConsoleState,
        cache: &ConsoleCache,
        config: &ConsoleConfiguration,
        command_writer: &mut EventWriter<ConsoleCommandEntered>,
    ) {
        self.render_with_particle_cache(ui, console_state, cache, config, command_writer, None);
    }

    pub fn render_with_particle_cache(
        &self,
        ui: &mut egui::Ui,
        console_state: &mut ConsoleState,
        cache: &ConsoleCache,
        config: &ConsoleConfiguration,
        command_writer: &mut EventWriter<ConsoleCommandEntered>,
        particle_cache: Option<&crate::ui::particle_search::ParticleSearchCache>,
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

                        let cursor_at_end = console_state.needs_cursor_at_end;
                        if cursor_at_end {
                            console_state.needs_cursor_at_end = false;
                            console_state.request_focus_and_cursor = true;
                        }

                        let text_edit_id = egui::Id::new("console_input");

                        // Filter out backtick characters from input events
                        ui.input_mut(|i| {
                            i.events.retain(|event| {
                                !matches!(event, egui::Event::Text(text) if text.contains('`'))
                            });
                        });

                        let response = ui.add(
                            egui::TextEdit::singleline(&mut console_state.input)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(ui.available_width())
                                .lock_focus(true)
                                .id(text_edit_id),
                        );

                        if !console_state.in_completion_mode
                            && !console_state.suggestions.is_empty()
                        {
                            if let Some(suggestion) = console_state.suggestions.first() {
                                let completed_input =
                                    calculate_completed_input(&console_state.input, suggestion);

                                if completed_input.len() > console_state.input.len() {
                                    let remaining_text =
                                        &completed_input[console_state.input.len()..];
                                    if !remaining_text.is_empty() {
                                        let font_id = ui
                                            .style()
                                            .text_styles
                                            .get(&egui::TextStyle::Monospace)
                                            .unwrap_or(&egui::FontId::monospace(14.0))
                                            .clone();

                                        let text_galley = ui.fonts(|f| {
                                            f.layout_no_wrap(
                                                console_state.input.clone(),
                                                font_id.clone(),
                                                egui::Color32::WHITE,
                                            )
                                        });

                                        let text_edit_margin = ui.spacing().button_padding.x;
                                        let text_edit_content_rect = response.rect;
                                        let text_start_x =
                                            text_edit_content_rect.left() + text_edit_margin;
                                        let text_y = text_edit_content_rect.center().y
                                            - (text_galley.size().y / 2.0);

                                        let suggestion_pos = egui::Pos2::new(
                                            text_start_x + text_galley.size().x,
                                            text_y,
                                        );

                                        ui.painter().text(
                                            suggestion_pos,
                                            egui::Align2::LEFT_TOP,
                                            remaining_text,
                                            font_id,
                                            egui::Color32::from_rgb(120, 120, 120),
                                        );
                                    }
                                }
                            }
                        }

                        if console_state.request_focus_and_cursor {
                            response.request_focus();

                            if let Some(mut state) =
                                egui::TextEdit::load_state(ui.ctx(), text_edit_id)
                            {
                                let text_len = console_state.input.len();
                                state
                                    .cursor
                                    .set_char_range(Some(egui::text::CCursorRange::one(
                                        egui::text::CCursor::new(text_len),
                                    )));
                                state.store(ui.ctx(), text_edit_id);
                            }

                            console_state.request_focus_and_cursor = false;
                        }

                        if (backtick_pressed && console_state.expanded)
                            || console_state.needs_initial_focus
                        {
                            response.request_focus();
                            console_state.needs_initial_focus = false;
                        }

                        if response.changed() {
                            console_state.on_input_changed();
                            console_state.update_suggestions_with_particle_cache(cache, config, particle_cache);
                        }

                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            console_state.commit_completion();

                            if !console_state.input.trim().is_empty() {
                                let command = console_state.input.clone();
                                console_state.input.clear();
                                console_state.suggestions.clear();
                                console_state.suggestion_index = None;
                                console_state.execute_command(command, config, command_writer);
                                console_state.history_index = 0;
                                if !console_state.expanded {
                                    console_state.expanded = true;
                                }
                            }
                            response.request_focus();
                        }

                        if response.has_focus() {
                            let mut tab_handled = false;
                            let mut space_pressed = false;

                            ui.input_mut(|i| {
                                if i.key_pressed(egui::Key::Tab)
                                    && !console_state.suggestions.is_empty()
                                {
                                    if !console_state.in_completion_mode {
                                        console_state.update_suggestions_with_particle_cache(cache, config, particle_cache);
                                    }

                                    console_state.handle_tab_completion();
                                    tab_handled = true;
                                    i.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
                                    if i.modifiers.shift {
                                        i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab);
                                    }
                                }

                                if i.key_pressed(egui::Key::Space)
                                    && console_state.in_completion_mode
                                {
                                    console_state.commit_completion();
                                    console_state.input.push(' ');
                                    console_state.update_suggestions_with_particle_cache(cache, config, particle_cache);
                                    space_pressed = true;
                                    i.consume_key(egui::Modifiers::NONE, egui::Key::Space);
                                }
                            });

                            if tab_handled || space_pressed {
                                response.request_focus();
                            }
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
}

pub fn receive_console_line(
    mut console_state: ResMut<ConsoleState>,
    mut events: EventReader<PrintConsoleLine>,
) {
    for event in events.read() {
        console_state.add_message(event.line.clone());
    }
}

fn calculate_completed_input(current_input: &str, suggestion: &str) -> String {
    if current_input.is_empty() {
        return suggestion.to_string();
    }

    if current_input.ends_with(' ') {
        format!("{}{}", current_input, suggestion)
    } else {
        let words: Vec<&str> = current_input.trim().split_whitespace().collect();

        if words.len() == 1 {
            suggestion.to_string()
        } else {
            let mut complete_words = words[..words.len() - 1].to_vec();
            complete_words.push(suggestion);
            complete_words.join(" ")
        }
    }
}
