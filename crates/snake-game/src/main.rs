use std::{
    fmt::Write as _,
    io::{self},
    process::{ExitCode, Termination},
    sync::mpsc,
    thread,
};

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode},
    execute,
    style::Print,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use snake_core::{input::InputKey, models::snake::Snake};

use crate::render::{BOARD_SIZE_X, BOARD_SIZE_Y, Board};

mod assets;
mod render;

pub const QUIT: char = 'q';

fn main() -> io::Result<ExitCode> {
    let game = Game::start()?;
    let board = Board::new();
    // TODO: impl the newtypes and instantiate the snek
    // let mut snake = Snake {hp:};
    let mut snake = Snake::spawn();
    // let mut snake_hp = 5_u32;

    // TODO: use newtype for directional inputs
    let (input_tx, input_rx) = std::sync::mpsc::channel();

    println!("\renter move ['{QUIT}' to quit]: ");
    thread::spawn(move || {
        loop {
            if let Ok(Event::Key(key)) = event::read() {
                // best effort to send key
                let _ = input_tx.send(key);
            }
        }
    });

    let exit_code = loop {
        // TODO: change to try_recv() after debugging board
        let input_key: Option<InputKey> = match input_rx.recv() {
            Ok(key) => {
                if let Some(input_keycode) = input_from_keycode(key.code) {
                    Some(input_keycode)
                } else {
                    match key.code {
                        KeyCode::Char(QUIT) => break game.exit(),
                        _ => None,
                    }
                }
            }
            // Err(mpsc::TryRecvError::Empty) => None,
            Err(err) => {
                eprintln!("\r{}", err);
                break ExitCode::FAILURE;
            }
        };
        if let Some(input_key) = input_key {
            println!("\rmove: {}", input_key);
            snake.move_to(input_key);
            let shd = snake.head_direction();
            println!("\rsnake head direction: {}", shd.into_inner());
        };

        let mut buffer = String::with_capacity(board.size() + BOARD_SIZE_Y * 2);

        for b_y in 0..BOARD_SIZE_Y {
            for b_x in 0..BOARD_SIZE_X {
                let _ = write!(&mut buffer, "{}", board.get_texture(b_x, b_y));
            }

            if b_y + 1 < BOARD_SIZE_Y {
                buffer.push_str("\r\n");
            }
        }

        execute!(io::stdout(), MoveTo(0, 0), Print(buffer))?;
    };

    Ok(exit_code)
}

pub fn input_from_keycode(key_code: KeyCode) -> Option<InputKey> {
    match key_code {
        KeyCode::Up => Some(InputKey::Up),
        KeyCode::Down => Some(InputKey::Down),
        KeyCode::Left => Some(InputKey::Left),
        KeyCode::Right => Some(InputKey::Right),
        _ => None,
    }
}

struct Game;
impl Game {
    pub fn start() -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0))?;
        Ok(Self)
    }

    /// drops `Game`, which in turn calls its `Drop` impl to exit the process
    pub fn exit(self) -> ExitCode {
        self.report()
    }
}
impl Drop for Game {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        println!("\rexiting game...");
    }
}
impl Termination for Game {
    fn report(self) -> ExitCode {
        ExitCode::SUCCESS
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroI64;

    #[test]
    fn some_test() {
        let nzu: NonZeroI64 = 5.try_into().unwrap();
        let my_i: i64 = nzu.into();

        assert_eq!(5, my_i);
    }
}
