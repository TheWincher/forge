use forge_editor::DocumentBufferSnapshot;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
};

use crate::editor_state::EditorState;

pub fn render(
    frame: &mut Frame,
    buffer: Option<&DocumentBufferSnapshot>,
    editor_state: &EditorState,
) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, areas[0], buffer);
    render_editor(frame, areas[1], buffer, editor_state);
    render_status_bar(frame, areas[2], buffer);
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
    area: ratatui::layout::Rect,
    buffer: Option<&DocumentBufferSnapshot>,
    editor_state: &EditorState,
) {
    let content = match buffer {
        Some(buffer) => Text::from(buffer.content.to_string()),
        None => Text::from(Line::from("Aucun document ouvert")),
    };

    let editor =
        Paragraph::new(content).block(Block::default().borders(Borders::LEFT | Borders::RIGHT));

    frame.render_widget(editor, area);

    if buffer.is_some() {
        let cursor_x = area.x + 1 + editor_state.cursor_column() as u16;

        let cursor_y = area.y + editor_state.cursor_line() as u16;

        if cursor_x < area.right() && cursor_y < area.bottom() {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}

fn render_status_bar(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    buffer: Option<&DocumentBufferSnapshot>,
) {
    let status = match buffer {
        Some(buffer) => {
            let dirty = if buffer.dirty { " [+]" } else { "" };

            format!(
                " NORMAL | {}{} | version {} | q: quitter ",
                buffer.path.display(),
                dirty,
                buffer.version,
            )
        }
        None => " NORMAL | aucun document | q: quitter ".to_string(),
    };

    frame.render_widget(Paragraph::new(status), area);
}
