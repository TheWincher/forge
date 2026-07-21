use std::path::PathBuf;

use crate::{DocumentId, WorkspaceId};

#[derive(Debug, Clone)]
pub struct DocumentOpened {
    pub workspace_id: WorkspaceId,
    pub document_id: DocumentId,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DocumentClosed {
    pub workspace_id: WorkspaceId,
    pub document_id: DocumentId,
}

#[derive(Debug, Clone)]
pub struct ActiveDocumentChanged {
    pub workspace_id: WorkspaceId,
    pub previous: Option<DocumentId>,
    pub current: Option<DocumentId>,
}
