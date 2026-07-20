use thiserror::Error;

use crate::context::RuntimeContext;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("plugin initialization failed: {0}")]
    InitializationFailed(String),

    #[error("plugin shutdown failed: {0}")]
    ShutdownFailed(String),

    #[error("plugin error: {0}")]
    Other(String),
}

pub trait Plugin: Send {
    fn name(&self) -> &'static str;

    fn init(&mut self, context: &RuntimeContext) -> Result<(), PluginError>;

    fn shutdown(&mut self, context: &RuntimeContext) -> Result<(), PluginError>;
}
