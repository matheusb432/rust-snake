use std::{io, process::ExitCode, thread, time::Duration};

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

const POOLING_4HZ_MS: u64 = 1000 / 4;

fn main() -> io::Result<ExitCode> {
    let _terminal = TerminalSession::start()?;
    let mut game = Game::new();
    let game_playable_bounds = game.playable_bounds();

    let snake = game.insert_object(Snake::spawn())?;
    snake
        .borrow_mut()
        .set_position(game_playable_bounds.middle());
    // snake
    //     .borrow_mut()
    //     .set_position(snake_core::Vector2Int { x: 10, y: 10 });
    let _apple = game.insert_object(Apple::spawn(game_playable_bounds))?;

    let mut renderer = CrosstermRenderer::default();
    println!("\renter move ['{}' to quit]: ", InputKey::QUIT_CHARACTER);
    renderer.render(&game.render_frame())?;

    let exit_code = loop {
        if let Event::Key(key) = event::read()? {
            match InputKey::from_keycode(key.code) {
                Some(InputKey::Move(direction)) => {
                    snake.borrow_mut().rotate_to(direction);
                }
                Some(InputKey::Quit) => break ExitCode::SUCCESS,
                None => {}
            }
        }

        snake.borrow_mut().tick();
        // TODO: make logic to eat apple
        // apple.borrow_mut().respawn(game_playable_bounds);
        renderer.render(&game.render_frame())?;
        // TODO: implement  pooling for 4 ticks per sec
        // thread::sleep(Duration::from_millis(POOLING_4HZ_MS));
    };

    Ok(exit_code)
}
