use std::{
    process::ExitCode,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;
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

pub const fn pool_from_hz(hz: u64) -> Duration {
    Duration::from_millis(1000 / hz)
}

// TODO: make customizable by difficulty ~(o<o)~
const MOVEMENT_INTERVAL: Duration = pool_from_hz(4);

/// 24 fps glory
const FRAMETIME: Duration = pool_from_hz(24);

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
    // TODO: move this to some game state?
    let mut movement_time_accumulated = Duration::ZERO;
    let mut frametime_accumulated = Duration::ZERO;
    // TODO: refactor into read input/ticks and state updates/render fns once stable enough
    let exit_code = 'game: loop {
        let update_time_current = Instant::now();
        let delta_time = update_time_current - update_time_previous;
        update_time_previous = update_time_current;
        movement_time_accumulated = (movement_time_accumulated + delta_time).min(MOVEMENT_INTERVAL);
        frametime_accumulated = (frametime_accumulated + delta_time).min(FRAMETIME);

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
            match input_key {
                InputKey::Move(direction) => snake.borrow_mut().rotate_to(direction),
                InputKey::Quit => break 'game ExitCode::SUCCESS,
            }
        }
        let is_snake_in_apple = snake.borrow().position() == apple.borrow().position();
        if is_snake_in_apple {
            // TODO: using Rc<RefCell<_>> got ugly. see if possible to refactor
            // it while still sharing ref with Game
            snake.borrow_mut().eat(&mut apple.borrow_mut());
        }
        // TODO move tick to after move_forward
        snake.borrow_mut().tick();
        apple.borrow_mut().tick(game_playable_bounds);
        // TODO: test if this an be ==, move this to pure fns
        if movement_time_accumulated >= MOVEMENT_INTERVAL {
            movement_time_accumulated -= MOVEMENT_INTERVAL;
            // TODO: make logic to eat apple
            // apple.borrow_mut().respawn(game_playable_bounds);
            snake.borrow_mut().move_forward(1);
        }

        if frametime_accumulated >= FRAMETIME {
            renderer.render(&game.render_frame())?;
            // TODO: maybe this should zero it?
            frametime_accumulated -= FRAMETIME;
        }
    };

    Ok(exit_code)
}
