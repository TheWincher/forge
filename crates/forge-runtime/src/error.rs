use std::io;
use thiserror::Error;
use tokio::sync::mpsc::error::TrySendError;

use crate::event::AppEvent;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Failed to initialize Tokio runtime: {0}")]
    TokioInitializationFailed(io::Error),

    #[error("Failed to send event: {0}")]
    SendEventFailed(TrySendError<AppEvent>),
}
