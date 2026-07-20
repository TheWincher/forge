use crate::services::{
    command::CommandService, config::ConfigService, plugin::PluginService,
    workspace::WorkspaceService,
};

// pub struct ServiceRegistryHandle {
//     workspace: WorkspaceHandle,
//     command: CommandHandle,
//     config: ConfigHandle,
// }

pub struct ServiceRegistry {
    workspace: WorkspaceService,
    command: CommandService,
    plugin: PluginService,
    config: ConfigService,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            workspace: WorkspaceService::new(),
            command: CommandService::new(),
            plugin: PluginService::new(),
            config: ConfigService::new(),
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
