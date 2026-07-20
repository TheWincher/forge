use std::collections::HashMap;

use crate::{
    context::RuntimeContext,
    plugin::{
        descriptor::PluginDescriptor,
        plugin::{Plugin, PluginError},
    },
};

#[derive(Debug, thiserror::Error)]
pub enum PluginServiceError {
    #[error("plugin `{plugin_id}` is already registered")]
    DuplicatePluginId { plugin_id: &'static str },

    #[error("failed to initialize plugin `{plugin_id}`")]
    InitializationFailed {
        plugin_id: &'static str,

        #[source]
        source: PluginError,
    },

    #[error("plugin `{plugin_id}` depends on missing plugin `{dependency}`")]
    MissingDependency {
        plugin_id: &'static str,
        dependency: &'static str,
    },

    #[error("failed to shut down plugin `{plugin_id}`")]
    ShutdownFailed {
        plugin_id: &'static str,

        #[source]
        source: PluginError,
    },
}

pub struct PluginService {
    plugins: Vec<Box<dyn Plugin>>,
    plugin_indices: HashMap<&'static str, usize>,
    initialized_count: usize,
}

impl PluginService {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            plugin_indices: HashMap::new(),
            initialized_count: 0,
        }
    }

    pub fn register<P>(&mut self, plugin: P) -> Result<(), PluginServiceError>
    where
        P: Plugin + 'static,
    {
        self.register_boxed(Box::new(plugin))
    }

    fn register_boxed(&mut self, plugin: Box<dyn Plugin>) -> Result<(), PluginServiceError> {
        let descriptor = *plugin.descriptor();
        let plugin_id = descriptor.id();

        if self.plugin_indices.contains_key(plugin_id) {
            return Err(PluginServiceError::DuplicatePluginId { plugin_id });
        }

        let index = self.plugins.len();

        tracing::debug!(
            plugin_id,
            plugin_name = descriptor.name(),
            plugin_version = descriptor.version(),
            "Registering plugin"
        );

        self.plugins.push(plugin);
        self.plugin_indices.insert(plugin_id, index);

        Ok(())
    }

    pub fn init_all(&mut self, context: &RuntimeContext) -> Result<(), PluginServiceError> {
        self.initialized_count = 0;

        for index in 0..self.plugins.len() {
            let descriptor = *self.plugins[index].descriptor();

            tracing::info!(
                plugin_id = descriptor.id(),
                plugin_name = descriptor.name(),
                plugin_version = descriptor.version(),
                "Initializing plugin"
            );

            if let Err(source) = self.plugins[index].init(context) {
                self.rollback_initialized(context);

                return Err(PluginServiceError::InitializationFailed {
                    plugin_id: descriptor.id(),
                    source,
                });
            }

            self.initialized_count += 1;
        }

        Ok(())
    }

    pub fn validate_dependencies(&self) -> Result<(), PluginServiceError> {
        for plugin in &self.plugins {
            let descriptor = plugin.descriptor();

            for dependency in descriptor.dependencies() {
                if !self.plugin_indices.contains_key(dependency) {
                    return Err(PluginServiceError::MissingDependency {
                        plugin_id: descriptor.id(),
                        dependency,
                    });
                }
            }
        }

        Ok(())
    }

    pub fn shutdown_all(&mut self, context: &RuntimeContext) -> Result<(), PluginServiceError> {
        let mut first_error = None;

        while self.initialized_count > 0 {
            let index = self.initialized_count - 1;
            let descriptor = *self.plugins[index].descriptor();

            tracing::info!(
                plugin_id = descriptor.id(),
                plugin_name = descriptor.name(),
                "Shutting down plugin"
            );

            if let Err(source) = self.plugins[index].shutdown(context) {
                tracing::error!(
                    plugin_id = descriptor.id(),
                    ?source,
                    "Plugin shutdown failed"
                );

                if first_error.is_none() {
                    first_error = Some(PluginServiceError::ShutdownFailed {
                        plugin_id: descriptor.id(),
                        source,
                    });
                }
            }

            self.initialized_count -= 1;
        }

        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    fn rollback_initialized(&mut self, context: &RuntimeContext) {
        while self.initialized_count > 0 {
            let index = self.initialized_count - 1;
            let descriptor = *self.plugins[index].descriptor();

            if let Err(error) = self.plugins[index].shutdown(context) {
                tracing::error!(
                    plugin_id = descriptor.id(),
                    ?error,
                    "Plugin rollback failed"
                );
            }

            self.initialized_count -= 1;
        }
    }

    pub fn contains(&self, plugin_id: &str) -> bool {
        self.plugin_indices.contains_key(plugin_id)
    }

    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &'static PluginDescriptor> + '_ {
        self.plugins.iter().map(|plugin| plugin.descriptor())
    }
}

#[cfg(test)]
mod tests {
    use crate::plugin::descriptor::PluginDescriptor;

    use super::*;

    const TEST_DESCRIPTOR: PluginDescriptor =
        PluginDescriptor::new("forge.test", "Test plugin", "0.1.0", &[]);

    struct TestPlugin;

    impl Plugin for TestPlugin {
        fn descriptor(&self) -> &'static PluginDescriptor {
            &TEST_DESCRIPTOR
        }

        fn init(&mut self, _context: &RuntimeContext) -> Result<(), PluginError> {
            Ok(())
        }

        fn shutdown(&mut self, _context: &RuntimeContext) -> Result<(), PluginError> {
            Ok(())
        }
    }

    #[test]
    fn rejects_duplicate_plugin_ids() {
        let mut service = PluginService::new();

        service.register(TestPlugin).unwrap();

        let error = service.register(TestPlugin).unwrap_err();

        assert!(matches!(
            error,
            PluginServiceError::DuplicatePluginId {
                plugin_id: "forge.test"
            }
        ));

        assert_eq!(service.len(), 1);
    }
}
