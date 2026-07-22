use std::{
    io::{self, Stdout},
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

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
        tokio::task::spawn_blocking(|| {
            if !event::poll(Duration::from_millis(50))? {
                return Ok(ControlFlow::Continue);
            }

            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') => Ok(ControlFlow::Exit),
                    _ => Ok(ControlFlow::Continue),
                },

                _ => Ok(ControlFlow::Continue),
            }
        })
        .await
        .map_err(TuiError::Join)?
    }

    async fn render(&mut self) -> Result<(), TuiError> {
        let buffer = self.app.active_buffer().await?;
        let editor_state = self.app.editor_state();

        self.terminal.draw(|frame| {
            ui::render(frame, buffer.as_ref(), editor_state);
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
