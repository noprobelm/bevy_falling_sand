use bevy::prelude::*;
use clap::{CommandFactory, FromArgMatches};
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
    pub command: Option<Result<T, clap::Error>>,
    pub console_line: EventWriter<'w, PrintConsoleLine>,
}

impl<T> ConsoleCommand<'_, T> {
    pub fn take(&mut self) -> Option<Result<T, clap::Error>> {
        self.command.take()
    }

    pub fn ok(&mut self) {
        self.console_line.write(PrintConsoleLine::new("[ok]".into()));
    }

    pub fn failed(&mut self) {
        self.console_line.write(PrintConsoleLine::new("[failed]".into()));
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
                command_writer.write(ConsoleCommandEntered {
                    command_name,
                    args,
                });
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
        println!("update_suggestions called with input: '{}'", self.input);
        self.suggestions.clear();
        self.suggestion_index = None;
        
        if !self.input.is_empty() {
            if let Some(trie) = &cache.commands_trie {
                // Only search for command names (first word)
                let trimmed = self.input.trim();
                let first_word = trimmed.split_whitespace().next().unwrap_or("");
                
                // Only suggest if we're still typing the first word
                if !trimmed.contains(' ') && !first_word.is_empty() {
                    self.suggestions = trie
                        .predictive_search(first_word.as_bytes())
                        .into_iter()
                        .take(5)
                        .map(|s| String::from_utf8(s).unwrap_or_default())
                        .collect();
                    
                    if !self.suggestions.is_empty() {
                        println!("Found {} suggestions for '{}': {:?}", self.suggestions.len(), first_word, self.suggestions);
                    }
                }
            } else {
                println!("No command trie available!");
            }
        }
    }
}

// Initialization system to populate command registry
pub fn init_commands(mut config: ResMut<ConsoleConfiguration>, mut cache: ResMut<ConsoleCache>) {
    use crate::console::commands::{HelpCommand, ClearCommand, EchoCommand};
    
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
        println!("Registering command for autocompletion: {}", name);
        builder.push(name.as_bytes());
    }
    let trie = builder.build();
    
    // Test the trie
    println!("Testing trie:");
    for prefix in ["h", "he", "hel", "c", "cl", "e", "ec"] {
        let results: Vec<String> = trie.predictive_search(prefix.as_bytes())
            .into_iter()
            .map(|s| String::from_utf8(s).unwrap_or_default())
            .collect();
        println!("  '{}' -> {:?}", prefix, results);
    }
    
    cache.commands_trie = Some(trie);
    println!("Command trie built with {} commands", config.commands.len());
}