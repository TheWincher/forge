use std::{collections::HashMap, sync::Arc};

use crate::{Command, CommandDescriptor, CommandError};

pub struct CommandRegistry {
    commands: HashMap<&'static str, Arc<dyn Command>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn register<C>(&mut self, command: C) -> Result<(), CommandError>
    where
        C: Command + 'static,
    {
        let command_id = command.descriptor().id();

        if self.commands.contains_key(command_id) {
            return Err(CommandError::DuplicateCommandId { command_id });
        }

        self.commands.insert(command_id, Arc::new(command));

        Ok(())
    }

    pub fn execute(&self, command_id: &str) -> Result<(), CommandError> {
        let command =
            self.commands
                .get(command_id)
                .ok_or_else(|| CommandError::CommandNotFound {
                    command_id: command_id.to_owned(),
                })?;

        command.execute()
    }

    pub fn contains(&self, command_id: &str) -> bool {
        self.commands.contains_key(command_id)
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &'static CommandDescriptor> + '_ {
        self.commands.values().map(|command| command.descriptor())
    }

    pub fn get(&self, command_id: &str) -> Result<Arc<dyn Command>, CommandError> {
        self.commands
            .get(command_id)
            .cloned()
            .ok_or_else(|| CommandError::CommandNotFound {
                command_id: command_id.to_owned(),
            })
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DESCRIPTOR: CommandDescriptor =
        CommandDescriptor::new("test.command", "Test command");

    struct TestCommand;

    impl Command for TestCommand {
        fn descriptor(&self) -> &'static CommandDescriptor {
            &TEST_DESCRIPTOR
        }

        fn execute(&self) -> Result<(), CommandError> {
            Ok(())
        }
    }

    #[test]
    fn registers_command() {
        let mut registry = CommandRegistry::new();

        registry.register(TestCommand).unwrap();

        assert_eq!(registry.len(), 1);
        assert!(registry.contains("test.command"));
    }

    #[test]
    fn rejects_duplicate_command_ids() {
        let mut registry = CommandRegistry::new();

        registry.register(TestCommand).unwrap();

        let error = registry.register(TestCommand).unwrap_err();

        assert!(matches!(
            error,
            CommandError::DuplicateCommandId {
                command_id: "test.command",
            }
        ));
    }

    #[test]
    fn executes_registered_command() {
        let mut registry = CommandRegistry::new();

        registry.register(TestCommand).unwrap();

        assert!(registry.execute("test.command").is_ok());
    }

    #[test]
    fn rejects_unknown_command() {
        let registry = CommandRegistry::new();

        let error = registry.execute("unknown.command").unwrap_err();

        assert!(matches!(
            error,
            CommandError::CommandNotFound { command_id }
                if command_id == "unknown.command"
        ));
    }

    #[test]
    fn exposes_descriptors() {
        let mut registry = CommandRegistry::new();

        registry.register(TestCommand).unwrap();

        let descriptors = registry.descriptors().collect::<Vec<_>>();

        assert_eq!(descriptors, vec![&TEST_DESCRIPTOR]);
    }
}
