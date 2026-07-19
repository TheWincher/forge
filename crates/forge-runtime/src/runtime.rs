use crate::{application::Application, task_manager::TaskManager};

pub struct Runtime {
    app: Application,
    task_manager: TaskManager,
    tokio_runtime: tokio::runtime::Runtime,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            app: Application::new(),
            task_manager: TaskManager::new(),
            tokio_runtime: tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"),
        }
    }

    pub fn run(&self) {}

    pub fn shutdown(&self) {}
}
