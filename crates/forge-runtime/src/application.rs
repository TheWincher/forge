pub struct ServiceRegistry {
    workspace: WorkspaceService,
    command: CommandService,
    plugin: PluginService,
}

pub struct Application {
    services: ServiceRegistry,
}

impl Application {
    pub fn new() -> Self {
        Self {}
    }
}
