use std::{process::ExitCode, sync::mpsc, thread, time::Instant};

use anyhow::Result;
use crossterm::event::{self, Event};
use snake_core::signal::SignalBusBuilder;

use crate::{
    game::{Game, GameSignal, GameUpdate},
    infra::{
        audio::RodioAudioClient, crossterm_renderer::CrosstermRenderer, input::InputKey,
        terminal::TerminalSession,
    },
    render::Renderer,
    scene::register_snake_scene,
};

mod board;
mod game;
mod infra;
mod render;
mod scene;

#[cfg(test)]
#[path = "../tests/support/mod.rs"]
mod test_utils;

fn main() -> Result<ExitCode> {
    let _terminal = TerminalSession::start()?;
    let mut signals = SignalBusBuilder::new();
    let mut game = Game::new(signals.register::<GameSignal>()?, RodioAudioClient::new()?);

    register_snake_scene(&mut game, &mut signals)?;

    let mut signal_bus = signals.build();
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

    let mut update_time_previous = Instant::now();
    let exit_code = loop {
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

        if game.update(delta_time, &mut signal_bus)? == GameUpdate::Quit {
            break ExitCode::SUCCESS;
        }
        if let Some(frame) = game.render_frame_if_due(delta_time) {
            renderer.render(&frame)?;
        }
    };

    Ok(exit_code)
}
