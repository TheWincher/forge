use crate::{handle::RuntimeHandle, services::registry::ServiceRegistryHandle};

#[derive(Clone)]
pub struct RuntimeContext {
    handle: RuntimeHandle,
    services: ServiceRegistryHandle,
}

impl RuntimeContext {
    pub(crate) fn new(handle: RuntimeHandle, services: ServiceRegistryHandle) -> Self {
        Self { handle, services }
    }

    pub fn handle(&self) -> &RuntimeHandle {
        &self.handle
    }

    pub fn services(&self) -> &ServiceRegistryHandle {
        &self.services
    }
}
