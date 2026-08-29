use std::{
    io,
    process::{ExitCode, Termination},
    sync::mpsc,
    thread,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use snake_core::{input::InputKey, models::snake::Snake};

use crate::{
    game::Game,
    input::InputKey,
    render::{Board, TerminalProjection, render_board},
};

mod assets;
mod game;
mod input;
mod render;

pub const QUIT: char = 'q';

fn main() -> io::Result<ExitCode> {
    let game = Game::start()?;
    let board = Board::new();
    let mut snake = Snake::spawn();
    let mut output = io::stdout();

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
        let input_key: Option<InputKey> = match input_rx.try_recv() {
            Ok(key) => {
                if let Some(input_keycode) = InputKey::from_keycode(key.code) {
                    Some(input_keycode)
                } else {
                    match key.code {
                        KeyCode::Char(QUIT) => break game.exit(),
                        _ => None,
                    }
                }
            }
            Err(mpsc::TryRecvError::Empty) => None,
            Err(err) => {
                eprintln!("\r{}", err);
                break ExitCode::FAILURE;
            }
        };
        if let Some(input_key) = input_key {
            snake.move_to(input_key);
            let shd = snake.head_direction();
            println!("\r\nsnake head direction: {}", shd.into_inner());
        };

        // TODO: mutate board textures b4 rendering
        render_board(
            &mut output,
            &board,
            TerminalProjection::DOUBLE_WIDTH_INTERIOR,
        )?;
    };

    Ok(exit_code)
}
