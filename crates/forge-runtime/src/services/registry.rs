use forge_config::Config;
use forge_event::{EventHandle, EventService};

use crate::{
    plugin::registrar::PluginRegistrar,
    services::{
        command::{CommandHandle, CommandService},
        config::{ConfigHandle, ConfigService},
        plugin::{PluginService, PluginServiceError},
        workspace::{WorkspaceHandle, WorkspaceService},
    },
};

#[derive(Debug, thiserror::Error)]
pub enum ServiceRegistryError {
    #[error("plugin service error")]
    Plugin(#[from] PluginServiceError),
}

pub struct ServiceRegistry {
    workspace: WorkspaceService,
    command: CommandService,
    plugin: PluginService,
    config: ConfigService,
    event: EventService,
}

impl ServiceRegistry {
    pub fn new<R>(config: Config, registrar: &R) -> Result<Self, ServiceRegistryError>
    where
        R: PluginRegistrar + ?Sized,
    {
        let mut plugin = PluginService::new();
        registrar.register(&mut plugin)?;
        plugin.validate_dependencies()?;

        Ok(Self {
            workspace: WorkspaceService::new(&config),
            command: CommandService::new(),
            config: ConfigService::new(config),
            event: EventService::new(),
            plugin,
        })
    }

    pub fn handle(&self) -> ServiceRegistryHandle {
        ServiceRegistryHandle {
            workspace: self.workspace.handle(),
            command: self.command.handle(),
            config: self.config.handle(),
            event: self.event.handle(),
        }
    }

    pub fn plugin(&self) -> &PluginService {
        &self.plugin
    }

    pub fn plugin_mut(&mut self) -> &mut PluginService {
        &mut self.plugin
    }

    pub fn workspace(&self) -> &WorkspaceService {
        &self.workspace
    }

    pub fn workspace_mut(&mut self) -> &mut WorkspaceService {
        &mut self.workspace
    }

    pub fn config(&self) -> &ConfigService {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut ConfigService {
        &mut self.config
    }

    pub fn command(&self) -> &CommandService {
        &self.command
    }

    pub fn command_mut(&mut self) -> &mut CommandService {
        &mut self.command
    }

    pub fn event(&self) -> &EventService {
        &self.event
    }

    pub fn event_mut(&mut self) -> &mut EventService {
        &mut self.event
    }
}

#[derive(Clone)]
pub struct ServiceRegistryHandle {
    workspace: WorkspaceHandle,
    command: CommandHandle,
    config: ConfigHandle,
    event: EventHandle,
}

impl ServiceRegistryHandle {
    pub fn workspace(&self) -> &WorkspaceHandle {
        &self.workspace
    }

    pub fn command(&self) -> &CommandHandle {
        &self.command
    }

    pub fn config(&self) -> &ConfigHandle {
        &self.config
    }

    pub fn event(&self) -> &EventHandle {
        &self.event
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use forge_config::Config;

    use super::*;
    use crate::{
        plugin::registrar::PluginRegistrar,
        services::plugin::{PluginService, PluginServiceError},
    };

    struct EmptyPluginRegistrar;

    impl PluginRegistrar for EmptyPluginRegistrar {
        fn register(&self, _plugins: &mut PluginService) -> Result<(), PluginServiceError> {
            Ok(())
        }
    }

    fn create_registry() -> ServiceRegistry {
        ServiceRegistry::new(Config::default(), &EmptyPluginRegistrar)
            .expect("service registry should be created")
    }

    #[test]
    fn creates_service_registry() {
        let registry = create_registry();

        let _workspace = registry.workspace();
        let _command = registry.command();
        let _event = registry.event();
        let _config = registry.config();
        let _plugin = registry.plugin();
    }

    #[test]
    fn creates_service_registry_handle() {
        let registry = create_registry();

        let handle = registry.handle();

        let _workspace = handle.workspace();
        let _command = handle.command();
        let _event = handle.event();
        let _config = handle.config();
    }

    #[test]
    fn event_handles_share_the_same_event_bus() {
        #[derive(Debug)]
        struct TestEvent;

        let registry = create_registry();

        let first_handle = registry.handle();
        let second_handle = registry.handle();

        let calls = Arc::new(AtomicUsize::new(0));
        let listener_calls = Arc::clone(&calls);

        first_handle
            .event()
            .subscribe::<TestEvent, _>(move |_: &TestEvent| {
                listener_calls.fetch_add(1, Ordering::SeqCst);
            });

        second_handle.event().publish(&TestEvent);

        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn registry_event_service_and_handle_share_the_same_event_bus() {
        #[derive(Debug)]
        struct TestEvent;

        let registry = create_registry();
        let handle = registry.handle();

        let calls = Arc::new(AtomicUsize::new(0));
        let listener_calls = Arc::clone(&calls);

        registry
            .event()
            .subscribe::<TestEvent, _>(move |_: &TestEvent| {
                listener_calls.fetch_add(1, Ordering::SeqCst);
            });

        handle.event().publish(&TestEvent);

        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn event_subscription_is_visible_from_new_handles() {
        #[derive(Debug)]
        struct TestEvent;

        let registry = create_registry();

        let calls = Arc::new(AtomicUsize::new(0));
        let listener_calls = Arc::clone(&calls);

        registry
            .handle()
            .event()
            .subscribe::<TestEvent, _>(move |_: &TestEvent| {
                listener_calls.fetch_add(1, Ordering::SeqCst);
            });

        let new_handle = registry.handle();
        new_handle.event().publish(&TestEvent);

        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
