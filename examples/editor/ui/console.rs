use bevy::prelude::*;
use bevy_egui::egui;

#[derive(Resource)]
pub struct ConsoleState {
    pub messages: Vec<String>,
    pub input: String,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub expanded: bool,
    pub height: f32,
}

impl Default for ConsoleState {
    fn default() -> Self {
        let mut state = Self {
            messages: Vec::new(),
            input: String::new(),
            history: Vec::new(),
            history_index: None,
            expanded: true,
            height: 300.0,
        };

        state.add_message("--- Bevy Falling Sand Editor Console ---".to_string());
        state.add_message("Type 'help' for available commands".to_string());

        state
    }
}

impl ConsoleState {
    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }

    pub fn execute_command(&mut self, command: String) {
        if command.trim().is_empty() {
            return;
        }

        self.add_message(format!("> {}", command));
        self.history.push(command.clone());
        self.history_index = None;

        match command.trim() {
            "clear" => {
                self.messages.clear();
            }
            "help" => {
                self.add_message("Available commands:".to_string());
                self.add_message("  clear - Clear the console".to_string());
                self.add_message("  help  - Show this help message".to_string());
            }
            _ => {
                self.add_message(format!("Unknown command: {}", command));
            }
        }
    }

    pub fn navigate_history(&mut self, up: bool) {
        if self.history.is_empty() {
            return;
        }

        match (self.history_index, up) {
            (None, true) => {
                self.history_index = Some(self.history.len() - 1);
                self.input = self.history[self.history.len() - 1].clone();
            }
            (Some(idx), true) if idx > 0 => {
                self.history_index = Some(idx - 1);
                self.input = self.history[idx - 1].clone();
            }
            (Some(idx), false) if idx < self.history.len() - 1 => {
                self.history_index = Some(idx + 1);
                self.input = self.history[idx + 1].clone();
            }
            (Some(idx), false) if idx == self.history.len() - 1 => {
                self.history_index = None;
                self.input.clear();
            }
            _ => {}
        }
    }
}

pub fn render_console(ui: &mut egui::Ui, console_state: &mut ConsoleState) {
    if ui.input(|i| i.key_pressed(egui::Key::Backtick)) {
        console_state.toggle();
    }

    // Check for Enter key to submit command when console has focus
    if ui.input(|i| i.key_pressed(egui::Key::Enter)) && !console_state.input.is_empty() {
        // We'll check if the input actually has focus inside the UI logic
        let command = console_state.input.clone();
        console_state.input.clear();
        console_state.execute_command(command);
        // Auto-expand when command is executed
        if !console_state.expanded {
            console_state.expanded = true;
        }
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
                                ui.label(
                                    egui::RichText::new(message)
                                        .monospace()
                                        .color(egui::Color32::from_rgb(200, 200, 200)),
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
                        console_state.history_index = None;
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
