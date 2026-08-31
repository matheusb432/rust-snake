use std::{cell::RefCell, io, process::ExitCode, rc::Rc};

use crossterm::event::{self, Event};
use snake_core::{
    GameObject, Move,
    models::{apple::Apple, snake::Snake},
};

use crate::{
    game::Game,
    infra::{crossterm_renderer::CrosstermRenderer, input::InputKey, terminal::TerminalSession},
    render::Renderer,
};

mod board;
mod game;
mod infra;
mod render;

fn main() -> io::Result<ExitCode> {
    let _terminal = TerminalSession::start()?;
    let mut game = Game::new();

    let snake = Rc::new(RefCell::new(Snake::spawn()));
    let snake_game_object: Rc<dyn GameObject> = snake.clone();
    // TODO: make snake have renderer textures
    // game.place_object(snake_game_object)
    //     .map_err(io::Error::other)?;
    let game_board_bounds = game.board().bounds();
    let apple = Rc::new(RefCell::new(Apple::spawn(game_board_bounds)));
    game.place_object(apple.clone()).map_err(io::Error::other)?;

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
