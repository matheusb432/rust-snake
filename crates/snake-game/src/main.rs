use std::{cell::RefCell, io, process::ExitCode, rc::Rc, sync::mpsc, thread};

use crossterm::event::{self, Event};
use snake_core::{GameObject, Move, models::snake::Snake};

use crate::{
    game::Game, infra::crossterm_renderer::CrosstermRenderer, input::InputKey, render::Renderer,
    terminal::TerminalSession,
};

mod assets;
mod board;
mod game;
mod infra;
mod input;
mod render;
mod terminal;

// TODO: move to core
// struct Apple{}
// impl GameObject for Apple{
//     fn rotation(&self) -> snake_core::Rotation {
//         todo!()
//     }

//     fn id(&self) -> snake_core::GameObjectId {
//         todo!()
//     }
// }

fn main() -> io::Result<ExitCode> {
    let _terminal = TerminalSession::start()?;
    let mut game = Game::new();

    let snake = Rc::new(RefCell::new(Snake::spawn()));
    let snake_game_object: Rc<dyn GameObject> = snake.clone();
    // TODO: make snake have renderer textures
    game.place_object(snake_game_object)
        .map_err(io::Error::other)?;
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

        match input_key {
            Some(InputKey::Move(direction)) => {
                snake.borrow_mut().move_to(direction);
            }
            Some(InputKey::Quit) => break ExitCode::SUCCESS,
            None => {}
        }

        renderer.render(game.board(), game.objects())?;
    };

    Ok(exit_code)
}
