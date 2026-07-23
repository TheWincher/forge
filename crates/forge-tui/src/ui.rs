use forge_editor::DocumentBufferSnapshot;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
};

use crate::editor_state::EditorState;

pub struct LayoutAreas {
    pub header: Rect,
    pub editor: Rect,
    pub status: Rect,
}

pub fn layout(area: Rect) -> LayoutAreas {
    let areas = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(area);

    LayoutAreas {
        header: areas[0],
        editor: areas[1],
        status: areas[2],
    }
}

pub fn editor_content_area(area: Rect) -> Rect {
    Block::default().borders(Borders::ALL).inner(area)
}

pub fn render(
    frame: &mut Frame,
    layout: &LayoutAreas,
    buffer: Option<&DocumentBufferSnapshot>,
    editor_state: &EditorState,
) {
    render_header(frame, layout.header, buffer);
    render_editor(frame, layout.editor, buffer, editor_state);
    render_status_bar(frame, layout.status, buffer, editor_state);
}

fn render_header(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    buffer: Option<&DocumentBufferSnapshot>,
) {
    let title = match buffer {
        Some(buffer) => format!(" Forge — {} ", buffer.path.display()),
        None => " Forge ".to_string(),
    };

    let header = Block::default().title(title).borders(Borders::ALL);

    frame.render_widget(header, area);
}

fn render_editor(
    frame: &mut Frame,
    area: Rect,
    buffer: Option<&DocumentBufferSnapshot>,
    editor_state: &EditorState,
) {
    let block = Block::default().borders(Borders::ALL).title(" Editor ");

    let inner_area = block.inner(area);

    frame.render_widget(block, area);

    let Some(buffer) = buffer else {
        return;
    };

    let viewport = editor_state.viewport();

    let visible_content = buffer
        .content
        .lines()
        .skip(viewport.scroll_y())
        .take(inner_area.height as usize)
        .map(|line| {
            line.chars()
                .skip(viewport.scroll_x())
                .take(inner_area.width as usize)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    frame.render_widget(Paragraph::new(visible_content), inner_area);

    let cursor = editor_state.cursor();

    let cursor_x = inner_area.x + cursor.column().saturating_sub(viewport.scroll_x()) as u16;

    let cursor_y = inner_area.y + cursor.line().saturating_sub(viewport.scroll_y()) as u16;

    if cursor_x < inner_area.right() && cursor_y < inner_area.bottom() {
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn render_status_bar(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    buffer: Option<&DocumentBufferSnapshot>,
    editor_state: &EditorState,
) {
    let status = match buffer {
        Some(buffer) => {
            let dirty = if buffer.dirty { " [+]" } else { "" };

            format!(
                " {:?} | {}{} | version {} | q: quitter ",
                editor_state.mode(),
                buffer.path.display(),
                dirty,
                buffer.version,
            )
        }
        None => " NORMAL | aucun document | q: quitter ".to_string(),
    };

    frame.render_widget(Paragraph::new(status), area);
}
