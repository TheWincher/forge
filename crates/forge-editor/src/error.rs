use std::{io, path::PathBuf};

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
}
