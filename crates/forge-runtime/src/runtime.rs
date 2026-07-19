use std::time::Duration;

use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{
    application::Application, error::RuntimeError, event::AppEvent, handle::RuntimeHandle,
    lifecycle::RuntimeState, signal::wait_for_shutdown, task_manager::TaskManager,
};

enum RuntimeAction {
    Continue,
    Stop,
}

pub struct Runtime {
    state: RuntimeState,
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

        Ok(Self {
            state: RuntimeState::Created,
            app: Application::new(),
            task_manager: TaskManager::new(),
            tokio_runtime,
            event_sender: sender,
            event_receiver: receiver,
        })
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        self.transition_to(RuntimeState::Starting);
        self.enable_signal_handler();
        self.transition_to(RuntimeState::Running);

        let event_loop = Self::event_loop(&mut self.event_receiver);
        self.tokio_runtime.block_on(event_loop)?;

        self.transition_to(RuntimeState::Stopping);

        let shutdown = self.task_manager.shutdown(Duration::from_secs(5));
        self.tokio_runtime.block_on(shutdown);

        self.transition_to(RuntimeState::Stopped);

        Ok(())
    }

    pub fn handle(&self) -> RuntimeHandle {
        RuntimeHandle::new(&self.event_sender)
    }

    async fn event_loop(receiver: &mut Receiver<AppEvent>) -> Result<(), RuntimeError> {
        loop {
            let event = receiver.recv().await;

            let Some(event) = event else {
                tracing::warn!("Event channel closed");
                return Ok(());
            };

            match Self::handle_event(event) {
                RuntimeAction::Continue => continue,
                RuntimeAction::Stop => return Ok(()),
            }
        }
    }

    fn handle_event(event: AppEvent) -> RuntimeAction {
        match event {
            AppEvent::ShutdownRequested => {
                tracing::info!("Shutdown requested");
                RuntimeAction::Stop
            }
            _ => RuntimeAction::Continue,
        }
    }

    fn enable_signal_handler(&mut self) {
        let handle = self.handle();

        self.task_manager.spawn(async move {
            if let Err(error) = wait_for_shutdown(handle).await {
                tracing::error!(?error, "Signal handler failed");
            }
        });
    }

    fn transition_to(&mut self, state: RuntimeState) {
        tracing::info!(
            from = ?self.state,
            to = ?state,
            "Runtime state changed"
        );

        self.state = state;
    }
}
