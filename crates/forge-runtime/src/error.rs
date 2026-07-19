use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Failed to initialize Tokio runtime: {0}")]
    TokioInitializationFailed(io::Error),
}
