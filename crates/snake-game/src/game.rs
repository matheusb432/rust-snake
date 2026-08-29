use std::{
    collections::HashMap,
    io,
    process::{ExitCode, Termination},
};

use crossterm::{cursor, terminal};
use snake_core::game_object::{GameObject, GameObjectId};

use crate::render::Board;

pub(crate) struct Game {
    objects: HashMap<GameObjectId, dyn GameObject>,
    // TODO enable moving/painting to it via Game
    board: GameBoard,
}
impl Game {
    pub fn start() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(
            io::stdout(),
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        Ok(Self {
            objects: HashMap::default(),
            board: Board::new(),
        })
    }

    /// drops `Game`, which in turn calls its `Drop` impl to exit the process
    pub fn exit(self) -> ExitCode {
        self.report()
    }
}
impl Drop for Game {
    fn drop(&mut self) {
        let _ = crossterm::execute!(io::stdout(), cursor::Show);
        let _ = terminal::disable_raw_mode();
        println!("\r\nexiting game...");
    }
}
impl Termination for Game {
    fn report(self) -> ExitCode {
        ExitCode::SUCCESS
    }
}
