use std::{
    io::{self, Stdout},
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect};

use crate::{app::TuiApp, error::TuiError, ui};

#[derive(PartialEq)]
pub enum ControlFlow {
    Continue,
    Exit,
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    app: TuiApp,
}

impl Tui {
    pub fn new(app: TuiApp) -> Result<Self, TuiError> {
        enable_raw_mode()?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal, app })
    }

    pub async fn run(&mut self) -> Result<(), TuiError> {
        loop {
            self.render().await?;

            match self.handle_events().await? {
                ControlFlow::Continue => {}
                ControlFlow::Exit => break,
            }
        }

        Ok(())
    }

    async fn handle_events(&mut self) -> Result<ControlFlow, TuiError> {
        let event = tokio::task::spawn_blocking(|| -> std::io::Result<Option<Event>> {
            if !event::poll(Duration::from_millis(50))? {
                return Ok(None);
            }

            Ok(Some(event::read()?))
        })
        .await
        .map_err(TuiError::Join)??;

        let Some(event) = event else {
            return Ok(ControlFlow::Continue);
        };

        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') => {
                    return Ok(ControlFlow::Exit);
                }

                KeyCode::Left | KeyCode::Char('h') => {
                    self.app.move_cursor_left().await?;
                }

                KeyCode::Right | KeyCode::Char('l') => {
                    self.app.move_cursor_right().await?;
                }

                KeyCode::Up | KeyCode::Char('k') => {
                    self.app.move_cursor_up().await?;
                }

                KeyCode::Down | KeyCode::Char('j') => {
                    self.app.move_cursor_down().await?;
                }

                _ => {}
            },

            _ => {}
        }

        Ok(ControlFlow::Continue)
    }

    async fn render(&mut self) -> Result<(), TuiError> {
        let buffer = self.app.active_buffer().await?;

        let terminal_size = self.terminal.size()?;
        let terminal_area = Rect::new(0, 0, terminal_size.width, terminal_size.height);
        let areas = ui::layout(terminal_area);
        let editor_content_area = ui::editor_content_area(areas.editor);

        self.app.editor_state_mut().resize_viewport(
            editor_content_area.width as usize,
            editor_content_area.height as usize,
        );

        let editor_state = self.app.editor_state();

        self.terminal.draw(|frame| {
            ui::render(frame, &areas, buffer.as_ref(), editor_state);
        })?;

        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}
