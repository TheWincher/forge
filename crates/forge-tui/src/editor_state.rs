use forge_editor::EditorMode;

use crate::{cursor::Cursor, viewport::Viewport};

#[derive(Debug, Default)]
pub struct EditorState {
    cursor: Cursor,
    mode: EditorMode,
    viewport: Viewport,
}

impl EditorState {
    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    pub fn resize_viewport(&mut self, width: usize, height: usize) {
        self.viewport.resize(width, height);
        self.viewport.ensure_cursor_visible(&self.cursor);
    }

    pub fn ensure_cursor_visible(&mut self) {
        self.viewport.ensure_cursor_visible(&self.cursor);
    }

    pub fn mode(&self) -> EditorMode {
        self.mode
    }

    pub fn enter_insert_mode(&mut self) {
        self.mode = EditorMode::Insert;
    }

    pub fn enter_normal_mode(&mut self) {
        self.mode = EditorMode::Normal;
    }

    pub fn is_insert(&self) -> bool {
        self.mode == EditorMode::Insert
    }

    pub fn is_normal(&self) -> bool {
        self.mode == EditorMode::Normal
    }
}
