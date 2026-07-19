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

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        self.tokio_runtime
            .block_on(Self::event_loop(&mut self.event_receiver))
    }

    pub fn handle(&self) -> RuntimeHandle {
        RuntimeHandle::new(&self.event_sender)
    }

    async fn event_loop(receiver: &mut Receiver<AppEvent>) -> Result<(), RuntimeError> {
        loop {
            let event_opt = receiver.recv().await;
            if let Some(event) = event_opt {
                match event {
                    AppEvent::ShutdownRequested => {
                        break;
                    }
                }
            } else {
                tracing::warn!("Event channel closed");
                break;
            }
        }

        Ok(())
    }
}
