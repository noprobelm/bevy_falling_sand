use bevy::prelude::*;
use shlex::Shlex;
use std::collections::{HashMap, VecDeque};
use trie_rs::{Trie, TrieBuilder};

#[derive(Clone, Debug, Event)]
pub struct ConsoleCommandEntered {
    pub command_path: Vec<String>,
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

#[derive(Resource)]
pub struct ConsoleConfiguration {
    pub command_tree: HashMap<String, CommandNode>,
    pub history_size: usize,
    pub symbol: String,
}

impl Default for ConsoleConfiguration {
    fn default() -> Self {
        Self {
            command_tree: HashMap::new(),
            history_size: 20,
            symbol: "> ".to_owned(),
        }
    }
}

#[derive(Resource, Default)]
pub struct ConsoleCache {
    pub context_tries: HashMap<Vec<String>, Trie<u8>>,
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
    pub needs_initial_focus: bool,

    pub user_typed_input: String,

    pub in_completion_mode: bool,

    pub needs_cursor_at_end: bool,

    pub request_focus_and_cursor: bool,
}

impl Default for ConsoleState {
    fn default() -> Self {
        let mut state = Self {
            messages: Vec::new(),
            input: String::new(),
            history: VecDeque::from([String::new()]),
            history_index: 0,
            expanded: false,
            height: 300.0,
            suggestions: Vec::new(),
            suggestion_index: None,
            needs_initial_focus: true,
            user_typed_input: String::new(),
            in_completion_mode: false,
            needs_cursor_at_end: false,
            request_focus_and_cursor: false,
        };

        state.add_message("--- Bevy Falling Sand Editor Console ---".to_string());
        state.add_message("Console ready. Type 'help' for available commands.".to_string());

        state
    }
}

pub trait ConsoleCommand: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;

    fn subcommand_types(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![]
    }

    fn execute_action(
        &self,
        _args: &[String],
        _console_writer: &mut EventWriter<PrintConsoleLine>,
        _commands: &mut Commands,
    ) {
    }

    fn execute(
        &self,
        path: &[String],
        args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        let subcommands = self.subcommand_types();

        let mut current_depth = 0;
        for (i, part) in path.iter().enumerate() {
            if part == self.name() {
                current_depth = i;
                break;
            }
        }

        if current_depth + 1 >= path.len() {
            if subcommands.is_empty() {
                self.execute_action(args, console_writer, commands);
            } else {
                console_writer.write(PrintConsoleLine::new(format!(
                    "error: '{}' requires a subcommand",
                    self.name()
                )));
                let subcmd_names: Vec<String> = subcommands
                    .iter()
                    .map(|cmd| cmd.name().to_string())
                    .collect();
                console_writer.write(PrintConsoleLine::new(format!(
                    "Available subcommands: {}",
                    subcmd_names.join(", ")
                )));
            }
            return;
        }

        let next_command = &path[current_depth + 1];
        for subcmd in subcommands {
            if subcmd.name() == next_command {
                subcmd.execute(path, args, console_writer, commands);
                return;
            }
        }

        console_writer.write(PrintConsoleLine::new(format!(
            "error: Unknown subcommand '{} {}'",
            self.name(),
            next_command
        )));
    }

    fn subcommands(&self) -> Vec<Box<dyn ConsoleCommand>> {
        self.subcommand_types()
    }


    fn build_command_node(&self) -> CommandNode {
        let mut node = CommandNode::new(self.name(), self.description());

        let subcommands = self.subcommand_types();
        if subcommands.is_empty() {
            node = node.executable();
        } else {
            for subcmd in subcommands {
                node = node.with_child(subcmd.build_command_node());
            }
        }

        node
    }
}

#[derive(Clone, Debug)]
pub struct CommandNode {
    pub name: String,

    pub description: String,

    pub children: HashMap<String, CommandNode>,

    pub is_executable: bool,
}

impl CommandNode {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            children: HashMap::new(),
            is_executable: false,
        }
    }

    pub fn executable(mut self) -> Self {
        self.is_executable = true;
        self
    }

    pub fn with_child(mut self, child: CommandNode) -> Self {
        self.children.insert(child.name.clone(), child);
        self
    }

    pub fn get_node(&self, path: &[String]) -> Option<&CommandNode> {
        if path.is_empty() {
            return Some(self);
        }

        if let Some(child) = self.children.get(&path[0]) {
            child.get_node(&path[1..])
        } else {
            None
        }
    }

    pub fn get_completions(&self) -> Vec<String> {
        self.children.keys().cloned().collect()
    }
}

impl ConsoleState {
    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }

    pub fn handle_tab_completion(&mut self) {
        if self.suggestions.is_empty() {
            return;
        }

        if !self.in_completion_mode {
            self.user_typed_input = self.input.clone();
            self.in_completion_mode = true;
            self.suggestion_index = Some(0);
        } else {
            if let Some(index) = self.suggestion_index {
                let next_index = (index + 1) % self.suggestions.len();
                self.suggestion_index = Some(next_index);
            } else {
                self.suggestion_index = Some(0);
            }
        }

        if let Some(index) = self.suggestion_index {
            if let Some(suggestion) = self.suggestions.get(index).cloned() {
                self.apply_suggestion(&suggestion);
                self.commit_completion();
                self.input.push(' ');
                self.needs_cursor_at_end = true;
            }
        }
    }

    fn apply_suggestion(&mut self, suggestion: &str) {
        self.input.clear();

        let user_input = &self.user_typed_input;

        if user_input.is_empty() {
            self.input = suggestion.to_string();
            return;
        }

        if user_input.ends_with(' ') {
            self.input = format!("{}{}", user_input, suggestion);
        } else {
            let words: Vec<&str> = user_input.trim().split_whitespace().collect();

            if words.len() == 1 {
                self.input = suggestion.to_string();
            } else {
                let mut complete_words = words[..words.len() - 1].to_vec();
                complete_words.push(suggestion);
                self.input = complete_words.join(" ");
            }
        }
    }

    pub fn commit_completion(&mut self) {
        self.in_completion_mode = false;
        self.user_typed_input.clear();
        self.suggestions.clear();
        self.suggestion_index = None;
        self.needs_cursor_at_end = true;
    }

    pub fn on_input_changed(&mut self) {
        if self.in_completion_mode {
            self.commit_completion();
        }
        self.history_index = 0;
        self.needs_cursor_at_end = false;
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

        let args = Shlex::new(&command).collect::<Vec<_>>();
        if !args.is_empty() {
            let (command_path, remaining_args) = self.find_command_path(&args, config);

            if !command_path.is_empty() {
                if let Some(root_node) = config.command_tree.get(&command_path[0]) {
                    if let Some(node) = root_node.get_node(&command_path[1..]) {
                        if node.is_executable {
                            self.add_message(format!(
                                "Executing command: {}",
                                command_path.join(" ")
                            ));
                            command_writer.write(ConsoleCommandEntered {
                                command_path,
                                args: remaining_args,
                            });
                            return;
                        } else {
                            self.add_message(format!(
                                "Executing command: {}",
                                command_path.join(" ")
                            ));
                            command_writer.write(ConsoleCommandEntered {
                                command_path,
                                args: remaining_args,
                            });
                            return;
                        }
                    }
                }

                let command_name = &command_path[0];
                if config.command_tree.contains_key(command_name) {
                    self.add_message(format!("Executing command: {}", command_name));
                    command_writer.write(ConsoleCommandEntered {
                        command_path: vec![command_name.clone()],
                        args: args[1..].to_vec(),
                    });
                } else {
                    self.add_message(format!("error: Unknown command '{}'", command_name));
                    self.list_available_commands(config);
                }
            } else {
                self.add_message("error: Empty command".to_string());
                self.list_available_commands(config);
            }
        }
    }

    fn find_command_path(
        &self,
        args: &[String],
        config: &ConsoleConfiguration,
    ) -> (Vec<String>, Vec<String>) {
        if args.is_empty() {
            return (vec![], vec![]);
        }

        let first_arg = &args[0];
        if let Some(root_node) = config.command_tree.get(first_arg) {
            let mut path = vec![first_arg.clone()];
            let mut current_node = root_node;
            let mut arg_index = 1;

            while arg_index < args.len() {
                if let Some(child) = current_node.children.get(&args[arg_index]) {
                    path.push(args[arg_index].clone());
                    current_node = child;
                    arg_index += 1;
                } else {
                    break;
                }
            }

            (path, args[arg_index..].to_vec())
        } else {
            (vec![first_arg.clone()], args[1..].to_vec())
        }
    }

    fn list_available_commands(&mut self, config: &ConsoleConfiguration) {
        if !config.command_tree.is_empty() {
            let commands: Vec<String> = config.command_tree.keys().cloned().collect();
            self.add_message(format!("Available commands: {}", commands.join(", ")));
        } else {
            self.add_message("Available commands: help, clear, echo".to_string());
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

    pub fn update_suggestions(&mut self, cache: &ConsoleCache, config: &ConsoleConfiguration) {
        if self.in_completion_mode {
            return;
        }

        self.suggestions.clear();
        self.suggestion_index = None;

        let input_to_analyze = if !self.user_typed_input.is_empty() {
            &self.user_typed_input
        } else {
            &self.input
        };

        if !input_to_analyze.is_empty() {
            let trimmed = input_to_analyze.trim();
            let words: Vec<&str> = trimmed.split_whitespace().collect();

            if words.is_empty() {
                return;
            }

            if words.len() == 1 && !input_to_analyze.ends_with(' ') {
                // Root command suggestions are handled by command_tree below
                let word = words[0];
                self.suggestions = config.command_tree
                    .keys()
                    .filter(|cmd| cmd.starts_with(word))
                    .take(5)
                    .cloned()
                    .collect();
                return;
            }

            let (context_path, partial_word) =
                self.parse_command_context(&words, config, input_to_analyze);
            self.suggestions =
                self.get_context_suggestions(context_path, partial_word, cache, config);
        }
    }

    fn parse_command_context(
        &self,
        words: &[&str],
        config: &ConsoleConfiguration,
        input: &str,
    ) -> (Vec<String>, String) {
        if words.is_empty() {
            return (vec![], String::new());
        }

        let word_strings: Vec<String> = words.iter().map(|s| s.to_string()).collect();

        let input_ends_with_space = input.ends_with(' ');

        if words.len() == 1 && !input_ends_with_space {
            return (vec![], words[0].to_string());
        }

        let first_word = &word_strings[0];
        if let Some(root_node) = config.command_tree.get(first_word) {
            let mut context_path = vec![first_word.clone()];
            let mut current_node = root_node;
            let mut word_index = 1;

            if words.len() == 1 && input_ends_with_space {
                return (context_path, String::new());
            }

            let max_word_index = if input_ends_with_space {
                word_strings.len()
            } else {
                word_strings.len() - 1
            };

            while word_index < max_word_index {
                if let Some(child) = current_node.children.get(&word_strings[word_index]) {
                    context_path.push(word_strings[word_index].clone());
                    current_node = child;
                    word_index += 1;
                } else {
                    break;
                }
            }

            let partial_word = if input_ends_with_space {
                String::new()
            } else if word_index < word_strings.len() {
                word_strings[word_index].clone()
            } else {
                String::new()
            };

            (context_path, partial_word)
        } else {
            (vec![], words[0].to_string())
        }
    }

    fn get_context_suggestions(
        &self,
        context_path: Vec<String>,
        partial_word: String,
        _cache: &ConsoleCache,
        config: &ConsoleConfiguration,
    ) -> Vec<String> {
        let completions = self.get_all_completions_for_context(&context_path, config);

        if context_path.is_empty() {
            if partial_word.is_empty() {
                completions
            } else {
                completions
                    .into_iter()
                    .filter(|s| s.starts_with(&partial_word))
                    .take(5)
                    .collect()
            }
        } else {
            completions
                .into_iter()
                .filter(|s| s.starts_with(&partial_word))
                .take(5)
                .collect()
        }
    }

    fn get_all_completions_for_context(
        &self,
        context_path: &[String],
        config: &ConsoleConfiguration,
    ) -> Vec<String> {
        if context_path.is_empty() {
            return config.command_tree.keys().cloned().collect();
        }

        if let Some(root_node) = config.command_tree.get(&context_path[0]) {
            if let Some(node) = root_node.get_node(&context_path[1..]) {
                return node.get_completions();
            }
        }

        vec![]
    }
}

impl ConsoleConfiguration {
    pub fn register_command<T: ConsoleCommand + Default>(&mut self) {
        let command = T::default();
        let name = command.name().to_string();
        let command_node = command.build_command_node();
        self.command_tree.insert(name, command_node);
    }
}

#[derive(Resource, Default)]
pub struct CommandRegistry {
    pub commands: Vec<Box<dyn ConsoleCommand>>,
}

impl CommandRegistry {
    pub fn register<T: ConsoleCommand + Default>(&mut self) {
        self.commands.push(Box::new(CommandWrapper::<T>::new()));
    }

    pub fn find_command(&self, name: &str) -> Option<&dyn ConsoleCommand> {
        self.commands
            .iter()
            .find(|cmd| cmd.name() == name)
            .map(|cmd| cmd.as_ref())
    }
}

struct CommandWrapper<T: ConsoleCommand> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: ConsoleCommand> CommandWrapper<T> {
    fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: ConsoleCommand + Default> ConsoleCommand for CommandWrapper<T> {
    fn name(&self) -> &'static str {
        T::default().name()
    }

    fn description(&self) -> &'static str {
        T::default().description()
    }

    fn execute(
        &self,
        path: &[String],
        args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        T::default().execute(path, args, console_writer, commands);
    }

    fn subcommands(&self) -> Vec<Box<dyn ConsoleCommand>> {
        T::default().subcommands()
    }


    fn build_command_node(&self) -> CommandNode {
        T::default().build_command_node()
    }
}

pub fn command_handler(
    mut cmd: EventReader<ConsoleCommandEntered>,
    mut console_writer: EventWriter<PrintConsoleLine>,
    registry: Res<CommandRegistry>,
    mut commands: Commands,
) {
    for command_event in cmd.read() {
        if command_event.command_path.is_empty() {
            continue;
        }

        let root_command_name = &command_event.command_path[0];
        if let Some(command) = registry.find_command(root_command_name) {
            command.execute(
                &command_event.command_path,
                &command_event.args,
                &mut console_writer,
                &mut commands,
            );
        }
    }
}

impl ConsoleCache {
    pub fn rebuild_tries(&mut self, config: &ConsoleConfiguration) {
        self.context_tries.clear();
        self.build_context_tries_recursive(&vec![], config);
    }

    fn build_context_tries_recursive(
        &mut self,
        current_path: &[String],
        config: &ConsoleConfiguration,
    ) {
        let completions = self.get_context_completions(current_path, config);
        if !completions.is_empty() {
            let mut builder: TrieBuilder<u8> = TrieBuilder::new();
            for completion in &completions {
                builder.push(completion.as_bytes());
            }
            self.context_tries
                .insert(current_path.to_vec(), builder.build());
        }

        for completion in completions {
            let mut next_path = current_path.to_vec();
            next_path.push(completion);
            self.build_context_tries_recursive(&next_path, config);
        }
    }

    fn get_context_completions(
        &self,
        context_path: &[String],
        config: &ConsoleConfiguration,
    ) -> Vec<String> {
        if context_path.is_empty() {
            config.command_tree.keys().cloned().collect()
        } else {
            if let Some(root_node) = config.command_tree.get(&context_path[0]) {
                if let Some(node) = root_node.get_node(&context_path[1..]) {
                    return node.get_completions();
                }
            }
            vec![]
        }
    }
}
