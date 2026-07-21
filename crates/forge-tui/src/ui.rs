use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
};

use forge_editor::DocumentBufferSnapshot;

pub fn render(frame: &mut Frame, buffer: Option<&DocumentBufferSnapshot>) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let title = buffer
        .and_then(|buffer| buffer.path.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("Forge");

    frame.render_widget(Paragraph::new(title), areas[0]);

    let content = buffer
        .map(|buffer| buffer.content.as_str())
        .unwrap_or("No document open");

    frame.render_widget(
        Paragraph::new(content).block(Block::default().borders(Borders::ALL)),
        areas[1],
    );

    let status = buffer
        .map(|buffer| {
            format!(
                "{} | version {}",
                if buffer.dirty { "modified" } else { "saved" },
                buffer.version
            )
        })
        .unwrap_or_default();

    frame.render_widget(Paragraph::new(status), areas[2]);
}
