use crate::{
    handle::RuntimeHandle, services::registry::ServiceRegistryHandle, task_manager::TaskHandle,
};

#[derive(Clone)]
pub struct RuntimeContext {
    handle: RuntimeHandle,
    services: ServiceRegistryHandle,
    tasks: TaskHandle,
}

impl RuntimeContext {
    pub(crate) fn new(
        handle: RuntimeHandle,
        services: ServiceRegistryHandle,
        task_handle: TaskHandle,
    ) -> Self {
        Self {
            handle,
            services,
            tasks: task_handle,
        }
    }

    pub fn handle(&self) -> &RuntimeHandle {
        &self.handle
    }

    pub fn services(&self) -> &ServiceRegistryHandle {
        &self.services
    }

    pub fn tasks(&self) -> &TaskHandle {
        &self.tasks
    }
}
