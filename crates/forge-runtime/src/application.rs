use forge_config::Config;
use thiserror::Error;

use crate::{
    context::RuntimeContext,
    plugin::registrar::PluginRegistrar,
    services::{
        plugin::PluginServiceError,
        registry::{ServiceRegistry, ServiceRegistryError},
    },
};

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("plugin service error")]
    PluginService(#[from] PluginServiceError),

    #[error("service registry error")]
    ServiceRegistry(#[from] ServiceRegistryError),
}

pub struct Application {
    services: ServiceRegistry,
}

impl Application {
    pub fn new<R>(config: Config, registrar: &R) -> Result<Self, ApplicationError>
    where
        R: PluginRegistrar + ?Sized,
    {
        let services = ServiceRegistry::new(config, registrar)?;

        Ok(Self { services })
    }

    pub fn services(&self) -> &ServiceRegistry {
        &self.services
    }

    pub fn services_mut(&mut self) -> &mut ServiceRegistry {
        &mut self.services
    }

    pub fn start(&mut self, context: &RuntimeContext) -> Result<(), ApplicationError> {
        self.services.plugin_mut().init_all(context)?;

        Ok(())
    }

    pub fn stop(&mut self, context: &RuntimeContext) -> Result<(), ApplicationError> {
        self.services.plugin_mut().shutdown_all(context)?;

        Ok(())
    }
}
