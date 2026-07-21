use std::sync::Arc;

use forge_config::Config;
use forge_event::EventHandle;
use tokio::sync::RwLock;

use crate::{Workspace, WorkspaceHandle};

pub struct WorkspaceService {
    workspace: Arc<RwLock<Option<Workspace>>>,
    events: EventHandle,
}

impl WorkspaceService {
    pub fn new(config: &Config, events: EventHandle) -> Self {
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
            events,
        }
    }

    pub fn handle(&self) -> WorkspaceHandle {
        WorkspaceHandle::new(Arc::clone(&self.workspace), self.events.clone())
    }
}
