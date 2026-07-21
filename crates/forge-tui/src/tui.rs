use std::io::{self, Stdout};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{app::TuiApp, error::TuiError, ui};

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
            let buffer = self.app.active_buffer().await?;

            self.terminal.draw(|frame| {
                ui::render(frame, buffer.as_ref());
            })?;

            tokio::time::sleep(std::time::Duration::from_millis(16)).await;
        }
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}
