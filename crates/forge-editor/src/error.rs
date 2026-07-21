use std::{io, path::PathBuf};

use forge_workspace::DocumentId;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EditorError {
    #[error("file not found: {0}")]
    FileNotFound(PathBuf),

    #[error("failed to read file: {path}")]
    FailedToReadFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to save file: {path}")]
    FailedToSaveFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("buffer already open: {0}")]
    BufferAlreadyOpen(DocumentId),

    #[error("buffer not open: {0}")]
    BufferNotOpen(DocumentId),
}
