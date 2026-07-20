use crate::{handle::RuntimeHandle, services::registry::ServiceRegistry};

#[derive(Clone)]
pub struct RuntimeContext<'a> {
    handle: RuntimeHandle,
    services: &'a ServiceRegistry,
}

impl<'a> RuntimeContext<'a> {
    pub(crate) fn new(handle: RuntimeHandle, services: &'a ServiceRegistry) -> Self {
        Self { handle, services }
    }

    pub fn handle(&self) -> &RuntimeHandle {
        &self.handle
    }

    pub fn services(&self) -> &ServiceRegistry {
        &self.services
    }

    pub fn services_mut(&mut self) -> &mut ServiceRegistry {
        self.services_mut()
    }
}
