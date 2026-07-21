use forge_editor::{DocumentBufferSnapshot, EditorHandle};
use forge_workspace::WorkspaceHandle;

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
        let Some(document) = self.workspace.active_document().await? else {
            return Ok(None);
        };

        Ok(Some(self.editor.buffer(document.id()).await?))
    }
}
