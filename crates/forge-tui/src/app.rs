use forge_editor::{DocumentBufferSnapshot, EditorError, EditorHandle};
use forge_workspace::{WorkspaceHandle, WorkspaceHandleError};

use crate::error::TuiError;

pub struct TuiApp {
    workspace: WorkspaceHandle,
    editor: EditorHandle,
}

impl TuiApp {
    pub fn new(workspace: WorkspaceHandle, editor: EditorHandle) -> Self {
        Self { workspace, editor }
    }

    pub async fn active_buffer(&self) -> Result<Option<DocumentBufferSnapshot>, TuiError> {
        let document = match self.workspace.active_document().await {
            Ok(document) => document,

            Err(WorkspaceHandleError::WorkspaceNotOpen) => {
                return Ok(None);
            }

            Err(error) => {
                return Err(error.into());
            }
        };

        let Some(document) = document else {
            return Ok(None);
        };

        match self.editor.buffer(document.id()).await {
            Ok(buffer) => Ok(Some(buffer)),

            // Peut arriver brièvement pendant la synchronisation
            // Workspace -> EventBus -> EditorService.
            Err(EditorError::BufferNotOpen { .. }) => Ok(None),

            Err(error) => Err(error.into()),
        }
    }
}
