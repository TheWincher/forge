use std::io;

use forge_editor::EditorError;
use forge_workspace::WorkspaceHandleError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TuiError {
    #[error(transparent)]
    Workspace(#[from] WorkspaceHandleError),

    #[error(transparent)]
    Editor(#[from] EditorError),

    #[error(transparent)]
    Io(#[from] io::Error),
}
