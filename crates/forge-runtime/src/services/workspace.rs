use std::sync::Arc;

use forge_config::Config;
use forge_workspace::Workspace;
use tokio::sync::RwLock;

pub struct WorkspaceService {
    workspace: Option<Workspace>,
}

impl WorkspaceService {
    pub fn new(config: &Config) -> Self {
        let workspace =
            config
                .workspace_root
                .clone()
                .and_then(|root| match Workspace::open(root) {
                    Ok(workspace) => Some(workspace),
                    Err(error) => {
                        tracing::warn!(%error, "Failed to open workspace");
                        None
                    }
                });

        Self { workspace }
    }
}

#[derive(Clone)]
pub struct WorkspaceHandle {
    workspace: Arc<RwLock<Option<Workspace>>>,
}

impl WorkspaceService {
    pub fn handle(&self) -> WorkspaceHandle {
        WorkspaceHandle {
            workspace: Arc::new(RwLock::new(self.workspace.clone())),
        }
    }
}
