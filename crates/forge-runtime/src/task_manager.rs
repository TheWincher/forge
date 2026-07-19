use std::time::Duration;

use tokio::task::{JoinHandle, JoinSet};

pub struct TaskManager {
    tasks: JoinSet<()>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: JoinSet::new(),
        }
    }
}

impl TaskManager {
    pub(crate) fn spawn<F>(&mut self, task: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.tasks.spawn(task);
    }

    pub(crate) async fn shutdown(&mut self, timeout: Duration) {
        let deadline = tokio::time::sleep(timeout);
        tokio::pin!(deadline);

        loop {
            tokio::select! {
                joined = self.tasks.join_next() => {
                    match joined {
                        Some(Ok(())) => continue,
                        Some(Err(error)) => {
                            tracing::warn!(?error, "Task panicked during shutdown");
                            continue;
                        }
                        None => {
                            tracing::info!("All tasks completed cleanly");
                            break;
                        }
                    }
                }
                _ = &mut deadline => {
                    tracing::warn!("Shutdown timeout reached, aborting remaining tasks");
                    self.tasks.abort_all();
                    break;
                }
            }
        }
    }
}
