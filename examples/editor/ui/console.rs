use bevy::prelude::*;
use bevy_egui::egui;

#[derive(Resource)]
pub struct ConsoleState {
    pub messages: Vec<String>,
    pub input: String,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub scroll_to_bottom: bool,
}

impl Default for ConsoleState {
    fn default() -> Self {
        let mut state = Self {
            messages: Vec::new(),
            input: String::new(),
            history: Vec::new(),
            history_index: None,
            scroll_to_bottom: true,
        };
        
        state.add_message("Welcome to Falling Sand Editor Console".to_string());
        state.add_message("Type 'help' for available commands".to_string());
        state.add_message(String::new());
        
        state
    }
}

impl ConsoleState {
    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
        self.scroll_to_bottom = true;
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
    let available_height = ui.available_height();
    
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(20, 20, 20))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)))
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                let text_height = available_height - 40.0;
                
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(text_height)
                    .stick_to_bottom(console_state.scroll_to_bottom)
                    .show(ui, |ui| {
                        for message in &console_state.messages {
                            ui.label(
                                egui::RichText::new(message)
                                    .monospace()
                                    .color(egui::Color32::from_rgb(200, 200, 200))
                            );
                        }
                        
                        if console_state.scroll_to_bottom {
                            console_state.scroll_to_bottom = false;
                        }
                    });
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(">")
                            .monospace()
                            .color(egui::Color32::from_rgb(100, 200, 100))
                    );
                    
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut console_state.input)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(ui.available_width())
                            .lock_focus(true)
                    );
                    
                    if response.changed() {
                        console_state.history_index = None;
                    }
                    
                    if response.has_focus() {
                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            let command = console_state.input.clone();
                            console_state.input.clear();
                            console_state.execute_command(command);
                        }
                        
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                            console_state.navigate_history(true);
                        }
                        
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                            console_state.navigate_history(false);
                        }
                    }
                    
                    response.request_focus();
                });
            });
        });
}