use thiserror::Error;

use crate::{context::RuntimeContext, plugin::descriptor::PluginDescriptor};

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
    fn descriptor(&self) -> &'static PluginDescriptor;

    fn init(&mut self, context: &RuntimeContext) -> Result<(), PluginError>;

    fn shutdown(&mut self, context: &RuntimeContext) -> Result<(), PluginError>;
}
