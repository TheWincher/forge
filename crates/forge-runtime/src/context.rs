use crate::handle::RuntimeHandle;
use forge_config::Config;
use forge_workspace::Workspace;

#[derive(Clone)]
pub struct RuntimeContext {
    handle: RuntimeHandle,
    config: Config,
    workspace: Option<Workspace>,
}

impl RuntimeContext {
    pub(crate) fn new(handle: RuntimeHandle, config: Config, workspace: Option<Workspace>) -> Self {
        Self {
            handle,
            config,
            workspace,
        }
    }

    pub fn handle(&self) -> &RuntimeHandle {
        &self.handle
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn workspace(&self) -> Option<&Workspace> {
        self.workspace.as_ref()
    }
}
