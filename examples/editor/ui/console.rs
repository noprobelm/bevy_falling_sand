use bevy::prelude::*;
use bevy_egui::egui;
use clap::{CommandFactory, FromArgMatches, Parser};
use shlex::Shlex;
use std::collections::{BTreeMap, VecDeque};
use trie_rs::{Trie, TrieBuilder};

// Events for console system
#[derive(Clone, Debug, Event)]
pub struct ConsoleCommandEntered {
    pub command_name: String,
    pub args: Vec<String>,
}

#[derive(Clone, Debug, Event)]
pub struct PrintConsoleLine {
    pub line: String,
}

impl PrintConsoleLine {
    pub fn new(line: String) -> Self {
        Self { line }
    }
}

// Resource for console configuration and commands
#[derive(Resource)]
pub struct ConsoleConfiguration {
    pub commands: BTreeMap<&'static str, clap::Command>,
    pub history_size: usize,
    pub symbol: String,
}

impl Default for ConsoleConfiguration {
    fn default() -> Self {
        Self {
            commands: BTreeMap::new(),
            history_size: 20,
            symbol: "> ".to_owned(),
        }
    }
}

// Command cache for autocompletion
#[derive(Resource, Default)]
pub struct ConsoleCache {
    pub commands_trie: Option<Trie<u8>>,
}

#[derive(Resource)]
pub struct ConsoleState {
    pub messages: Vec<String>,
    pub input: String,
    pub history: VecDeque<String>,
    pub history_index: usize,
    pub expanded: bool,
    pub height: f32,
    pub suggestions: Vec<String>,
    pub suggestion_index: Option<usize>,
}

impl Default for ConsoleState {
    fn default() -> Self {
        let mut state = Self {
            messages: Vec::new(),
            input: String::new(),
            history: VecDeque::from([String::new()]),
            history_index: 0,
            expanded: true,
            height: 300.0,
            suggestions: Vec::new(),
            suggestion_index: None,
        };

        state.add_message("--- Bevy Falling Sand Editor Console ---".to_string());
        state.add_message("Console ready. Type 'help' for available commands.".to_string());

        state
    }
}

// Traits for command system
pub trait Command: NamedCommand + CommandFactory + FromArgMatches + Sized + Resource {}
impl<T: NamedCommand + CommandFactory + FromArgMatches + Sized + Resource> Command for T {}

pub trait NamedCommand {
    fn name() -> &'static str;
}

// System param for commands
pub struct ConsoleCommand<'w, T> {
    command: Option<Result<T, clap::Error>>,
    console_line: EventWriter<'w, PrintConsoleLine>,
}

impl<T> ConsoleCommand<'_, T> {
    pub fn take(&mut self) -> Option<Result<T, clap::Error>> {
        self.command.take()
    }

    pub fn ok(&mut self) {
        self.console_line
            .write(PrintConsoleLine::new("[ok]".into()));
    }

    pub fn failed(&mut self) {
        self.console_line
            .write(PrintConsoleLine::new("[failed]".into()));
    }

    pub fn reply(&mut self, msg: impl Into<String>) {
        self.console_line.write(PrintConsoleLine::new(msg.into()));
    }

    pub fn reply_ok(&mut self, msg: impl Into<String>) {
        self.console_line.write(PrintConsoleLine::new(msg.into()));
        self.ok();
    }

    pub fn reply_failed(&mut self, msg: impl Into<String>) {
        self.console_line.write(PrintConsoleLine::new(msg.into()));
        self.failed();
    }
}

// Built-in commands
#[derive(Parser, Resource)]
#[command(name = "help")]
pub struct HelpCommand {
    command: Option<String>,
}

impl NamedCommand for HelpCommand {
    fn name() -> &'static str {
        "help"
    }
}

#[derive(Parser, Resource)]
#[command(name = "clear")]
pub struct ClearCommand;

impl NamedCommand for ClearCommand {
    fn name() -> &'static str {
        "clear"
    }
}

#[derive(Parser, Resource)]
#[command(name = "echo")]
pub struct EchoCommand {
    message: String,
}

impl NamedCommand for EchoCommand {
    fn name() -> &'static str {
        "echo"
    }
}

impl ConsoleState {
    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }

    pub fn execute_command(
        &mut self,
        command: String,
        config: &ConsoleConfiguration,
        command_writer: &mut EventWriter<ConsoleCommandEntered>,
    ) {
        if command.trim().is_empty() {
            return;
        }

        self.add_message(format!("{}{}", config.symbol, command));
        self.history.insert(1, command.clone());
        if self.history.len() > config.history_size + 1 {
            self.history.pop_back();
        }
        self.history_index = 0;

        let mut args = Shlex::new(&command).collect::<Vec<_>>();
        if !args.is_empty() {
            let command_name = args.remove(0);

            if config.commands.contains_key(command_name.as_str()) {
                self.add_message(format!("Executing command: {}", command_name));
                command_writer.write(ConsoleCommandEntered { command_name, args });
            } else {
                self.add_message(format!("error: Unknown command '{}'", command_name));
                self.add_message("Available commands: help, clear, echo".to_string());
            }
        }
    }

    pub fn navigate_history(&mut self, up: bool) {
        if self.history.len() <= 1 {
            return;
        }

        if up && self.history_index < self.history.len() - 1 {
            if self.history_index == 0 && !self.input.trim().is_empty() {
                *self.history.get_mut(0).unwrap() = self.input.clone();
            }
            self.history_index += 1;
            self.input = self.history.get(self.history_index).unwrap().clone();
        } else if !up && self.history_index > 0 {
            self.history_index -= 1;
            self.input = self.history.get(self.history_index).unwrap().clone();
        }
    }

    pub fn update_suggestions(&mut self, cache: &ConsoleCache) {
        self.suggestions.clear();
        self.suggestion_index = None;

        if !self.input.is_empty() {
            if let Some(trie) = &cache.commands_trie {
                let words = Shlex::new(&self.input).collect::<Vec<_>>();
                let query = words.join(" ");

                self.suggestions = trie
                    .predictive_search(query)
                    .into_iter()
                    .take(5)
                    .map(|s| String::from_utf8(s).unwrap_or_default())
                    .collect();
            }
        }
    }
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
                        console_state.suggestion_index
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

                    // Render inline autocompletion suggestion
                    if let Some(suggestion) = &current_suggestion {
                        if suggestion.starts_with(&console_state.input) && !console_state.input.is_empty() {
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
                                let text_y = text_edit_content_rect.center().y - (text_galley.size().y / 2.0);
                                
                                let suggestion_pos = egui::Pos2::new(
                                    text_start_x + text_galley.size().x,
                                    text_y
                                );
                                
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
                        println!("Input changed to: '{}', suggestions: {:?}", console_state.input, console_state.suggestions);
                    }

                    // Handle Enter key submission - auto-complete if suggestion exists
                    if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        // Auto-complete with current suggestion if available
                        if let Some(suggestion) = &current_suggestion {
                            if suggestion.starts_with(&console_state.input) && !console_state.input.is_empty() {
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
                    if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Tab)) && !console_state.suggestions.is_empty() {
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

// Command handler systems
pub fn help_command(
    mut cmd: EventReader<ConsoleCommandEntered>,
    config: Res<ConsoleConfiguration>,
    mut writer: EventWriter<PrintConsoleLine>,
) {
    for command_event in cmd.read() {
        if command_event.command_name == "help" {
            if let Some(target_cmd) = command_event.args.first() {
                if let Some(cmd_info) = config.commands.get(target_cmd.as_str()) {
                    let mut cloned_cmd = cmd_info.clone();
                    writer.write(PrintConsoleLine::new(
                        cloned_cmd.render_long_help().to_string(),
                    ));
                } else {
                    writer.write(PrintConsoleLine::new(format!(
                        "Command '{}' does not exist",
                        target_cmd
                    )));
                }
            } else {
                writer.write(PrintConsoleLine::new("Available commands:".to_string()));
                for (name, cmd) in &config.commands {
                    let help_text = cmd.get_about().map(|s| s.to_string()).unwrap_or_default();
                    writer.write(PrintConsoleLine::new(format!("  {} - {}", name, help_text)));
                }
            }
        }
    }
}

pub fn clear_command(
    mut cmd: EventReader<ConsoleCommandEntered>,
    mut console_state: ResMut<ConsoleState>,
) {
    for command_event in cmd.read() {
        if command_event.command_name == "clear" {
            console_state.messages.clear();
            console_state.add_message("--- Bevy Falling Sand Editor Console ---".to_string());
        }
    }
}

pub fn echo_command(
    mut cmd: EventReader<ConsoleCommandEntered>,
    mut writer: EventWriter<PrintConsoleLine>,
) {
    for command_event in cmd.read() {
        if command_event.command_name == "echo" {
            let message = command_event.args.join(" ");
            writer.write(PrintConsoleLine::new(message));
        }
    }
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

// Initialization system to populate command registry
pub fn init_commands(mut config: ResMut<ConsoleConfiguration>, mut cache: ResMut<ConsoleCache>) {
    // Register help command
    let help_cmd = HelpCommand::command().no_binary_name(true);
    config.commands.insert(HelpCommand::name(), help_cmd);

    // Register clear command
    let clear_cmd = ClearCommand::command().no_binary_name(true);
    config.commands.insert(ClearCommand::name(), clear_cmd);

    // Register echo command
    let echo_cmd = EchoCommand::command().no_binary_name(true);
    config.commands.insert(EchoCommand::name(), echo_cmd);
    
    // Build command trie for autocompletion
    let mut builder: TrieBuilder<u8> = TrieBuilder::new();
    for name in config.commands.keys() {
        println!("Registering command: {}", name);
        builder.push(name.as_bytes());
    }
    cache.commands_trie = Some(builder.build());
    println!("Built trie with {} commands", config.commands.len());
}
