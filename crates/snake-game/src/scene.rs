use std::{cell::RefCell, rc::Rc};

use anyhow::{Result, anyhow};
use snake_core::{
    GameObject, Transform, TransformRotation,
    models::{
        apple::{Apple, AppleSignal},
        snake::{Snake, SnakeSignal},
    },
    random::RandomSource,
    signal::SignalBusBuilder,
};

use crate::{
    board::Board,
    game::{Game, GameSignal},
    infra::{audio::Sound, input::InputKey},
    status_line::register_status_line,
};

/// registers and manages snake scene state.
pub(crate) fn register_snake_scene(
    game: &mut Game,
    signals: &mut SignalBusBuilder<Game, anyhow::Error>,
    random: impl RandomSource + 'static,
) -> Result<()> {
    let random: Rc<RefCell<dyn RandomSource>> = Rc::new(RefCell::new(random));
    let mut snake = Snake::spawn(signals.register::<SnakeSignal>()?);
    snake.set_position(game.playable_bounds().middle());
    let position = Board::vacant_position(snake.occupied_cells(), &mut *random.borrow_mut())
        .ok_or_else(|| anyhow!("board has no vacant cell for the initial apple"))?;
    let apple = Rc::new(RefCell::new(Apple::spawn(
        position,
        signals.register::<AppleSignal>()?,
    )));
    register_snake_objects(game, signals, Rc::new(RefCell::new(snake)), apple, random)?;
    register_status_line(game, signals)
}

/// registers game objects and signal listeners
fn register_snake_objects(
    game: &mut Game,
    signals: &mut SignalBusBuilder<Game, anyhow::Error>,
    snake: Rc<RefCell<Snake>>,
    apple: Rc<RefCell<Apple>>,
    random: Rc<RefCell<dyn RandomSource>>,
) -> Result<()> {
    let snake_id = snake.borrow().id();
    let apple_id = apple.borrow().id();
    let snake_reference = Rc::downgrade(&snake);
    let apple_reference = Rc::downgrade(&apple);
    game.insert_object(snake)?;
    game.insert_object(apple)?;

    game.on_input({
        let snake = snake_reference.clone();
        move |input| {
            if let (InputKey::Move(direction), Some(snake)) = (input, snake.upgrade()) {
                snake.borrow_mut().request_movement(direction);
            }
        }
    });
    game.on_update({
        let snake = snake_reference.clone();
        move |delta_time, _| {
            if let Some(snake) = snake.upgrade() {
                snake.borrow_mut().tick(delta_time);
            }
        }
    });
    game.on_collision_check({
        let snake = snake_reference.clone();
        let apple = apple_reference.clone();
        move |bounds| {
            if let Some(snake) = snake.upgrade() {
                let food = apple
                    .upgrade()
                    .and_then(|apple| apple.borrow().collision_cell());
                snake.borrow_mut().resolve_collisions(bounds, food);
            }
        }
    });
    signals.on::<SnakeSignal>({
        let apple = apple_reference.clone();
        move |signal, _| {
            if let Some(apple) = apple.upgrade() {
                apple.borrow_mut().on_snake_signal(signal);
            }
        }
    });
    signals.on::<AppleSignal>({
        let snake = snake_reference.clone();
        let apple = apple_reference.clone();
        let random = random.clone();
        move |signal, game| match signal {
            AppleSignal::Eaten { apple_id: eaten_id } => {
                if *eaten_id == apple_id
                    && let (Some(snake), Some(apple)) = (snake.upgrade(), apple.upgrade())
                {
                    let position = Board::vacant_position(
                        snake.borrow().occupied_cells(),
                        &mut *random.borrow_mut(),
                    );
                    match position {
                        Some(position) => apple.borrow_mut().respawn(position),
                        None => game.win(),
                    }
                    game.play_sound(Sound::Coin);
                }
            }
        }
    });
    signals.on::<SnakeSignal>({
        let snake = snake_reference.clone();
        move |signal, game| match signal {
            SnakeSignal::Killed {
                snake_id: killed_id,
            } => {
                if *killed_id == snake_id && snake.upgrade().is_some() {
                    game.play_sound(Sound::Impact);
                    game.end();
                }
            }
            SnakeSignal::FoodEaten { .. } => {}
        }
    });
    signals.try_on::<GameSignal>(move |signal, game| {
        match signal {
            GameSignal::Reset => {
                let bounds = game.playable_bounds();
                if let (Some(snake), Some(apple)) =
                    (snake_reference.upgrade(), apple_reference.upgrade())
                {
                    let mut snake = snake.borrow_mut();
                    snake.respawn(Transform {
                        position: bounds.middle(),
                        rotation: TransformRotation::default(),
                    });
                    let position = Board::vacant_position(
                        snake.occupied_cells(),
                        &mut *random.borrow_mut(),
                    )
                    .ok_or_else(|| anyhow!("board has no vacant cell for the reset apple"))?;
                    apple.borrow_mut().respawn(position);
                }
                game.start()?;
            }
            &GameSignal::PauseChanged { is_paused } => {
                game.play_sound(if is_paused {
                    Sound::Pause
                } else {
                    Sound::Unpause
                });
            }
            GameSignal::Started => {
                game.play_sound(Sound::Start);
            }
            GameSignal::Over | GameSignal::Won => {}
        }
        Ok(())
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
        time::Duration,
    };

    use snake_core::{
        Bounds, GameObject, Move, MoveDirection, Rotation, Vector2Int, assets,
        models::{
            apple::{Apple, AppleSignal},
            snake::{Snake, SnakeContact, SnakeSignal},
        },
        signal::{SignalBus, SignalBusBuilder},
    };

    use super::register_snake_objects;
    use crate::{
        game::{Game, GameState},
        infra::{input::InputKey, random::RandRandomSource},
        test_utils::new_game,
    };

    struct TestScene {
        game: Game,
        signals: SignalBus<Game, anyhow::Error>,
        snake: Rc<RefCell<Snake>>,
        apple: Rc<RefCell<Apple>>,
        apples_eaten: Rc<Cell<usize>>,
    }

    fn new_scene() -> TestScene {
        let mut signals = SignalBusBuilder::new();
        let mut game = new_game(&mut signals);
        let snake = Rc::new(RefCell::new(Snake::spawn(
            signals.register::<SnakeSignal>().unwrap(),
        )));
        let apple = Rc::new(RefCell::new(Apple::spawn(
            Vector2Int::new(18, 18),
            signals.register::<AppleSignal>().unwrap(),
        )));
        snake
            .borrow_mut()
            .set_position(game.playable_bounds().middle());
        register_snake_objects(
            &mut game,
            &mut signals,
            snake.clone(),
            apple.clone(),
            Rc::new(RefCell::new(RandRandomSource::new(0))),
        )
        .unwrap();
        let apples_eaten = Rc::new(Cell::new(0));
        signals.on::<AppleSignal>({
            let apples_eaten = apples_eaten.clone();
            move |_, _| apples_eaten.set(apples_eaten.get() + 1)
        });
        TestScene {
            game,
            signals: signals.build(),
            snake,
            apple,
            apples_eaten,
        }
    }

    #[test]
    fn callbacks_do_not_keep_registered_objects_alive() {
        let TestScene {
            game,
            signals,
            snake,
            apple,
            ..
        } = new_scene();
        let snake_reference = Rc::downgrade(&snake);
        let apple_reference = Rc::downgrade(&apple);
        drop(snake);
        drop(apple);
        assert_eq!(snake_reference.strong_count(), 1);
        assert_eq!(apple_reference.strong_count(), 1);

        drop(game);

        assert!(snake_reference.upgrade().is_none());
        assert!(apple_reference.upgrade().is_none());
        drop(signals);
    }

    #[test]
    fn collision_grows_the_snake_and_respawns_the_apple_on_the_next_update() {
        let TestScene {
            mut game,
            mut signals,
            snake,
            apple,
            ..
        } = new_scene();
        let position = apple.borrow().position();
        snake.borrow_mut().set_position(position);
        let size = snake.borrow().hp();
        game.start().unwrap();

        game.update(Duration::ZERO, &mut signals).unwrap();

        assert_eq!(snake.borrow().hp(), size + 1);
        game.update(Duration::ZERO, &mut signals).unwrap();
        assert!(!apple.borrow().eaten());
        let position = apple.borrow().position();
        let Bounds { start, end } = game.playable_bounds();
        assert!((start.x..end.x).contains(&position.x));
        assert!((start.y..end.y).contains(&position.y));
        assert_eq!(
            game.render_frame()
                .cells()
                .iter()
                .filter(|cell| {
                    matches!(cell.texture(), assets::SNAKE_HEAD | assets::SNAKE_PART)
                })
                .count(),
            usize::from(size + 1)
        );
    }

    #[test]
    fn dead_snake_does_not_consume_an_overlapping_apple() {
        let TestScene {
            mut game,
            mut signals,
            snake,
            apple,
            apples_eaten,
        } = new_scene();
        let position = Vector2Int::new(10, 10);
        apple.borrow_mut().respawn(position);
        snake.borrow_mut().set_position(position);
        snake.borrow_mut().on_collision(SnakeContact::Solid);
        let size = snake.borrow().hp();
        game.start().unwrap();

        game.update(Duration::ZERO, &mut signals).unwrap();

        assert_eq!(game.state(), GameState::GameOver);
        assert_eq!(apples_eaten.get(), 0);
        assert_eq!(snake.borrow().hp(), size);
        assert!(!apple.borrow().eaten());
        assert_eq!(apple.borrow().position(), position);
    }

    #[test]
    fn reset_respawns_objects_during_play_and_after_game_over() {
        for game_over in [false, true] {
            let TestScene {
                mut game,
                mut signals,
                snake,
                apple,
                ..
            } = new_scene();
            let size = snake.borrow().hp();
            game.start().unwrap();
            snake.borrow_mut().add_part();
            snake.borrow_mut().set_position(Vector2Int::new(2, 2));
            if game_over {
                snake.borrow_mut().on_collision(SnakeContact::Solid);
                game.update(Duration::ZERO, &mut signals).unwrap();
            }

            game.queue_input(Some(InputKey::Reset));
            game.update(Duration::ZERO, &mut signals).unwrap();

            assert_eq!(game.state(), GameState::InGame);
            assert!(snake.borrow().is_alive());
            assert_eq!(snake.borrow().hp(), size);
            assert_eq!(snake.borrow().position(), game.playable_bounds().middle());
            assert!(!apple.borrow().eaten());
        }
    }

    #[test]
    fn render_frame_resolves_rotated_snake_parts_relative_to_the_snake() {
        let TestScene { game, snake, .. } = new_scene();
        snake.borrow_mut().set_position(Vector2Int::new(10, 10));
        snake.borrow_mut().rotate_to(MoveDirection::Down);

        let frame = game.render_frame();
        let snake_cells = frame
            .cells()
            .iter()
            .filter(|cell| matches!(cell.texture(), assets::SNAKE_HEAD | assets::SNAKE_PART))
            .map(|cell| (cell.position(), cell.texture(), cell.rotation()))
            .collect::<Vec<_>>();

        assert_eq!(
            snake_cells,
            [
                (Vector2Int::new(10, 10), assets::SNAKE_HEAD, Rotation::DOWN),
                (Vector2Int::new(9, 10), assets::SNAKE_PART, Rotation::RIGHT),
                (Vector2Int::new(8, 10), assets::SNAKE_PART, Rotation::RIGHT),
                (Vector2Int::new(7, 10), assets::SNAKE_PART, Rotation::RIGHT),
            ]
        );
    }
}
