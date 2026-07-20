use std::{future::Future, pin::Pin, time::Duration};

use tokio::{
    sync::{mpsc, oneshot},
    task::{JoinError, JoinSet},
};

type BoxedTask = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

enum TaskCommand {
    Spawn(BoxedTask),
    Shutdown {
        timeout: Duration,
        response: oneshot::Sender<()>,
    },
}

pub struct TaskManager {
    tasks: JoinSet<()>,
    command_sender: mpsc::Sender<TaskCommand>,
    command_receiver: mpsc::Receiver<TaskCommand>,
}

impl TaskManager {
    pub fn new() -> Self {
        let (command_sender, command_receiver) = mpsc::channel(100);

        Self {
            tasks: JoinSet::new(),
            command_sender,
            command_receiver,
        }
    }

    pub fn handle(&self) -> TaskHandle {
        TaskHandle {
            command_sender: self.command_sender.clone(),
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                command = self.command_receiver.recv() => {
                    let Some(command) = command else {
                        self.abort_all().await;
                        break;
                    };

                    match command {
                        TaskCommand::Spawn(task) => {
                            self.tasks.spawn(task);
                        }

                        TaskCommand::Shutdown {
                            timeout,
                            response,
                        } => {
                            self.shutdown_tasks(timeout).await;

                            let _ = response.send(());
                            break;
                        }
                    }
                }

                result = self.tasks.join_next(), if !self.tasks.is_empty() => {
                    if let Some(result) = result {
                        Self::handle_task_result(result);
                    }
                }
            }
        }
    }

    async fn shutdown_tasks(&mut self, timeout: Duration) {
        self.command_receiver.close();
        self.drain_pending_tasks();

        let graceful_shutdown = async {
            while let Some(result) = self.tasks.join_next().await {
                Self::handle_task_result(result);
            }
        };

        if tokio::time::timeout(timeout, graceful_shutdown)
            .await
            .is_err()
        {
            tracing::warn!(
                ?timeout,
                "Task shutdown timeout reached, aborting remaining tasks"
            );

            self.abort_all().await;
        } else {
            tracing::info!("All tasks completed cleanly");
        }
    }

    fn drain_pending_tasks(&mut self) {
        while let Ok(command) = self.command_receiver.try_recv() {
            match command {
                TaskCommand::Spawn(task) => {
                    self.tasks.spawn(task);
                }

                TaskCommand::Shutdown { .. } => {
                    tracing::warn!("Ignoring duplicate task manager shutdown command");
                }
            }
        }
    }

    async fn abort_all(&mut self) {
        self.tasks.abort_all();

        while let Some(result) = self.tasks.join_next().await {
            Self::handle_task_result(result);
        }
    }

    fn handle_task_result(result: Result<(), JoinError>) {
        if let Err(error) = result {
            if error.is_cancelled() {
                tracing::debug!("Task was cancelled");
            } else if error.is_panic() {
                tracing::warn!(?error, "Task panicked");
            } else {
                tracing::warn!(?error, "Task failed");
            }
        }
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct TaskHandle {
    command_sender: mpsc::Sender<TaskCommand>,
}

impl TaskHandle {
    pub async fn spawn<F>(&self, task: F) -> Result<(), TaskError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.command_sender
            .send(TaskCommand::Spawn(Box::pin(task)))
            .await
            .map_err(|_| TaskError::ManagerStopped)
    }

    pub async fn shutdown(&self, timeout: Duration) -> Result<(), TaskError> {
        let (response_sender, response_receiver) = oneshot::channel();

        self.command_sender
            .send(TaskCommand::Shutdown {
                timeout,
                response: response_sender,
            })
            .await
            .map_err(|_| TaskError::ManagerStopped)?;

        response_receiver
            .await
            .map_err(|_| TaskError::ManagerStopped)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("task manager has stopped")]
    ManagerStopped,
}
