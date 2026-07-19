use std::{future::Future, time::Duration};

use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};

use crate::{
    application::Application, error::RuntimeError, event::AppEvent, handle::RuntimeHandle,
    lifecycle::RuntimeState, signal::wait_for_shutdown, task_manager::TaskManager,
};

pub struct Runtime {
    state: RuntimeState,
    app: Application,
    task_manager: TaskManager,
    tokio_runtime: tokio::runtime::Runtime,
    event_sender: Sender<AppEvent>,
    event_receiver: Receiver<AppEvent>,
    tasks: Vec<JoinHandle<()>>,
}

struct RuntimeContext<'a> {
    state: &'a mut RuntimeState,
    app: &'a mut Application,
    task_manager: &'a mut TaskManager,
}

impl Runtime {
    pub fn new() -> Result<Self, RuntimeError> {
        let tokio_runtime =
            tokio::runtime::Runtime::new().map_err(RuntimeError::TokioInitializationFailed)?;

        let (sender, receiver) = mpsc::channel::<AppEvent>(100);

        let mut runtime = Self {
            state: RuntimeState::Created,
            app: Application::new(),
            task_manager: TaskManager::new(),
            tokio_runtime,
            event_sender: sender,
            event_receiver: receiver,
            tasks: vec![],
        };

        runtime.enable_signal_handler();

        Ok(runtime)
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        self.state = RuntimeState::Running;

        let context = RuntimeContext {
            state: &mut self.state,
            app: &mut self.app,
            task_manager: &mut self.task_manager,
        };

        let receiver = &mut self.event_receiver;

        self.tokio_runtime
            .block_on(Self::event_loop(receiver, context))
    }

    pub fn shutdown(&mut self) {
        for task in self.tasks.drain(..) {
            self.tokio_runtime.block_on(async {
                let result = tokio::time::timeout(Duration::from_secs(5), task).await;

                if result.is_err() {
                    tracing::warn!("Task shutdown timeout");
                }
            });
        }
    }

    pub fn handle(&self) -> RuntimeHandle {
        RuntimeHandle::new(&self.event_sender)
    }

    async fn event_loop(
        receiver: &mut Receiver<AppEvent>,
        mut context: RuntimeContext<'_>,
    ) -> Result<(), RuntimeError> {
        loop {
            let event_opt = receiver.recv().await;
            if let Some(event) = event_opt {
                match event {
                    AppEvent::Started => {}
                    AppEvent::ShutdownRequested => {
                        tracing::info!("Shutdown requested");
                        *context.state = RuntimeState::Stopping;
                        break;
                    }
                    AppEvent::ShutdownCompleted => {}
                }
            } else {
                tracing::warn!("Event channel closed");
                break;
            }
        }

        Ok(())
    }

    fn spawn<F>(&mut self, task: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let handle = self.tokio_runtime.spawn(task);
        self.tasks.push(handle);
    }

    fn enable_signal_handler(&mut self) {
        let handle = self.handle();

        self.spawn(async move {
            if let Err(error) = wait_for_shutdown(handle).await {
                tracing::error!(?error, "Signal handler failed");
            }
        });
    }
}
