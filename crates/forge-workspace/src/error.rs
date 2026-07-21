use std::path::PathBuf;

use thiserror::Error;

use crate::DocumentId;

#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("Folder not found or it is a file: {0}")]
    FolderNotFoundOrFile(PathBuf),

    #[error("workspace is already closed")]
    AlreadyClosed,

    #[error("document not found: {0:?}")]
    DocumentNotFound(DocumentId),

    #[error("file not found: {0}")]
    FileNotFound(PathBuf),

    #[error("document already open: {0}")]
    DocumentAlreadyOpen(PathBuf),

    #[error("document is outside of workspace: {0}")]
    DocumentOutsideWorkspace(PathBuf),

    #[error("workspace is closed")]
    WorkspaceClosed,
}

#[derive(Debug, Error)]
pub enum WorkspaceHandleError {
    #[error("no workspace is currently open")]
    WorkspaceNotOpen,

    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
}
