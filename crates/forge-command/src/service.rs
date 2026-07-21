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
        self.registry.read().await.execute(command_id)
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{CommandDescriptor, CommandError};

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

    #[tokio::test]
    async fn cloned_handles_share_commands() {
        let registry = Arc::new(RwLock::new(CommandRegistry::new()));

        let first = CommandHandle::new(Arc::clone(&registry));
        let second = first.clone();

        first.register(TestCommand).await.unwrap();

        assert!(second.contains("test.command").await);
        assert_eq!(second.len().await, 1);
    }

    #[tokio::test]
    async fn executes_command_from_handle() {
        let registry = Arc::new(RwLock::new(CommandRegistry::new()));
        let handle = CommandHandle::new(registry);

        handle.register(TestCommand).await.unwrap();

        assert!(handle.execute("test.command").await.is_ok());
    }
}
