#[derive(Debug, Default)]
pub struct EditorState {
    cursor_line: usize,
    cursor_column: usize,
    scroll_x: usize,
    scroll_y: usize,
}

impl EditorState {
    pub fn cursor_line(&self) -> usize {
        self.cursor_line
    }

    pub fn cursor_column(&self) -> usize {
        self.cursor_column
    }

    pub fn scroll_x(&self) -> usize {
        self.scroll_x
    }

    pub fn scroll_y(&self) -> usize {
        self.scroll_y
    }
}
