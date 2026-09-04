use std::{
    cell::RefCell,
    process::ExitCode,
    rc::Rc,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use anyhow::{Result, bail};
use crossterm::event::{self, Event};
use snake_core::{
    GameObject, Rotation, TransformRotation,
    models::{apple::Apple, snake::Snake},
    signal::{Signal, SignalBus},
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
    let mut signal_bus = SignalBus::new();
    let signal_emitter = signal_bus.emitter();

    let game = Rc::new(RefCell::new(Game::new(signal_bus.emitter())));
    let game_playable_bounds = game.borrow().playable_bounds();

    // TODO: move startup orchestration to Game, so it can be reused in the signal listener
    let snake = game
        .borrow_mut()
        .insert_object(Snake::spawn(signal_bus.emitter()))?;
    snake
        .borrow_mut()
        .set_position(game_playable_bounds.middle());
    let apple = game
        .borrow_mut()
        .insert_object(Apple::spawn(game_playable_bounds, signal_bus.emitter()))?;

    let mut renderer = CrosstermRenderer::default();
    let (input_tx, input_rx) = mpsc::channel();

    // TODO: move to own game object
    // println!("\renter move ['{}' to quit]: ", InputKey::QUIT_CHARACTER);
    thread::spawn(move || {
        loop {
            if let Ok(Event::Key(key)) = event::read() {
                let _ = input_tx.send(key);
            }
        }
    });
    // TODO: remove if unnecessary
    renderer.render(&game.borrow().render_frame())?;

    signal_bus.subscribe({
        let game = game.clone();
        let snake = snake.clone();
        let apple = apple.clone();

        move |signal| match signal {
            Signal::GameOver => {
                // TODO: render game over text as its own game object
                // println!(
                //     "\r\nGame over! Press '{}' to restart.",
                //     InputKey::RESET_CHARACTER.to_ascii_uppercase()
                // );
            }
            Signal::GameReset => {
                let game_playable_bounds = game.borrow().playable_bounds();
                snake.borrow_mut().respawn(snake_core::Transform {
                    position: game_playable_bounds.middle(),
                    rotation: TransformRotation::default(),
                });
                apple.borrow_mut().respawn(game_playable_bounds);
                let _ = game.borrow_mut().start().inspect_err(|e| {
                    // TODO: add graceful err handling
                    panic!("{}", e);
                });
            }
            Signal::SnakeKilled { .. } => {
                game.borrow_mut().to_game_over();
            }
            _ => {}
        }
    });

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
            game.borrow_mut()
                .queue_input(InputKey::from_keycode(key.code));
        }

        let input_keys = game.borrow_mut().flush_input();
        for input_key in input_keys {
            let game_state = game.borrow().state();
            match (input_key, game_state) {
                (input_key, GameState::GameOver) => {
                    if let InputKey::Reset = input_key {
                        game.borrow_mut().reset();
                    }
                    break;
                }
                (InputKey::Move(direction), game_state) => {
                    match game_state {
                        GameState::NotStarted => game.borrow_mut().start()?,
                        GameState::Paused => game.borrow_mut().unpause()?,
                        GameState::InGame => (),
                        GameState::GameOver => unreachable!(),
                    }
                    // TODO: try to make it not be necessary to tick?
                    snake.borrow_mut().tick(Duration::ZERO, Some(direction));
                }
                (InputKey::Pause, _) => {
                    game.borrow_mut().toggle_pause();
                }
                (InputKey::Quit, _) => break 'game ExitCode::SUCCESS,
                (InputKey::Reset, GameState::InGame) => {
                    game.borrow_mut().reset();
                }
                (InputKey::Reset, _) => {}
            }
        }

        // TODO: move to state update ticks fn
        if game.borrow().state() == GameState::InGame {
            snake.borrow_mut().tick(delta_time, None);

            let is_snake_in_apple = snake.borrow().position() == apple.borrow().position();
            if is_snake_in_apple {
                // TODO: using Rc<RefCell<_>> got ugly. see if possible to refactor
                // it while still sharing ref with Game
                snake.borrow_mut().eat(&mut apple.borrow_mut());
            }
            apple.borrow_mut().tick(game_playable_bounds);
        }

        signal_bus.dispatch_pending();

        if let Some(frame) = game.borrow_mut().render_frame_if_due(delta_time) {
            renderer.render(&frame)?;
        }
    };

    Ok(exit_code)
}
