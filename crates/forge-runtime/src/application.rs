use forge_config::Config;
use forge_workspace::Workspace;

use crate::{plugin::Plugin, services::registry::ServiceRegistry};

pub struct Application {
    services: ServiceRegistry,
    config: Config,
    workspace: Option<Workspace>,
    plugins: Vec<Box<dyn Plugin>>,
}

impl Application {
    pub fn new() -> Self {
        let config = Config::load();
        let workspace =
            config
                .workspace_root
                .clone()
                .and_then(|root| match Workspace::open(root) {
                    Ok(workspace) => Some(workspace),
                    Err(error) => {
                        tracing::warn!(%error, "Failed to open workspace");
                        None
                    }
                });

        Self {
            services: ServiceRegistry::new(),
            config,
            workspace,
            plugins: vec![],
        }
    }

    pub fn services(&self) -> &ServiceRegistry {
        &self.services
    }

    pub fn services_mut(&mut self) -> &mut ServiceRegistry {
        &mut self.services
    }
}
