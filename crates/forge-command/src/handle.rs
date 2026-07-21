use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{Command, CommandError, CommandRegistry};

#[derive(Clone)]
pub struct CommandHandle {
    registry: Arc<RwLock<CommandRegistry>>,
}

impl CommandHandle {
    pub(crate) fn new(registry: Arc<RwLock<CommandRegistry>>) -> Self {
        Self { registry }
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

    pub async fn contains(&self, command_id: &str) -> bool {
        self.registry.read().await.contains(command_id)
    }

    pub async fn len(&self) -> usize {
        self.registry.read().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.registry.read().await.is_empty()
    }
}
