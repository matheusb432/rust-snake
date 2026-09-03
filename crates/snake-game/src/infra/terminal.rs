use std::io::stdout;

use anyhow::{Context, Result};
use crossterm::{cursor, terminal};

pub(crate) struct TerminalSession;

impl TerminalSession {
    pub fn start() -> Result<Self> {
        terminal::enable_raw_mode().context("failed to enable terminal raw mode")?;

        if let Err(error) = crossterm::execute!(
            stdout(),
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        ) {
            let _ = terminal::disable_raw_mode();
            return Err(error).context("failed to initialize the terminal display");
        }

        Ok(Self)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = crossterm::execute!(stdout(), cursor::Show);
        let _ = terminal::disable_raw_mode();
        println!("\r\nexiting game...");
    }
}
