use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use snake_core::{Vector2Int, models::text::Text, signal::SignalBusBuilder};

use crate::{
    board::BOARD_SIZE_Y,
    game::{Game, GameSignal, GameState},
    infra::input::InputKey,
};

mod scorebar;

pub(crate) fn register_status_line(
    game: &mut Game,
    signals: &mut SignalBusBuilder<Game, anyhow::Error>,
) -> Result<()> {
    let position = Vector2Int::new(0, BOARD_SIZE_Y as i32);
    let game_over = format!(
        "Game over! Press '{}' to restart.",
        InputKey::RESET_CHARACTER.to_ascii_uppercase()
    );
    let won = format!(
        "You won! Press '{}' to restart.",
        InputKey::RESET_CHARACTER.to_ascii_uppercase()
    );
    for (content, state) in [
        ("Press any key to start", GameState::NotStarted),
        ("Paused", GameState::Paused),
        (game_over.as_str(), GameState::GameOver),
        (won.as_str(), GameState::Won),
    ] {
        let mut text = Text::new(position, content);
        text.set_visible(game.state() == state);
        let text = Rc::new(RefCell::new(text));
        let text_reference = Rc::downgrade(&text);
        game.insert_object(text)?;
        signals.on::<GameSignal>(move |_, game| {
            if let Some(text) = text_reference.upgrade() {
                text.borrow_mut().set_visible(game.state() == state);
            }
        });
    }
    // TODO: uncomment once scorebar is impl
    // game.insert_object(Rc::new(RefCell::new(scorebar::Scorebar::new(position))))?;
    Ok(())
}
