use crate::{Command, CommandError, CommandHandle, CommandRegistry};

pub struct CommandService {
    registry: CommandRegistry,
}

impl CommandService {
    pub fn new() -> Self {
        Self {
            registry: CommandRegistry::new(),
        }
    }

    pub fn register<C>(&mut self, command: C) -> Result<(), CommandError>
    where
        C: Command + 'static,
    {
        self.registry.register(command)
    }

    pub fn execute(&self, command_id: &str) -> Result<(), CommandError> {
        self.registry.execute(command_id)
    }

    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }

    pub fn handle(&self) -> CommandHandle {
        CommandHandle {}
    }
}

impl Default for CommandService {
    fn default() -> Self {
        Self::new()
    }
}
