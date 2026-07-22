use std::io;

use thiserror::Error;
use tokio::sync::mpsc::error::TrySendError;

use crate::{application::ApplicationError, event::AppEvent, task_manager::TaskError};

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("failed to start")]
    FailedToStart,

    #[error("state channel closed")]
    StateChannelClosed,

    #[error("failed to initialize Tokio runtime")]
    TokioInitializationFailed(#[source] std::io::Error),

    #[error("Failed to send event: {0}")]
    SendEventFailed(TrySendError<AppEvent>),

    #[error("Signal error: {0}")]
    SignalError(#[from] io::Error),

    #[error("task manager error")]
    Task(#[from] TaskError),

    #[error("application error")]
    Application(#[from] ApplicationError),
    // #[error("dispatcher error")]
    // Dispatcher(#[from] DispatcherError),
}
