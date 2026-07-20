use std::time::Duration;

use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{
    application::Application,
    context::RuntimeContext,
    dispatcher::EventDispatcher,
    error::RuntimeError,
    event::AppEvent,
    handle::RuntimeHandle,
    lifecycle::RuntimeState,
    signal::wait_for_shutdown,
    task_manager::{TaskHandle, TaskManager},
};

pub enum RuntimeAction {
    Continue,
    Stop,
}

pub struct Runtime {
    state: RuntimeState,
    app: Application,
    task_handle: TaskHandle,
    tokio_runtime: tokio::runtime::Runtime,
    event_sender: Sender<AppEvent>,
    event_receiver: Receiver<AppEvent>,
}

impl Runtime {
    pub fn new() -> Result<Self, RuntimeError> {
        let tokio_runtime =
            tokio::runtime::Runtime::new().map_err(RuntimeError::TokioInitializationFailed)?;

        let (task_manager, task_handle) = TaskManager::new();
        tokio_runtime.spawn(task_manager.run());

        let (sender, receiver) = mpsc::channel::<AppEvent>(100);

        Ok(Self {
            state: RuntimeState::Created,
            app: Application::new(),
            task_handle,
            tokio_runtime,
            event_sender: sender,
            event_receiver: receiver,
        })
    }

    pub fn context(&self) -> RuntimeContext {
        RuntimeContext::new(
            self.handle(),
            self.app.services().handle(),
            self.task_handle.clone(),
        )
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        self.transition_to(RuntimeState::Starting);
        self.start_runtime()?;

        self.transition_to(RuntimeState::Running);
        self.run_event_loop()?;

        self.transition_to(RuntimeState::Stopping);
        self.stop_runtime()?;

        self.transition_to(RuntimeState::Stopped);

        Ok(())
    }

    pub fn handle(&self) -> RuntimeHandle {
        RuntimeHandle::new(&self.event_sender)
    }

    fn start_runtime(&mut self) -> Result<(), RuntimeError> {
        let context = self.context();
        self.app.start(&context)?;

        let runtime_handle = self.handle();
        let task_handle = self.task_handle.clone();

        self.tokio_runtime
            .block_on(Self::enable_signal_handler(task_handle, runtime_handle))?;

        Ok(())
    }

    fn run_event_loop(&mut self) -> Result<(), RuntimeError> {
        let event_loop = Self::event_loop(&mut self.event_receiver);
        self.tokio_runtime.block_on(event_loop)?;

        Ok(())
    }

    fn stop_runtime(&mut self) -> Result<(), RuntimeError> {
        let context = self.context();
        self.app.stop(&context)?;

        let shutdown = self.task_handle.shutdown(Duration::from_secs(5));
        self.tokio_runtime.block_on(shutdown)?;

        Ok(())
    }

    async fn event_loop(receiver: &mut Receiver<AppEvent>) -> Result<(), RuntimeError> {
        let dispatcher = EventDispatcher::new();

        loop {
            let event = receiver.recv().await;

            let Some(event) = event else {
                tracing::warn!("Event channel closed");
                return Ok(());
            };

            let action = dispatcher.dispatch(event).await?;

            match action {
                RuntimeAction::Continue => continue,
                RuntimeAction::Stop => return Ok(()),
            }
        }
    }

    async fn enable_signal_handler(
        task_handle: TaskHandle,
        runtime_handle: RuntimeHandle,
    ) -> Result<(), RuntimeError> {
        task_handle
            .spawn(async move {
                if let Err(error) = wait_for_shutdown(runtime_handle).await {
                    tracing::error!(?error, "Signal handler failed");
                }
            })
            .await?;

        Ok(())
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
