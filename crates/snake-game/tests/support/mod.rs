use snake_core::signal::SignalBusBuilder;

use crate::{
    game::{Game, GameSignal},
    infra::audio::RodioAudioClient,
};

pub(crate) fn new_game<HandlerError>(signals: &mut SignalBusBuilder<Game, HandlerError>) -> Game {
    Game::new(
        signals.register::<GameSignal>().unwrap(),
        RodioAudioClient::new().unwrap(),
    )
}
