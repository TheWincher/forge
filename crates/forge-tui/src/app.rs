use forge_editor::{BackspaceResult, DocumentBufferSnapshot, EditorError, EditorHandle};
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

    pub async fn move_cursor_left(&mut self) -> Result<(), TuiError> {
        self.editor_state.cursor_mut().move_left();
        Ok(())
    }

    pub async fn move_cursor_right(&mut self) -> Result<(), TuiError> {
        let Some(buffer) = self.active_buffer().await? else {
            return Ok(());
        };

        let cursor = self.editor_state.cursor();
        let line_length = buffer
            .content
            .lines()
            .nth(cursor.line())
            .map(str::chars)
            .map(Iterator::count)
            .unwrap_or(0);

        self.editor_state.cursor_mut().move_right(line_length);

        Ok(())
    }

    pub async fn move_cursor_up(&mut self) -> Result<(), TuiError> {
        let Some(buffer) = self.active_buffer().await? else {
            return Ok(());
        };

        let target_line = self.editor_state.cursor().line().saturating_sub(1);

        let target_line_length = buffer
            .content
            .lines()
            .nth(target_line)
            .map(str::chars)
            .map(Iterator::count)
            .unwrap_or(0);

        self.editor_state.cursor_mut().move_up(target_line_length);
        self.editor_state.ensure_cursor_visible();

        Ok(())
    }

    pub async fn move_cursor_down(&mut self) -> Result<(), TuiError> {
        let Some(buffer) = self.active_buffer().await? else {
            return Ok(());
        };

        let lines: Vec<&str> = buffer.content.lines().collect();
        let line_count = lines.len().max(1);

        let target_line = (self.editor_state.cursor().line() + 1).min(line_count - 1);

        let target_line_length = lines
            .get(target_line)
            .map(|line| line.chars().count())
            .unwrap_or(0);

        self.editor_state
            .cursor_mut()
            .move_down(line_count, target_line_length);

        self.editor_state.ensure_cursor_visible();

        Ok(())
    }

    pub async fn insert_character(&mut self, character: char) -> Result<(), TuiError> {
        let Some(buffer) = self.active_buffer().await? else {
            return Ok(());
        };

        let document_id = buffer.document_id;
        let line = self.editor_state.cursor().line();
        let column = self.editor_state.cursor().column();

        let inserted = self
            .editor
            .insert_character(document_id, line, column, character)
            .await?;

        if inserted {
            self.editor_state.cursor_mut().move_right_unbounded();
            self.editor_state.ensure_cursor_visible();
        }

        Ok(())
    }

    pub async fn backspace(&mut self) -> Result<(), TuiError> {
        let Some(buffer) = self.active_buffer().await? else {
            return Ok(());
        };

        let line = self.editor_state.cursor().line();
        let column = self.editor_state.cursor().column();

        let result = self
            .editor
            .backspace(buffer.document_id, line, column)
            .await?;

        match result {
            BackspaceResult::Noop => {}

            BackspaceResult::CharacterDeleted => {
                self.editor_state.cursor_mut().move_left();
            }

            BackspaceResult::LinesJoined { line, column } => {
                self.editor_state.cursor_mut().move_to(line, column);
            }
        }

        self.editor_state.ensure_cursor_visible();

        Ok(())
    }
}
