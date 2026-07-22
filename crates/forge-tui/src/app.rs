use forge_editor::{DocumentBufferSnapshot, EditorError, EditorHandle};
use forge_workspace::{WorkspaceHandle, WorkspaceHandleError};

use crate::{editor_state::EditorState, error::TuiError};

pub struct TuiApp {
    workspace: WorkspaceHandle,
    editor: EditorHandle,
    editor_state: EditorState,
}

impl TuiApp {
    pub fn new(workspace: WorkspaceHandle, editor: EditorHandle) -> Self {
        Self {
            workspace,
            editor,
            editor_state: EditorState::default(),
        }
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

    pub fn editor_state(&self) -> &EditorState {
        &self.editor_state
    }

    pub fn editor_state_mut(&mut self) -> &mut EditorState {
        &mut self.editor_state
    }
}
