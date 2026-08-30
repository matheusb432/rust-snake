use std::{cell::RefCell, io, process::ExitCode, rc::Rc, sync::mpsc, thread};

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
    let apple = Rc::new(RefCell::new(Apple::new()));
    game.place_object(apple.clone()).map_err(io::Error::other)?;

    let mut renderer = CrosstermRenderer::default();

    let (input_tx, input_rx) = mpsc::channel();

    println!("\renter move ['{}' to quit]: ", InputKey::QUIT_CHARACTER);
    thread::spawn(move || {
        loop {
            if let Ok(Event::Key(key)) = event::read() {
                let _ = input_tx.send(key);
            }
        }
    });

    let exit_code = loop {
        let input_key = match input_rx.try_recv() {
            Ok(key) => InputKey::from_keycode(key.code),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(error) => {
                eprintln!("\r{error}");
                break ExitCode::FAILURE;
            }
        };

        let game_board = game.board();
        let game_board_bounds = game_board.get_upper_bounds();
        match input_key {
            Some(InputKey::Move(direction)) => {
                snake.borrow_mut().rotate_to(direction);
                apple.borrow_mut().be_eaten(game_board_bounds);
            }
            Some(InputKey::Quit) => break ExitCode::SUCCESS,
            None => {}
        }

        renderer.render(game.board(), game.objects())?;
    };

    Ok(exit_code)
}
