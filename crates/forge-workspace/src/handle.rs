use std::{path::PathBuf, sync::Arc};

use forge_event::EventHandle;
use tokio::sync::RwLock;

use crate::{
    ActiveDocumentChanged, Document, DocumentClosed, DocumentId, DocumentOpened, Workspace,
    WorkspaceId, error::WorkspaceHandleError,
};

#[derive(Clone)]
pub struct WorkspaceHandle {
    workspace: Arc<RwLock<Option<Workspace>>>,
    events: EventHandle,
}

impl WorkspaceHandle {
    pub fn new(workspace: Arc<RwLock<Option<Workspace>>>, events: EventHandle) -> Self {
        Self { workspace, events }
    }

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

    pub async fn open(&self, root: impl Into<PathBuf>) -> Result<(), WorkspaceHandleError> {
        let workspace = Workspace::open(root)?;

        *self.workspace.write().await = Some(workspace);

        Ok(())
    }

    pub async fn close(&self) -> Result<(), WorkspaceHandleError> {
        let mut guard = self.workspace.write().await;
        let workspace = guard
            .as_mut()
            .ok_or(WorkspaceHandleError::WorkspaceNotOpen)?;

        workspace.close()?;
        *guard = None;

        Ok(())
    }

    pub async fn id(&self) -> Option<WorkspaceId> {
        self.workspace.read().await.as_ref().map(Workspace::id)
    }

    pub async fn open_document(
        &self,
        path: impl Into<PathBuf>,
    ) -> Result<DocumentId, WorkspaceHandleError> {
        let mut guard = self.workspace.write().await;
        let workspace = guard
            .as_mut()
            .ok_or(WorkspaceHandleError::WorkspaceNotOpen)?;

        let workspace_id = workspace.id();
        let previous = workspace.active_document().map(Document::id);
        tracing::info!("previous: {:?}", previous);
        let path = path.into();
        let document_id = workspace.open_document(&path)?;
        let current = workspace.active_document().map(Document::id);

        drop(guard);

        self.events.publish(&DocumentOpened {
            document_id,
            workspace_id,
            path,
        });

        if previous != current {
            self.events.publish(&ActiveDocumentChanged {
                workspace_id,
                previous,
                current,
            });
        }

        Ok(document_id)
    }

    pub async fn close_document(&self, id: DocumentId) -> Result<(), WorkspaceHandleError> {
        let mut guard = self.workspace.write().await;
        let workspace = guard
            .as_mut()
            .ok_or(WorkspaceHandleError::WorkspaceNotOpen)?;

        let workspace_id = workspace.id();
        let previous = workspace.active_document().map(Document::id);
        let current = workspace.active_document().map(Document::id);
        workspace.close_document(id)?;

        drop(guard);

        self.events.publish(&DocumentClosed {
            document_id: id,
            workspace_id,
        });

        if previous != current {
            self.events.publish(&ActiveDocumentChanged {
                workspace_id,
                previous,
                current,
            });
        }

        Ok(())
    }

    pub async fn set_active_document(&self, id: DocumentId) -> Result<(), WorkspaceHandleError> {
        let mut guard = self.workspace.write().await;
        let workspace = guard
            .as_mut()
            .ok_or(WorkspaceHandleError::WorkspaceNotOpen)?;

        let workspace_id = workspace.id();
        let previous = workspace.active_document().map(Document::id);
        workspace.set_active_document(id)?;

        let current = workspace.active_document().map(Document::id);

        drop(guard);

        if previous != current {
            self.events.publish(&ActiveDocumentChanged {
                workspace_id,
                previous,
                current,
            });
        }

        Ok(())
    }

    pub async fn document(&self, id: DocumentId) -> Result<Option<Document>, WorkspaceHandleError> {
        let guard = self.workspace.read().await;
        let workspace = guard
            .as_ref()
            .ok_or(WorkspaceHandleError::WorkspaceNotOpen)?;

        Ok(workspace.document(id).cloned())
    }

    pub async fn documents(&self) -> Result<Vec<Document>, WorkspaceHandleError> {
        let guard = self.workspace.read().await;
        let workspace = guard
            .as_ref()
            .ok_or(WorkspaceHandleError::WorkspaceNotOpen)?;

        Ok(workspace.documents().cloned().collect())
    }

    pub async fn active_document(&self) -> Result<Option<Document>, WorkspaceHandleError> {
        let guard = self.workspace.read().await;
        let workspace = guard
            .as_ref()
            .ok_or(WorkspaceHandleError::WorkspaceNotOpen)?;

        Ok(workspace.active_document().cloned())
    }
}
