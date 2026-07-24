use forge_editor::CursorPosition;

#[derive(Debug, Clone, Copy, Default)]
pub struct Cursor {
    line: usize,
    column: usize,
}

impl Cursor {
    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn position(&self) -> CursorPosition {
        CursorPosition {
            line: self.line,
            column: self.column,
        }
    }

    pub fn move_left(&mut self) {
        self.column = self.column.saturating_sub(1);
    }

    pub fn move_right(&mut self, line_length: usize) {
        self.column = (self.column + 1).min(line_length);
    }

    pub fn move_up(&mut self, target_line_length: usize) {
        self.line = self.line.saturating_sub(1);
        self.column = self.column.min(target_line_length);
    }

    pub fn move_down(&mut self, line_count: usize, target_line_length: usize) {
        if self.line + 1 < line_count {
            self.line += 1;
            self.column = self.column.min(target_line_length);
        }
    }

    pub fn move_to(&mut self, line: usize, column: usize) {
        self.line = line;
        self.column = column;
    }

    pub fn move_right_unbounded(&mut self) {
        self.column += 1;
    }
}
