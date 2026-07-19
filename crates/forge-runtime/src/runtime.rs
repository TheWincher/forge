use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{
    application::Application, error::RuntimeError, event::AppEvent, handle::RuntimeHandle,
    task_manager::TaskManager,
};

pub struct Runtime {
    app: Application,
    task_manager: TaskManager,
    tokio_runtime: tokio::runtime::Runtime,
    event_sender: Sender<AppEvent>,
    event_receiver: Receiver<AppEvent>,
}

impl Runtime {
    pub fn new() -> Result<Self, RuntimeError> {
        let tokio_runtime =
            tokio::runtime::Runtime::new().map_err(RuntimeError::TokioInitializationFailed)?;

        let (sender, receiver) = mpsc::channel::<AppEvent>(100);

        let runtime = Self {
            app: Application::new(),
            task_manager: TaskManager::new(),
            tokio_runtime,
            event_sender: sender,
            event_receiver: receiver,
        };

        Ok(runtime)
    }

    pub fn run(&self) {}

    pub fn shutdown(&self) {}

    pub fn handle(&self) -> RuntimeHandle {
        RuntimeHandle::new(&self.event_sender)
    }
}
