use crate::{application::Application, error::RuntimeError, task_manager::TaskManager};

pub struct Runtime {
    app: Application,
    task_manager: TaskManager,
    tokio_runtime: tokio::runtime::Runtime,
}

impl Runtime {
    pub fn new() -> Result<Self, RuntimeError> {
        let runtime = Self {
            app: Application::new(),
            task_manager: TaskManager::new(),
            tokio_runtime: tokio::runtime::Runtime::new()
                .map_err(RuntimeError::TokioInitializationFailed)?,
        };

        Ok(runtime)
    }

    pub fn run(&self) {}

    pub fn shutdown(&self) {}
}
