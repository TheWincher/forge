use crate::{context::RuntimeContext, error::RuntimeError, services::registry::ServiceRegistry};

pub struct Application {
    services: ServiceRegistry,
}

impl Application {
    pub fn new() -> Self {
        Self {
            services: ServiceRegistry::new(),
        }
    }

    pub fn services(&self) -> &ServiceRegistry {
        &self.services
    }

    pub fn services_mut(&mut self) -> &mut ServiceRegistry {
        &mut self.services
    }

    pub fn start(&mut self, context: &RuntimeContext) -> Result<(), RuntimeError> {
        self.services.plugin_mut().init_all(context);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }
}
