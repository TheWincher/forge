use std::time::Duration;

use forge_config::Config;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    watch,
};

use crate::{
    application::Application,
    context::RuntimeContext,
    dispatcher::EventDispatcher,
    error::RuntimeError,
    event::AppEvent,
    handle::RuntimeHandle,
    lifecycle::RuntimeState,
    plugin::registrar::{DefaultPluginRegistrar, PluginRegistrar},
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
    tokio_handle: tokio::runtime::Handle,
    event_sender: Sender<AppEvent>,
    event_receiver: Receiver<AppEvent>,
    state_sender: watch::Sender<RuntimeState>,
}

impl Runtime {
    pub fn new() -> Result<Self, RuntimeError> {
        Self::with_registrar(&DefaultPluginRegistrar)
    }

    pub fn with_registrar<R>(registrar: &R) -> Result<Self, RuntimeError>
    where
        R: PluginRegistrar + ?Sized,
    {
        let config = Config::load();
        let app = Application::new(config, registrar)?;

        let tokio_handle = tokio::runtime::Handle::current();

        let (task_manager, task_handle) = TaskManager::new();
        tokio_handle.spawn(task_manager.run());

        let (sender, receiver) = mpsc::channel::<AppEvent>(100);
        let (state_sender, _state_receiver) = watch::channel(RuntimeState::Created);

        Ok(Self {
            state: RuntimeState::Created,
            app,
            task_handle,
            tokio_handle,
            event_sender: sender,
            event_receiver: receiver,
            state_sender,
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

        if let Err(error) = self.start_runtime() {
            self.transition_to(RuntimeState::Failed);
            return Err(error);
        }

        self.transition_to(RuntimeState::Running);

        let runtime_result = self.run_event_loop();

        self.transition_to(RuntimeState::Stopping);

        let shutdown_result = self.stop_runtime();

        match (runtime_result, shutdown_result) {
            (Ok(()), Ok(())) => {
                self.transition_to(RuntimeState::Stopped);
                Ok(())
            }

            (Err(runtime_error), Ok(())) => {
                self.transition_to(RuntimeState::Failed);
                Err(runtime_error)
            }

            (Ok(()), Err(shutdown_error)) => {
                self.transition_to(RuntimeState::Failed);
                Err(shutdown_error)
            }

            (Err(runtime_error), Err(shutdown_error)) => {
                tracing::error!(
                    ?shutdown_error,
                    "Runtime shutdown also failed after event loop failure"
                );

                self.transition_to(RuntimeState::Failed);
                Err(runtime_error)
            }
        }
    }

    pub fn handle(&self) -> RuntimeHandle {
        RuntimeHandle::new(&self.event_sender, self.state_sender.subscribe())
    }

    fn start_runtime(&mut self) -> Result<(), RuntimeError> {
        let runtime_handle = self.handle();
        let task_handle = self.task_handle.clone();

        self.tokio_handle
            .block_on(Self::enable_signal_handler(task_handle, runtime_handle))?;

        let context = self.context();
        self.app.start(&context)?;

        Ok(())
    }

    fn run_event_loop(&mut self) -> Result<(), RuntimeError> {
        let event_loop = Self::event_loop(&mut self.event_receiver);
        self.tokio_handle.block_on(event_loop)?;

        Ok(())
    }

    fn stop_runtime(&mut self) -> Result<(), RuntimeError> {
        let context = self.context();
        self.app.stop(&context)?;

        let shutdown = self.task_handle.shutdown(Duration::from_secs(5));
        self.tokio_handle.block_on(shutdown)?;

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
        let _ = self.state_sender.send(state);
    }
}
