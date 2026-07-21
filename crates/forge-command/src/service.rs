use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{Command, CommandError, CommandHandle, CommandRegistry};

pub struct CommandService {
    registry: Arc<RwLock<CommandRegistry>>,
}

impl CommandService {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RwLock::new(CommandRegistry::new())),
        }
    }

    pub async fn register<C>(&self, command: C) -> Result<(), CommandError>
    where
        C: Command + 'static,
    {
        self.registry.write().await.register(command)
    }

    pub async fn execute(&self, command_id: &str) -> Result<(), CommandError> {
        let command = {
            let registry = self.registry.read().await;
            registry.get(command_id)?
        };

        command.execute()
    }

    pub fn handle(&self) -> CommandHandle {
        CommandHandle::new(Arc::clone(&self.registry))
    }
}

impl Default for CommandService {
    fn default() -> Self {
        Self::new()
    }
}
