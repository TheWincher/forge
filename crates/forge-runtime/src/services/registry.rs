use forge_config::Config;

use crate::services::{
    command::{CommandHandle, CommandService},
    config::{ConfigHandle, ConfigService},
    plugin::PluginService,
    workspace::{WorkspaceHandle, WorkspaceService},
};

pub struct ServiceRegistry {
    workspace: WorkspaceService,
    command: CommandService,
    plugin: PluginService,
    config: ConfigService,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        let config = Config::load();

        Self {
            workspace: WorkspaceService::new(&config),
            command: CommandService::new(),
            plugin: PluginService::new(),
            config: ConfigService::new(config),
        }
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
