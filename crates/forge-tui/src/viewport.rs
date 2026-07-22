use crate::cursor::Cursor;

#[derive(Debug, Clone, Copy, Default)]
pub struct Viewport {
    scroll_x: usize,
    scroll_y: usize,
    width: usize,
    height: usize,
}

impl Viewport {
    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    pub fn scroll_x(&self) -> usize {
        self.scroll_x
    }

    pub fn scroll_y(&self) -> usize {
        self.scroll_y
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn ensure_cursor_visible(&mut self, cursor: &Cursor) {
        if self.height == 0 {
            return;
        }

        if cursor.line() < self.scroll_y {
            self.scroll_y = cursor.line();
        } else if cursor.line() >= self.scroll_y + self.height {
            self.scroll_y = cursor.line() - self.height + 1;
        }

        if self.width == 0 {
            return;
        }

        if cursor.column() < self.scroll_x {
            self.scroll_x = cursor.column();
        } else if cursor.column() >= self.scroll_x + self.width {
            self.scroll_x = cursor.column() - self.width + 1;
        }
    }
}
