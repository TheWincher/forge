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

    #[error("plugin dependency graph contains a cycle")]
    DependencyCycle,

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
    initialized_plugins: Vec<usize>,
}

impl PluginService {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            plugin_indices: HashMap::new(),
            initialized_plugins: Vec::new(),
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
        self.initialized_plugins.clear();
        let initialization_order = self.initialization_order()?;

        for index in initialization_order {
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

            self.initialized_plugins.push(index);
        }

        Ok(())
    }

    fn initialization_order(&self) -> Result<Vec<usize>, PluginServiceError> {
        let mut dependency_counts = vec![0usize; self.plugins.len()];
        let mut dependents = vec![Vec::<usize>::new(); self.plugins.len()];

        for (plugin_index, plugin) in self.plugins.iter().enumerate() {
            let descriptor = plugin.descriptor();

            for dependency_id in descriptor.dependencies() {
                let dependency_index = *self.plugin_indices.get(dependency_id).ok_or(
                    PluginServiceError::MissingDependency {
                        plugin_id: descriptor.id(),
                        dependency: dependency_id,
                    },
                )?;

                dependency_counts[plugin_index] += 1;
                dependents[dependency_index].push(plugin_index);
            }
        }

        let mut ready = std::collections::VecDeque::new();

        for (index, dependency_count) in dependency_counts.iter().enumerate() {
            if *dependency_count == 0 {
                ready.push_back(index);
            }
        }

        let mut order = Vec::with_capacity(self.plugins.len());

        while let Some(plugin_index) = ready.pop_front() {
            order.push(plugin_index);

            for dependent_index in &dependents[plugin_index] {
                dependency_counts[*dependent_index] -= 1;

                if dependency_counts[*dependent_index] == 0 {
                    ready.push_back(*dependent_index);
                }
            }
        }

        if order.len() != self.plugins.len() {
            return Err(PluginServiceError::DependencyCycle);
        }

        Ok(order)
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

        while let Some(index) = self.initialized_plugins.pop() {
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
        }

        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    fn rollback_initialized(&mut self, context: &RuntimeContext) {
        while let Some(index) = self.initialized_plugins.pop() {
            let descriptor = *self.plugins[index].descriptor();

            if let Err(error) = self.plugins[index].shutdown(context) {
                tracing::error!(
                    plugin_id = descriptor.id(),
                    ?error,
                    "Plugin rollback failed"
                );
            }
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
    use super::*;

    use crate::{
        context::RuntimeContext,
        plugin::{
            descriptor::PluginDescriptor,
            plugin::{Plugin, PluginError},
        },
    };

    const INDEPENDENT_DESCRIPTOR: PluginDescriptor =
        PluginDescriptor::new("forge.independent", "Independent plugin", "0.1.0", &[]);

    const WORKSPACE_DESCRIPTOR: PluginDescriptor =
        PluginDescriptor::new("forge.workspace", "Workspace", "0.1.0", &[]);

    const FILESYSTEM_DESCRIPTOR: PluginDescriptor =
        PluginDescriptor::new("forge.fs", "File system", "0.1.0", &[]);

    const GIT_DESCRIPTOR: PluginDescriptor = PluginDescriptor::new(
        "forge.git",
        "Git",
        "0.1.0",
        &["forge.workspace", "forge.fs"],
    );

    const EDITOR_DESCRIPTOR: PluginDescriptor =
        PluginDescriptor::new("forge.editor", "Editor", "0.1.0", &["forge.workspace"]);

    const MISSING_DEPENDENCY_DESCRIPTOR: PluginDescriptor = PluginDescriptor::new(
        "forge.missing-dependency",
        "Missing dependency plugin",
        "0.1.0",
        &["forge.unknown"],
    );

    const CYCLE_A_DESCRIPTOR: PluginDescriptor =
        PluginDescriptor::new("forge.cycle-a", "Cycle A", "0.1.0", &["forge.cycle-b"]);

    const CYCLE_B_DESCRIPTOR: PluginDescriptor =
        PluginDescriptor::new("forge.cycle-b", "Cycle B", "0.1.0", &["forge.cycle-c"]);

    const CYCLE_C_DESCRIPTOR: PluginDescriptor =
        PluginDescriptor::new("forge.cycle-c", "Cycle C", "0.1.0", &["forge.cycle-a"]);

    struct TestPlugin {
        descriptor: &'static PluginDescriptor,
    }

    impl TestPlugin {
        const fn new(descriptor: &'static PluginDescriptor) -> Self {
            Self { descriptor }
        }
    }

    impl Plugin for TestPlugin {
        fn descriptor(&self) -> &'static PluginDescriptor {
            self.descriptor
        }

        fn init(&mut self, _context: &RuntimeContext) -> Result<(), PluginError> {
            Ok(())
        }

        fn shutdown(&mut self, _context: &RuntimeContext) -> Result<(), PluginError> {
            Ok(())
        }
    }

    fn plugin_ids_from_order(service: &PluginService, order: &[usize]) -> Vec<&'static str> {
        order
            .iter()
            .map(|index| service.plugins[*index].descriptor().id())
            .collect()
    }

    #[test]
    fn registers_plugin() {
        let mut service = PluginService::new();

        service
            .register(TestPlugin::new(&WORKSPACE_DESCRIPTOR))
            .unwrap();

        assert_eq!(service.len(), 1);
        assert!(!service.is_empty());
        assert!(service.contains("forge.workspace"));
    }

    #[test]
    fn rejects_duplicate_plugin_ids() {
        let mut service = PluginService::new();

        service
            .register(TestPlugin::new(&WORKSPACE_DESCRIPTOR))
            .unwrap();

        let error = service
            .register(TestPlugin::new(&WORKSPACE_DESCRIPTOR))
            .unwrap_err();

        assert!(matches!(
            error,
            PluginServiceError::DuplicatePluginId {
                plugin_id: "forge.workspace",
            }
        ));

        assert_eq!(service.len(), 1);
    }

    #[test]
    fn exposes_registered_plugin_descriptors() {
        let mut service = PluginService::new();

        service
            .register(TestPlugin::new(&WORKSPACE_DESCRIPTOR))
            .unwrap();

        service.register(TestPlugin::new(&GIT_DESCRIPTOR)).unwrap();

        let descriptor_ids = service
            .descriptors()
            .map(PluginDescriptor::id)
            .collect::<Vec<_>>();

        assert_eq!(descriptor_ids, vec!["forge.workspace", "forge.git",]);
    }

    #[test]
    fn accepts_existing_dependencies() {
        let mut service = PluginService::new();

        // Le plugin dépendant est volontairement enregistré en premier.
        service.register(TestPlugin::new(&GIT_DESCRIPTOR)).unwrap();

        service
            .register(TestPlugin::new(&FILESYSTEM_DESCRIPTOR))
            .unwrap();

        service
            .register(TestPlugin::new(&WORKSPACE_DESCRIPTOR))
            .unwrap();

        assert!(service.validate_dependencies().is_ok());
    }

    #[test]
    fn rejects_missing_dependency() {
        let mut service = PluginService::new();

        service
            .register(TestPlugin::new(&MISSING_DEPENDENCY_DESCRIPTOR))
            .unwrap();

        let error = service.validate_dependencies().unwrap_err();

        assert!(matches!(
            error,
            PluginServiceError::MissingDependency {
                plugin_id: "forge.missing-dependency",
                dependency: "forge.unknown",
            }
        ));
    }

    #[test]
    fn computes_dependency_initialization_order() {
        let mut service = PluginService::new();

        // Ordre d'enregistrement volontairement incorrect.
        service.register(TestPlugin::new(&GIT_DESCRIPTOR)).unwrap();

        service
            .register(TestPlugin::new(&WORKSPACE_DESCRIPTOR))
            .unwrap();

        service
            .register(TestPlugin::new(&FILESYSTEM_DESCRIPTOR))
            .unwrap();

        let order = service.initialization_order().unwrap();
        let plugin_ids = plugin_ids_from_order(&service, &order);

        let workspace_position = plugin_ids
            .iter()
            .position(|id| *id == "forge.workspace")
            .unwrap();

        let filesystem_position = plugin_ids.iter().position(|id| *id == "forge.fs").unwrap();

        let git_position = plugin_ids.iter().position(|id| *id == "forge.git").unwrap();

        assert!(workspace_position < git_position);
        assert!(filesystem_position < git_position);
    }

    #[test]
    fn computes_order_for_multiple_dependents() {
        let mut service = PluginService::new();

        service.register(TestPlugin::new(&GIT_DESCRIPTOR)).unwrap();

        service
            .register(TestPlugin::new(&EDITOR_DESCRIPTOR))
            .unwrap();

        service
            .register(TestPlugin::new(&WORKSPACE_DESCRIPTOR))
            .unwrap();

        service
            .register(TestPlugin::new(&FILESYSTEM_DESCRIPTOR))
            .unwrap();

        let order = service.initialization_order().unwrap();
        let plugin_ids = plugin_ids_from_order(&service, &order);

        let workspace_position = plugin_ids
            .iter()
            .position(|id| *id == "forge.workspace")
            .unwrap();

        let filesystem_position = plugin_ids.iter().position(|id| *id == "forge.fs").unwrap();

        let git_position = plugin_ids.iter().position(|id| *id == "forge.git").unwrap();

        let editor_position = plugin_ids
            .iter()
            .position(|id| *id == "forge.editor")
            .unwrap();

        assert!(workspace_position < git_position);
        assert!(filesystem_position < git_position);
        assert!(workspace_position < editor_position);
    }

    #[test]
    fn preserves_registration_order_for_independent_plugins() {
        let mut service = PluginService::new();

        service
            .register(TestPlugin::new(&INDEPENDENT_DESCRIPTOR))
            .unwrap();

        service
            .register(TestPlugin::new(&WORKSPACE_DESCRIPTOR))
            .unwrap();

        service
            .register(TestPlugin::new(&FILESYSTEM_DESCRIPTOR))
            .unwrap();

        let order = service.initialization_order().unwrap();
        let plugin_ids = plugin_ids_from_order(&service, &order);

        assert_eq!(
            plugin_ids,
            vec!["forge.independent", "forge.workspace", "forge.fs",]
        );
    }

    #[test]
    fn rejects_direct_dependency_cycle() {
        const DIRECT_CYCLE_A: PluginDescriptor = PluginDescriptor::new(
            "forge.direct-cycle-a",
            "Direct cycle A",
            "0.1.0",
            &["forge.direct-cycle-b"],
        );

        const DIRECT_CYCLE_B: PluginDescriptor = PluginDescriptor::new(
            "forge.direct-cycle-b",
            "Direct cycle B",
            "0.1.0",
            &["forge.direct-cycle-a"],
        );

        let mut service = PluginService::new();

        service.register(TestPlugin::new(&DIRECT_CYCLE_A)).unwrap();

        service.register(TestPlugin::new(&DIRECT_CYCLE_B)).unwrap();

        let error = service.initialization_order().unwrap_err();

        assert!(matches!(error, PluginServiceError::DependencyCycle));
    }

    #[test]
    fn rejects_transitive_dependency_cycle() {
        let mut service = PluginService::new();

        service
            .register(TestPlugin::new(&CYCLE_A_DESCRIPTOR))
            .unwrap();

        service
            .register(TestPlugin::new(&CYCLE_B_DESCRIPTOR))
            .unwrap();

        service
            .register(TestPlugin::new(&CYCLE_C_DESCRIPTOR))
            .unwrap();

        let error = service.initialization_order().unwrap_err();

        assert!(matches!(error, PluginServiceError::DependencyCycle));
    }

    #[test]
    fn cycle_does_not_hide_independent_plugins() {
        let mut service = PluginService::new();

        service
            .register(TestPlugin::new(&INDEPENDENT_DESCRIPTOR))
            .unwrap();

        service
            .register(TestPlugin::new(&CYCLE_A_DESCRIPTOR))
            .unwrap();

        service
            .register(TestPlugin::new(&CYCLE_B_DESCRIPTOR))
            .unwrap();

        service
            .register(TestPlugin::new(&CYCLE_C_DESCRIPTOR))
            .unwrap();

        let error = service.initialization_order().unwrap_err();

        assert!(matches!(error, PluginServiceError::DependencyCycle));
    }
}
