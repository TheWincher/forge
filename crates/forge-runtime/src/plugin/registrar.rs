use crate::services::plugin::{PluginService, PluginServiceError};

pub trait PluginRegistrar {
    fn register(&self, plugins: &mut PluginService) -> Result<(), PluginServiceError>;
}

pub struct DefaultPluginRegistrar;

impl PluginRegistrar for DefaultPluginRegistrar {
    fn register(&self, plugins: &mut PluginService) -> Result<(), PluginServiceError> {
        // plugins.register(WorkspacePlugin::new())?;
        // plugins.register(EditorPlugin::new())?;
        // plugins.register(GitPlugin::new())?;

        Ok(())
    }
}
