use std::{cell::RefCell, rc::Rc};

use snake_core::{
    GameObjectId,
    signal::{Signal, SignalBus},
};

#[test]
fn dispatches_an_emitted_signal_to_every_subscriber() {
    let mut signal_bus = SignalBus::new();
    let signal_emitter = signal_bus.emitter();
    let apple_id = GameObjectId::new();
    let apple_ids_received_first = Rc::new(RefCell::new(Vec::new()));
    let apple_ids_received_second = Rc::new(RefCell::new(Vec::new()));

    signal_bus.subscribe({
        let apple_ids_received = Rc::clone(&apple_ids_received_first);
        move |signal| {
            if let Signal::AppleEaten { apple_id } = signal {
                apple_ids_received.borrow_mut().push(*apple_id);
            }
        }
    });
    signal_bus.subscribe({
        let apple_ids_received = Rc::clone(&apple_ids_received_second);
        move |signal| {
            if let Signal::AppleEaten { apple_id } = signal {
                apple_ids_received.borrow_mut().push(*apple_id);
            }
        }
    });

    signal_emitter.emit(Signal::AppleEaten { apple_id });

    assert!(apple_ids_received_first.borrow().is_empty());
    assert!(apple_ids_received_second.borrow().is_empty());

    signal_bus.dispatch_pending();

    assert_eq!(*apple_ids_received_first.borrow(), [apple_id]);
    assert_eq!(*apple_ids_received_second.borrow(), [apple_id]);
}

#[test]
fn dispatches_signals_in_emission_order() {
    let mut signal_bus = SignalBus::new();
    let signal_emitter = signal_bus.emitter();
    let signals_received = Rc::new(RefCell::new(Vec::new()));

    signal_bus.subscribe({
        let signals_received = Rc::clone(&signals_received);
        move |signal| {
            let signal_name = match signal {
                Signal::AppleEaten { .. } => "apple_eaten",
                Signal::GamePauseChanged { .. } => "game_pause_changed",
                Signal::GameOver => "game_over",
            };
            signals_received.borrow_mut().push(signal_name);
        }
    });

    signal_emitter.emit(Signal::GamePauseChanged { is_paused: true });
    signal_emitter.emit(Signal::GameOver);
    signal_emitter.emit(Signal::AppleEaten {
        apple_id: GameObjectId::new(),
    });
    signal_bus.dispatch_pending();

    assert_eq!(
        *signals_received.borrow(),
        ["game_pause_changed", "game_over", "apple_eaten"]
    );
}

#[test]
fn defers_signals_emitted_by_subscribers_until_the_next_dispatch() {
    let mut signal_bus = SignalBus::new();
    let signal_emitter = signal_bus.emitter();
    let signals_received = Rc::new(RefCell::new(Vec::new()));

    signal_bus.subscribe({
        let signal_emitter = signal_emitter.clone();
        move |signal| {
            if matches!(signal, Signal::GameOver) {
                signal_emitter.emit(Signal::GamePauseChanged { is_paused: true });
            }
        }
    });
    signal_bus.subscribe({
        let signals_received = Rc::clone(&signals_received);
        move |signal| {
            let signal_name = match signal {
                Signal::GamePauseChanged { .. } => "game_pause_changed",
                Signal::GameOver => "game_over",
                Signal::AppleEaten { .. } => return,
            };
            signals_received.borrow_mut().push(signal_name);
        }
    });

    signal_emitter.emit(Signal::GameOver);
    signal_bus.dispatch_pending();

    assert_eq!(*signals_received.borrow(), ["game_over"]);

    signal_bus.dispatch_pending();

    assert_eq!(
        *signals_received.borrow(),
        ["game_over", "game_pause_changed"]
    );
}
