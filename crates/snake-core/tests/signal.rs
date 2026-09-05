use snake_core::{GameObjectId, signal::SignalBusBuilder};

#[derive(Clone, Copy)]
struct AppleEaten {
    apple_id: GameObjectId,
}

struct GamePauseChanged;
struct GameOver;

#[test]
fn dispatches_an_emitted_signal_to_every_subscriber() {
    #[derive(Default)]
    struct Context {
        apple_ids_received_first: Vec<GameObjectId>,
        apple_ids_received_second: Vec<GameObjectId>,
    }

    let mut signal_bus_builder = SignalBusBuilder::<Context>::new();
    let signal_emitter = signal_bus_builder.register::<AppleEaten>().unwrap();
    signal_bus_builder.on::<AppleEaten>(|signal, context| {
        context.apple_ids_received_first.push(signal.apple_id);
    });
    signal_bus_builder.on::<AppleEaten>(|signal, context| {
        context.apple_ids_received_second.push(signal.apple_id);
    });
    let mut signal_bus = signal_bus_builder.build();
    let mut context = Context::default();
    let apple_id = GameObjectId::new();

    signal_emitter.emit(AppleEaten { apple_id });

    assert!(context.apple_ids_received_first.is_empty());
    assert!(context.apple_ids_received_second.is_empty());

    signal_bus.dispatch_pending(&mut context).unwrap();

    assert_eq!(context.apple_ids_received_first, [apple_id]);
    assert_eq!(context.apple_ids_received_second, [apple_id]);
}

#[test]
fn dispatches_signals_in_emission_order() {
    #[derive(Default)]
    struct Context {
        signals_received: Vec<&'static str>,
    }

    let mut signal_bus_builder = SignalBusBuilder::<Context>::new();
    let game_pause_changed_emitter = signal_bus_builder.register::<GamePauseChanged>().unwrap();
    let game_over_emitter = signal_bus_builder.register::<GameOver>().unwrap();
    let apple_eaten_emitter = signal_bus_builder.register::<AppleEaten>().unwrap();
    signal_bus_builder.on::<GamePauseChanged>(|_, context| {
        context.signals_received.push("game_pause_changed");
    });
    signal_bus_builder.on::<GameOver>(|_, context| {
        context.signals_received.push("game_over");
    });
    signal_bus_builder.on::<AppleEaten>(|_, context| {
        context.signals_received.push("apple_eaten");
    });
    let mut signal_bus = signal_bus_builder.build();
    let mut context = Context::default();

    game_pause_changed_emitter.emit(GamePauseChanged);
    game_over_emitter.emit(GameOver);
    apple_eaten_emitter.emit(AppleEaten {
        apple_id: GameObjectId::new(),
    });
    signal_bus.dispatch_pending(&mut context).unwrap();

    assert_eq!(
        context.signals_received,
        ["game_pause_changed", "game_over", "apple_eaten"]
    );
}

#[test]
fn defers_signals_emitted_by_subscribers_until_the_next_dispatch() {
    #[derive(Default)]
    struct Context {
        signals_received: Vec<&'static str>,
    }

    let mut signal_bus_builder = SignalBusBuilder::<Context>::new();
    let game_pause_changed_emitter = signal_bus_builder.register::<GamePauseChanged>().unwrap();
    let game_over_emitter = signal_bus_builder.register::<GameOver>().unwrap();
    signal_bus_builder.on::<GameOver>({
        let game_pause_changed_emitter = game_pause_changed_emitter.clone();
        move |_, _| game_pause_changed_emitter.emit(GamePauseChanged)
    });
    signal_bus_builder.on::<GameOver>(|_, context| {
        context.signals_received.push("game_over");
    });
    signal_bus_builder.on::<GamePauseChanged>(|_, context| {
        context.signals_received.push("game_pause_changed");
    });
    let mut signal_bus = signal_bus_builder.build();
    let mut context = Context::default();

    game_over_emitter.emit(GameOver);
    signal_bus.dispatch_pending(&mut context).unwrap();

    assert_eq!(context.signals_received, ["game_over"]);

    signal_bus.dispatch_pending(&mut context).unwrap();

    assert_eq!(
        context.signals_received,
        ["game_over", "game_pause_changed"]
    );
}
