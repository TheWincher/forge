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
