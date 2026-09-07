use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use snake_core::{
    Vector2Int,
    models::{snake::SnakeSignal, text::Text},
    signal::SignalBusBuilder,
};

use crate::{
    board::{BOARD_SIZE_X, BOARD_SIZE_Y},
    game::{Game, GameSignal, GameState},
    infra::input::InputKey,
    status_line::scorebar::Scorebar,
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

    let scorebar_position =
        Vector2Int::new(((BOARD_SIZE_X - 1) * 2) as i32, BOARD_SIZE_Y as i32 + 1);
    let scorebar = Rc::new(RefCell::new(Scorebar::new(scorebar_position)));
    let scorebar_reference = Rc::downgrade(&scorebar);
    game.insert_object(scorebar)?;
    signals.on::<SnakeSignal>({
        move |signal, _game| {
            let Some(scorebar) = scorebar_reference.upgrade() else {
                return;
            };
            if let SnakeSignal::FoodEaten { .. } = *signal {
                scorebar.borrow_mut().score.add_unit();
            }
        }
    });
    Ok(())
}
