use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use forge_config::Config;
use forge_workspace::Workspace;
use tokio::sync::RwLock;

pub struct WorkspaceService {
    workspace: Arc<RwLock<Option<Workspace>>>,
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

        Self {
            workspace: Arc::new(RwLock::new(workspace)),
        }
    }

    pub fn handle(&self) -> WorkspaceHandle {
        WorkspaceHandle {
            workspace: Arc::clone(&self.workspace),
        }
    }
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkspaceHandleError {
    #[error("failed to open workspace")]
    OpenFailed(#[source] forge_workspace::WorkspaceError),
}

#[derive(Clone)]
pub struct WorkspaceHandle {
    workspace: Arc<RwLock<Option<Workspace>>>,
}

impl WorkspaceHandle {
    pub async fn is_open(&self) -> bool {
        self.workspace.read().await.is_some()
    }

    pub async fn root(&self) -> Option<PathBuf> {
        self.workspace
            .read()
            .await
            .as_ref()
            .map(|workspace| workspace.root().to_path_buf())
    }

    pub async fn open(&self, root: impl AsRef<Path>) -> Result<(), WorkspaceHandleError> {
        let workspace = Workspace::open(root.as_ref().to_path_buf())
            .map_err(WorkspaceHandleError::OpenFailed)?;

        *self.workspace.write().await = Some(workspace);

        Ok(())
    }

    pub async fn close(&self) {
        *self.workspace.write().await = None;
    }
}
