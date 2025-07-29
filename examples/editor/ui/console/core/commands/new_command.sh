#!/bin/bash

# Check if command name is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <command_name>"
    echo "Example: $0 spawn"
    exit 1
fi

# Convert command name to lowercase for consistency
COMMAND_NAME=$(echo "$1" | tr '[:upper:]' '[:lower:]')

# Convert command name to PascalCase for struct names
PASCAL_CASE=$(echo "$COMMAND_NAME" | sed 's/^\(.\)/\U\1/g; s/_\(.\)/\U\1/g; s/_//g')

# Create a clean plugin name (remove "Command" if it's already in the name)
PLUGIN_NAME=$(echo "$PASCAL_CASE" | sed 's/Command$//')CommandPlugin

# Create the module file
MODULE_FILE="${COMMAND_NAME}.rs"

# Check if file already exists
if [ -f "$MODULE_FILE" ]; then
    echo "Error: File '$MODULE_FILE' already exists"
    exit 1
fi

# Generate the boilerplate code
cat > "$MODULE_FILE" << EOF
use bevy::prelude::*;

use super::super::core::{ConsoleCommand, PrintConsoleLine};

pub struct ${PLUGIN_NAME};

impl Plugin for ${PLUGIN_NAME} {
    fn build(&self, _app: &mut App) {
        // Add observers for command events here
    }
}

#[derive(Default)]
pub struct ${PASCAL_CASE}Command;

impl ConsoleCommand for ${PASCAL_CASE}Command {
    fn name(&self) -> &'static str {
        "${COMMAND_NAME}"
    }

    fn description(&self) -> &'static str {
        "TODO: Add description for ${COMMAND_NAME} command"
    }

    fn subcommand_types(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![
            Box::new(${PASCAL_CASE}SubCommand1),
            Box::new(${PASCAL_CASE}SubCommand2),
        ]
    }
}

#[derive(Default)]
pub struct ${PASCAL_CASE}SubCommand1;

impl ConsoleCommand for ${PASCAL_CASE}SubCommand1 {
    fn name(&self) -> &'static str {
        "subcommand1"
    }

    fn description(&self) -> &'static str {
        "TODO: Add description for subcommand1"
    }

    fn subcommand_types(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![
            Box::new(${PASCAL_CASE}SubCommand1Action1),
            Box::new(${PASCAL_CASE}SubCommand1Action2),
        ]
    }
}

#[derive(Default)]
pub struct ${PASCAL_CASE}SubCommand2;

impl ConsoleCommand for ${PASCAL_CASE}SubCommand2 {
    fn name(&self) -> &'static str {
        "subcommand2"
    }

    fn description(&self) -> &'static str {
        "TODO: Add description for subcommand2"
    }

    fn subcommand_types(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![
            Box::new(${PASCAL_CASE}SubCommand2Action1),
            Box::new(${PASCAL_CASE}SubCommand2Action2),
        ]
    }
}

#[derive(Default)]
pub struct ${PASCAL_CASE}SubCommand1Action1;

impl ConsoleCommand for ${PASCAL_CASE}SubCommand1Action1 {
    fn name(&self) -> &'static str {
        "action1"
    }

    fn description(&self) -> &'static str {
        "TODO: Add description for action1"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Executing ${COMMAND_NAME} subcommand1 action1...".to_string(),
        ));
        // TODO: Implement action logic here
    }
}

#[derive(Default)]
pub struct ${PASCAL_CASE}SubCommand1Action2;

impl ConsoleCommand for ${PASCAL_CASE}SubCommand1Action2 {
    fn name(&self) -> &'static str {
        "action2"
    }

    fn description(&self) -> &'static str {
        "TODO: Add description for action2"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Executing ${COMMAND_NAME} subcommand1 action2...".to_string(),
        ));
        // TODO: Implement action logic here
    }
}

#[derive(Default)]
pub struct ${PASCAL_CASE}SubCommand2Action1;

impl ConsoleCommand for ${PASCAL_CASE}SubCommand2Action1 {
    fn name(&self) -> &'static str {
        "action1"
    }

    fn description(&self) -> &'static str {
        "TODO: Add description for action1"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Executing ${COMMAND_NAME} subcommand2 action1...".to_string(),
        ));
        // TODO: Implement action logic here
    }
}

#[derive(Default)]
pub struct ${PASCAL_CASE}SubCommand2Action2;

impl ConsoleCommand for ${PASCAL_CASE}SubCommand2Action2 {
    fn name(&self) -> &'static str {
        "action2"
    }

    fn description(&self) -> &'static str {
        "TODO: Add description for action2"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Executing ${COMMAND_NAME} subcommand2 action2...".to_string(),
        ));
        // TODO: Implement action logic here
    }
}
EOF

# Add the module to mod.rs
MOD_FILE="mod.rs"
if [ -f "$MOD_FILE" ]; then
    # Check if the module is already added
    if grep -q "pub mod ${COMMAND_NAME};" "$MOD_FILE"; then
        echo "Module '${COMMAND_NAME}' already exists in mod.rs"
    else
        # Find the last pub mod line and add after it
        LAST_MOD_LINE=$(grep -n "^pub mod" "$MOD_FILE" | tail -1 | cut -d: -f1)
        if [ -n "$LAST_MOD_LINE" ]; then
            sed -i "${LAST_MOD_LINE}a\\pub mod ${COMMAND_NAME};" "$MOD_FILE"
            echo "Added 'pub mod ${COMMAND_NAME};' to mod.rs"
        else
            # If no pub mod lines found, add at the beginning
            sed -i "1i\\pub mod ${COMMAND_NAME};" "$MOD_FILE"
            echo "Added 'pub mod ${COMMAND_NAME};' to mod.rs"
        fi
    fi

    # Add the use statement for the plugin
    if grep -q "use ${COMMAND_NAME}::${PLUGIN_NAME};" "$MOD_FILE"; then
        echo "Use statement for '${PLUGIN_NAME}' already exists in mod.rs"
    else
        # Find the last use statement and add after it
        LAST_USE_LINE=$(grep -n "^use.*CommandPlugin;" "$MOD_FILE" | tail -1 | cut -d: -f1)
        if [ -n "$LAST_USE_LINE" ]; then
            sed -i "${LAST_USE_LINE}a\\use ${COMMAND_NAME}::${PLUGIN_NAME};" "$MOD_FILE"
            echo "Added use statement for '${PLUGIN_NAME}' to mod.rs"
        else
            # If no use statements found, add after the pub mod statements
            LAST_MOD_LINE=$(grep -n "^pub mod" "$MOD_FILE" | tail -1 | cut -d: -f1)
            if [ -n "$LAST_MOD_LINE" ]; then
                sed -i "${LAST_MOD_LINE}a\\\\nuse ${COMMAND_NAME}::${PLUGIN_NAME};" "$MOD_FILE"
                echo "Added use statement for '${PLUGIN_NAME}' to mod.rs"
            fi
        fi
    fi

    # Add the plugin to the plugin list in ConsoleCommandsPlugin
    if grep -q "${PLUGIN_NAME}," "$MOD_FILE"; then
        echo "Plugin '${PLUGIN_NAME}' already exists in ConsoleCommandsPlugin"
    else
        # Find the last plugin in the add_plugins call and add after it
        LAST_PLUGIN_LINE=$(grep -n ".*CommandPlugin,$" "$MOD_FILE" | tail -1 | cut -d: -f1)
        if [ -n "$LAST_PLUGIN_LINE" ]; then
            sed -i "${LAST_PLUGIN_LINE}a\\            ${PLUGIN_NAME}," "$MOD_FILE"
            echo "Added '${PLUGIN_NAME}' to ConsoleCommandsPlugin"
        else
            echo "Warning: Could not find plugin list in ConsoleCommandsPlugin"
        fi
    fi

    # Add the command import to the use statement in init_command_registry
    if grep -q "${COMMAND_NAME}::\*" "$MOD_FILE"; then
        echo "Import for '${COMMAND_NAME}' already exists in init_command_registry"
    else
        # Find the use commands line and add the new command
        USE_COMMANDS_LINE=$(grep -n "use commands::" "$MOD_FILE" | cut -d: -f1)
        if [ -n "$USE_COMMANDS_LINE" ]; then
            # Get the current use statement and add the new command
            CURRENT_USE=$(sed -n "${USE_COMMANDS_LINE}p" "$MOD_FILE")
            # Add the new command before the closing brace and semicolon
            NEW_USE=$(echo "$CURRENT_USE" | sed "s/};/, ${COMMAND_NAME}::\*};/")
            sed -i "${USE_COMMANDS_LINE}s/.*/    $NEW_USE/" "$MOD_FILE"
            echo "Added '${COMMAND_NAME}::*' to commands import"
        else
            echo "Warning: Could not find commands use statement"
        fi
    fi

    # Add registry.register call
    if grep -q "registry.register::<${PASCAL_CASE}Command>();" "$MOD_FILE"; then
        echo "Registry registration for '${PASCAL_CASE}Command' already exists"
    else
        # Find the last registry.register line and add after it
        LAST_REGISTRY_LINE=$(grep -n "registry.register::<.*Command>();" "$MOD_FILE" | tail -1 | cut -d: -f1)
        if [ -n "$LAST_REGISTRY_LINE" ]; then
            sed -i "${LAST_REGISTRY_LINE}a\\    registry.register::<${PASCAL_CASE}Command>();" "$MOD_FILE"
            echo "Added registry registration for '${PASCAL_CASE}Command'"
        else
            echo "Warning: Could not find registry registration section"
        fi
    fi

    # Add config.register_command call
    if grep -q "config.register_command::<${PASCAL_CASE}Command>();" "$MOD_FILE"; then
        echo "Config registration for '${PASCAL_CASE}Command' already exists"
    else
        # Find the last config.register_command line and add after it
        LAST_CONFIG_LINE=$(grep -n "config.register_command::<.*Command>();" "$MOD_FILE" | tail -1 | cut -d: -f1)
        if [ -n "$LAST_CONFIG_LINE" ]; then
            sed -i "${LAST_CONFIG_LINE}a\\    config.register_command::<${PASCAL_CASE}Command>();" "$MOD_FILE"
            echo "Added config registration for '${PASCAL_CASE}Command'"
        else
            echo "Warning: Could not find config registration section"
        fi
    fi
else
    echo "Warning: mod.rs not found in current directory"
fi

echo "Successfully created ${MODULE_FILE} with boilerplate code"
echo "Command structure:"
echo "  ${COMMAND_NAME}"
echo "       subcommand1"
echo "          action1"
echo "          action2"
echo "       subcommand2"
echo "           action1"
echo "           action2"