use std::{io, process::ExitCode};

use crossterm::event::{self, Event};
use snake_core::{
    Move,
    models::{apple::Apple, snake::Snake},
};

use crate::{
    game::{Game, InsertObjectError},
    infra::{crossterm_renderer::CrosstermRenderer, input::InputKey, terminal::TerminalSession},
    render::Renderer,
};

mod board;
mod game;
mod infra;
mod render;

impl From<InsertObjectError> for io::Error {
    fn from(error: InsertObjectError) -> Self {
        Self::other(error)
    }
}

fn main() -> io::Result<ExitCode> {
    let _terminal = TerminalSession::start()?;
    let mut game = Game::new();
    let game_board_bounds = game.board().bounds();

    let snake = game.insert_object(Snake::spawn())?;
    let apple = game.insert_object(Apple::spawn(game_board_bounds))?;

    let mut renderer = CrosstermRenderer::default();
    println!("\renter move ['{}' to quit]: ", InputKey::QUIT_CHARACTER);
    renderer.render(game.board(), game.objects())?;

    let exit_code = loop {
        let Event::Key(key) = event::read()? else {
            continue;
        };

        match InputKey::from_keycode(key.code) {
            Some(InputKey::Move(direction)) => {
                snake.borrow_mut().rotate_to(direction);
                apple.borrow_mut().respawn(game_board_bounds);
                renderer.render(game.board(), game.objects())?;
            }
            Some(InputKey::Quit) => break ExitCode::SUCCESS,
            None => {}
        }
    };

    Ok(exit_code)
}
