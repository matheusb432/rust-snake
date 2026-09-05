use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use snake_core::{
    GameObject, Transform, TransformRotation,
    models::{
        apple::{Apple, AppleSignal},
        snake::{Snake, SnakeSignal},
    },
    signal::SignalBusBuilder,
};

use crate::{
    game::{Game, GameSignal},
    infra::{audio::Sound, input::InputKey},
};

/// registers and manages snake scene state.
pub(crate) fn register_snake_scene(
    game: &mut Game,
    signals: &mut SignalBusBuilder<Game, anyhow::Error>,
) -> Result<()> {
    let snake = Rc::new(RefCell::new(Snake::spawn(
        signals.register::<SnakeSignal>()?,
    )));
    let apple = Rc::new(RefCell::new(Apple::spawn(
        game.playable_bounds(),
        signals.register::<AppleSignal>()?,
    )));
    snake
        .borrow_mut()
        .set_position(game.playable_bounds().middle());
    register_snake_objects(game, signals, snake, apple)
}

/// registers game objects and signal listeners
fn register_snake_objects(
    game: &mut Game,
    signals: &mut SignalBusBuilder<Game, anyhow::Error>,
    snake: Rc<RefCell<Snake>>,
    apple: Rc<RefCell<Apple>>,
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
                snake.borrow_mut().move_in_direction(direction);
            }
        }
    });
    game.on_update({
        let snake = snake_reference.clone();
        let apple = apple_reference.clone();
        move |delta_time, bounds| {
            let snake = snake.upgrade();
            if let Some(snake) = &snake {
                snake.borrow_mut().tick(delta_time);
            }
            let Some(apple) = apple.upgrade() else {
                return;
            };
            let can_eat = snake.is_some_and(|snake| {
                let snake = snake.borrow();
                snake.is_alive() && snake.position() == apple.borrow().position()
            });
            if can_eat {
                apple.borrow_mut().be_eaten();
            }
            apple.borrow_mut().tick(bounds);
        }
    });
    signals.on::<AppleSignal>({
        let snake = snake_reference.clone();
        move |signal, game| match signal {
            AppleSignal::Eaten { apple_id: eaten_id } => {
                if *eaten_id == apple_id
                    && let Some(snake) = snake.upgrade()
                {
                    snake.borrow_mut().add_part();
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
                    game.end();
                }
            }
        }
    });
    signals.try_on::<GameSignal>(move |signal, game| {
        match signal {
            GameSignal::Reset => {
                let bounds = game.playable_bounds();
                if let Some(snake) = snake_reference.upgrade() {
                    snake.borrow_mut().respawn(Transform {
                        position: bounds.middle(),
                        rotation: TransformRotation::default(),
                    });
                }
                if let Some(apple) = apple_reference.upgrade() {
                    apple.borrow_mut().respawn(bounds);
                }
                game.start()?;
            }
            GameSignal::Over => {
                // TODO: render game over text as its own game object
                // println!(
                //     "\r\nGame over! Press '{}' to restart.",
                //     InputKey::RESET_CHARACTER.to_ascii_uppercase()
                // );
            }
            GameSignal::PauseChanged { is_paused: _ } => {
                // TODO: render (or disable) pause text
            }
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
            snake::{Snake, SnakeSignal},
        },
        signal::{SignalBus, SignalBusBuilder},
    };

    use super::{register_snake_objects, register_snake_scene};
    use crate::{
        game::{Game, GameState},
        infra::input::InputKey,
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
            game.playable_bounds(),
            signals.register::<AppleSignal>().unwrap(),
        )));
        snake
            .borrow_mut()
            .set_position(game.playable_bounds().middle());
        register_snake_objects(&mut game, &mut signals, snake.clone(), apple.clone()).unwrap();
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
    fn registered_scene_handles_movement_pause_and_resume() {
        let mut signals = SignalBusBuilder::new();
        let mut game = new_game(&mut signals);
        register_snake_scene(&mut game, &mut signals).unwrap();
        let mut signals = signals.build();
        let head_position = |game: &Game| {
            game.render_frame()
                .cells()
                .iter()
                .find(|cell| cell.texture() == assets::SNAKE_HEAD)
                .unwrap()
                .position_world()
        };
        assert_eq!(head_position(&game), Vector2Int::new(11, 11));

        game.queue_input(Some(InputKey::Move(MoveDirection::Right)));
        game.update(Duration::ZERO, &mut signals).unwrap();
        assert_eq!(head_position(&game), Vector2Int::new(12, 11));

        game.queue_input(Some(InputKey::Pause));
        game.update(Duration::from_secs(1), &mut signals).unwrap();
        assert_eq!(game.state(), GameState::Paused);
        assert_eq!(head_position(&game), Vector2Int::new(12, 11));

        game.queue_input(Some(InputKey::Move(MoveDirection::Up)));
        game.update(Duration::ZERO, &mut signals).unwrap();
        assert_eq!(game.state(), GameState::InGame);
        assert_eq!(head_position(&game), Vector2Int::new(12, 10));
    }

    #[test]
    fn grows_once_when_an_apple_is_eaten_before_dispatch() {
        let TestScene {
            mut game,
            mut signals,
            snake,
            apple,
            ..
        } = new_scene();
        let size = snake.borrow().hp();
        apple.borrow_mut().be_eaten();
        apple.borrow_mut().be_eaten();

        assert_eq!(snake.borrow().hp(), size);
        game.update(Duration::ZERO, &mut signals).unwrap();
        assert_eq!(snake.borrow().hp(), size + 1);
        game.update(Duration::ZERO, &mut signals).unwrap();
        assert_eq!(snake.borrow().hp(), size + 1);
    }

    #[test]
    fn collision_grows_the_snake_and_respawns_the_apple_before_rendering() {
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
        apple.borrow_mut().respawn(Bounds {
            start: position,
            end: Vector2Int::new(11, 11),
        });
        snake.borrow_mut().set_position(position);
        snake.borrow_mut().kill();
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
                snake.borrow_mut().kill();
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
            .map(|cell| (cell.position_world(), cell.texture(), cell.rotation()))
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
