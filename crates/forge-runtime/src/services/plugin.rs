use thiserror::Error;

use crate::{
    context::RuntimeContext,
    plugin::{Plugin, PluginError},
};

#[derive(Debug, Error)]
pub enum PluginServiceError {
    #[error("failed to initialize plugin `{plugin}`")]
    InitializationFailed {
        plugin: String,

        #[source]
        source: PluginError,
    },

    #[error("failed to shut down plugin `{plugin}`")]
    ShutdownFailed {
        plugin: String,

        #[source]
        source: PluginError,
    },
}

pub struct PluginService {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginService {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn init_all(&mut self, context: &RuntimeContext) -> Result<(), PluginServiceError> {
        for plugin in &mut self.plugins {
            let plugin_name = plugin.name().to_owned();
            plugin
                .init(context)
                .map_err(|source| PluginServiceError::InitializationFailed {
                    plugin: plugin_name,
                    source,
                })?;
        }

        Ok(())
    }

    pub fn shutdown_all(&mut self, context: &RuntimeContext) -> Result<(), PluginServiceError> {
        for plugin in self.plugins.iter_mut().rev() {
            let plugin_name = plugin.name().to_owned();

            plugin
                .shutdown(context)
                .map_err(|source| PluginServiceError::ShutdownFailed {
                    plugin: plugin_name,
                    source,
                })?;
        }

        Ok(())
    }
}
