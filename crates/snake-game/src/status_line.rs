use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use snake_core::{
    TextureColor, Vector2Int,
    models::{scorebar::Scorebar, snake::SnakeSignal, text::Text},
    signal::SignalBusBuilder,
};

use crate::{
    game::{Game, GameSignal, GameState},
    infra::input::InputKey,
};

pub(crate) fn register_status_line(
    game: &mut Game,
    signals: &mut SignalBusBuilder<Game, anyhow::Error>,
) -> Result<()> {
    let position = game.board().screen_position_below(0);
    let game_over = format!(
        "Game over! Press '{}' to restart.",
        InputKey::RESET_CHARACTER.to_ascii_uppercase()
    );
    let won = format!(
        "You won! Press '{}' to restart.",
        InputKey::RESET_CHARACTER.to_ascii_uppercase()
    );
    for (content, state, color) in [
        (
            "Press any key to start",
            GameState::NotStarted,
            TextureColor::White,
        ),
        ("Paused", GameState::Paused, TextureColor::White),
        (game_over.as_str(), GameState::GameOver, TextureColor::Red),
        (won.as_str(), GameState::Won, TextureColor::Green),
    ] {
        let mut text = Text::new(position, content, color);
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

    let scorebar_position = game.board().screen_position_right(Vector2Int::new(1, 1));
    let scorebar = Rc::new(RefCell::new(Scorebar::new(scorebar_position)));
    let scorebar_reference = Rc::downgrade(&scorebar);
    game.insert_object(scorebar)?;
    signals.on::<SnakeSignal>({
        let scorebar_reference = scorebar_reference.clone();
        move |signal, _| {
            let Some(scorebar) = scorebar_reference.upgrade() else {
                return;
            };
            if let SnakeSignal::FoodEaten { .. } = *signal {
                scorebar.borrow_mut().score.add_unit();
            }
        }
    });
    signals.on::<GameSignal>(move |signal, _| {
        let Some(scorebar) = scorebar_reference.upgrade() else {
            return;
        };
        match signal {
            GameSignal::Reset => {
                // the idea to not save score for resetting, but to save if when dying (on .Over) is
                // to make death a bit less frustrating
                scorebar.borrow_mut().reset();
            }
            GameSignal::Won | GameSignal::Over => {
                scorebar.borrow_mut().save_and_reset();
            }
            GameSignal::PauseChanged { .. } | GameSignal::Started => {}
        }
    });
    Ok(())
}
