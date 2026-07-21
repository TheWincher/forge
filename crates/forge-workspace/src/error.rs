use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("Folder not found or it is a file: {0}")]
    FolderNotFoundOrFile(PathBuf),

    #[error("workspace is already closed")]
    AlreadyClosed,
}
