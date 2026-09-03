use std::{
    process::ExitCode,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;
use crossterm::event::{self, Event};
use snake_core::{
    GameObject,
    models::{apple::Apple, snake::Snake},
};

use crate::{
    game::{Game, GameState},
    infra::{crossterm_renderer::CrosstermRenderer, input::InputKey, terminal::TerminalSession},
    render::Renderer,
};

mod board;
mod game;
mod infra;
mod render;

fn main() -> Result<ExitCode> {
    let _terminal = TerminalSession::start()?;
    let mut game = Game::new();
    let game_playable_bounds = game.playable_bounds();

    let snake = game.insert_object(Snake::spawn())?;
    snake
        .borrow_mut()
        .set_position(game_playable_bounds.middle());
    let apple = game.insert_object(Apple::spawn(game_playable_bounds))?;

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
    renderer.render(&game.render_frame())?;

    let mut update_time_previous = Instant::now();
    let exit_code = 'game: loop {
        let update_time = Instant::now();
        let delta_time = update_time - update_time_previous;
        update_time_previous = update_time;

        let first_key = match input_rx.try_recv() {
            Ok(key) => Some(key),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(error) => {
                eprintln!("\r{error}");
                break ExitCode::FAILURE;
            }
        };

        if let Some(key) = first_key {
            game.queue_input(InputKey::from_keycode(key.code));
        }

        for input_key in game.flush_input() {
            // TODO: review if states need to get more elaborate
            match (input_key, game.state()) {
                (InputKey::Move(direction), game_state) => {
                    match game_state {
                        GameState::NotStarted => game.start()?,
                        GameState::Paused => game.unpause()?,
                        GameState::InGame => (),
                    }
                    snake.borrow_mut().tick(Duration::ZERO, Some(direction));
                }
                (InputKey::Pause, _) => {
                    game.toggle_pause();
                }
                (InputKey::Quit, _) => break 'game ExitCode::SUCCESS,
                (InputKey::Reset, GameState::InGame) => {
                    todo!("reset game state and respawn actors")
                }
                (InputKey::Reset, _) => {}
            }
        }

        // TODO: move to state update ticks fn
        if game.state() == GameState::InGame {
            {
                let mut snake = snake.borrow_mut();
                snake.tick(delta_time, None);
                if snake.part_collides_with_head() {
                    snake.kill();
                    println!(
                        "\r\nGame over! Press '{}' to restart.",
                        InputKey::RESET_CHARACTER.to_ascii_uppercase()
                    );
                }
            }

            let is_snake_in_apple = snake.borrow().position() == apple.borrow().position();
            if is_snake_in_apple {
                // TODO: using Rc<RefCell<_>> got ugly. see if possible to refactor
                // it while still sharing ref with Game
                snake.borrow_mut().eat(&mut apple.borrow_mut());
            }
            apple.borrow_mut().tick(game_playable_bounds);
        }

        if let Some(frame) = game.render_frame_if_due(delta_time) {
            renderer.render(&frame)?;
        }
    };

    Ok(exit_code)
}
