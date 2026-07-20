use forge_config::Config;

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
}

impl ServiceRegistry {
    pub fn new<R>(config: Config, registrar: &R) -> Result<Self, ServiceRegistryError>
    where
        R: PluginRegistrar + ?Sized,
    {
        let mut plugin = PluginService::new();
        registrar.register(&mut plugin)?;

        Ok(Self {
            workspace: WorkspaceService::new(&config),
            command: CommandService::new(),
            config: ConfigService::new(config),
            plugin,
        })
    }

    pub fn handle(&self) -> ServiceRegistryHandle {
        ServiceRegistryHandle {
            workspace: self.workspace.handle(),
            command: self.command.handle(),
            config: self.config.handle(),
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
}

#[derive(Clone)]
pub struct ServiceRegistryHandle {
    workspace: WorkspaceHandle,
    command: CommandHandle,
    config: ConfigHandle,
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
}
