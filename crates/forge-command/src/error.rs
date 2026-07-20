use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("command `{command_id}` is already registered")]
    DuplicateCommandId { command_id: &'static str },

    #[error("command `{command_id}` was not found")]
    CommandNotFound { command_id: String },

    #[error("command execution failed: {0}")]
    ExecutionFailed(String),
}
