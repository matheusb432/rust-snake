use std::io;

use crossterm::{cursor, terminal};

pub(crate) struct TerminalSession;

impl TerminalSession {
    pub fn start() -> io::Result<Self> {
        terminal::enable_raw_mode()?;

        if let Err(error) = crossterm::execute!(
            io::stdout(),
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        ) {
            let _ = terminal::disable_raw_mode();
            return Err(error);
        }

        Ok(Self)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = crossterm::execute!(io::stdout(), cursor::Show);
        let _ = terminal::disable_raw_mode();
        println!("\r\nexiting game...");
    }
}
