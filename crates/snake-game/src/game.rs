use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque, hash_map::Entry},
    error::Error,
    fmt,
    rc::Rc,
    time::Duration,
};

use anyhow::bail;
use snake_core::{
    Bounds, GameObject, GameObjectId, Vector2Int,
    signal::{Signal, SignalBus, SignalEmitter},
};

use crate::{
    board::{BOARD_SIZE_X, BOARD_SIZE_Y, Board},
    infra::input::{InputKey, InputQueue},
    render::{RenderFrame, RenderViewport, append_render_cells},
};

const FRAME_INTERVAL: Duration = Duration::from_millis(1000 / 60);

pub(crate) struct Game {
    objects: HashMap<GameObjectId, Rc<RefCell<dyn GameObject>>>,
    object_render_order: Vec<GameObjectId>,
    board: Board,
    playable_bounds: Bounds,
    state: GameState,
    input_queue: InputQueue,
    frame_time_elapsed: Duration,
    emitter: SignalEmitter,
}

impl Game {
    pub fn new(emitter: SignalEmitter) -> Self {
        Self {
            objects: HashMap::new(),
            object_render_order: Vec::new(),
            board: Board::new(),
            playable_bounds: Bounds {
                start: Vector2Int::new(1, 1),
                end: Vector2Int::new((BOARD_SIZE_X - 2) as i32, (BOARD_SIZE_Y - 2) as i32),
            },
            state: GameState::NotStarted,
            input_queue: InputQueue::new(),
            frame_time_elapsed: Duration::ZERO,
            emitter,
        }
    }

    pub fn insert_object<T>(&mut self, object: T) -> Result<Rc<RefCell<T>>, InsertObjectError>
    where
        T: GameObject + 'static,
    {
        let id = object.id();
        let Entry::Vacant(entry) = self.objects.entry(id) else {
            return Err(InsertObjectError::ObjectAlreadyPlaced { id });
        };

        let object_handle = Rc::new(RefCell::new(object));
        let game_object_handle: Rc<RefCell<dyn GameObject>> = object_handle.clone();
        entry.insert(game_object_handle);
        self.object_render_order.push(id);
        Ok(object_handle)
    }

    pub const fn playable_bounds(&self) -> Bounds {
        self.playable_bounds
    }

    pub fn queue_input(&mut self, input_key: Option<InputKey>) {
        let Some(input_key) = input_key else {
            return;
        };

        self.input_queue.push_back(input_key);
    }

    pub fn flush_input(&mut self) -> VecDeque<InputKey> {
        self.input_queue.flush()
    }

    pub fn render_frame(&self) -> RenderFrame {
        let mut cells = Vec::new();
        append_render_cells(&mut cells, Vector2Int::default(), &self.board);

        for id in &self.object_render_order {
            // TODO: refactor to just use hashmap?
            let object = self
                .objects
                .get(id)
                .expect("render order must contain only inserted game objects")
                .borrow();
            append_render_cells(&mut cells, object.position(), &*object);
        }

        RenderFrame::new(RenderViewport::new(BOARD_SIZE_X, BOARD_SIZE_Y), cells)
    }

    pub fn render_frame_if_due(&mut self, delta_time: Duration) -> Option<RenderFrame> {
        self.frame_time_elapsed = self
            .frame_time_elapsed
            .saturating_add(delta_time)
            .min(FRAME_INTERVAL);
        if self.frame_time_elapsed != FRAME_INTERVAL {
            return None;
        }

        self.frame_time_elapsed = Duration::ZERO;
        Some(self.render_frame())
    }

    pub fn state(&self) -> GameState {
        self.state
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        if self.state != GameState::NotStarted {
            bail!("game already started, it cannot be started.")
        }
        // TODO: maybe refactor GameState to typestate pattern of Game
        self.state = GameState::InGame;
        Ok(())
    }

    pub fn to_game_over(&mut self) {
        self.state = GameState::GameOver;
        self.emitter.emit(Signal::GameOver);
    }

    pub fn unpause(&mut self) -> anyhow::Result<()> {
        if self.state != GameState::Paused {
            bail!("game is not paused, it cannot be unpaused.")
        }
        self.state = GameState::InGame;
        Ok(())
    }

    pub fn toggle_pause(&mut self) {
        self.state = match self.state {
            GameState::Paused => GameState::InGame,
            GameState::InGame => GameState::Paused,
            _ => GameState::Paused,
        };
    }
    pub fn reset(&mut self) {
        self.state = GameState::NotStarted;
        // TODO: listen to event in main
        self.emitter.emit(Signal::GameReset);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    NotStarted,
    InGame,
    Paused,
    GameOver,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum InsertObjectError {
    ObjectAlreadyPlaced { id: GameObjectId },
}

impl fmt::Display for InsertObjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ObjectAlreadyPlaced { id } => {
                write!(formatter, "object {id} is already placed")
            }
        }
    }
}

impl Error for InsertObjectError {}

#[cfg(test)]
mod tests {
    use snake_core::{
        GameObject, GameObjectId, Move, MoveDirection, Render, RenderItem, Rotation, Texture,
        TextureColor, Vector2Int, ZIndex, assets, models::snake::Snake, signal::SignalBus,
    };

    use super::{Game, InsertObjectError};
    use crate::infra::input::InputKey;

    struct Foo {
        id: GameObjectId,
        position: Vector2Int,
    }

    impl Foo {
        const TEXTURE: Texture = Texture::new('f', TextureColor::White);

        fn at(position: Vector2Int) -> Self {
            Self {
                id: GameObjectId::new(),
                position,
            }
        }
    }

    impl Render for Foo {
        fn visit_render_items(&self, visit: &mut dyn FnMut(RenderItem)) {
            visit(RenderItem::glyph(
                Vector2Int::default(),
                Self::TEXTURE,
                Rotation::default(),
                ZIndex::DEFAULT,
            ));
        }
    }

    impl GameObject for Foo {
        fn position(&self) -> Vector2Int {
            self.position
        }

        fn id(&self) -> GameObjectId {
            self.id
        }

        fn rotation(&self) -> Rotation {
            Rotation::default()
        }
    }

    fn new_game() -> Game {
        Game::new(SignalBus::new().emitter())
    }
    // TODO: create shared test_utils

    pub(crate) fn spawn_snake() -> Snake {
        let emitter = SignalBus::new().emitter();
        Snake::spawn(emitter)
    }

    #[test]
    fn inserts_an_object_and_returns_its_handle() {
        let mut game = new_game();
        let foo = Foo::at(Vector2Int::new(4, 7));
        let updated_position = Vector2Int::new(8, 9);

        let foo_handle = game.insert_object(foo).unwrap();
        foo_handle.borrow_mut().position = updated_position;

        assert!(
            game.render_frame().cells().iter().any(|cell| {
                cell.position_world() == updated_position && cell.texture() == Foo::TEXTURE
            }),
            "the stored object should reflect mutations through its returned handle"
        );
    }

    #[test]
    fn inserts_objects_at_the_same_position() {
        let mut game = new_game();
        let position = Vector2Int::new(24, 7);

        assert!(game.insert_object(Foo::at(position)).is_ok());
        assert!(game.insert_object(Foo::at(position)).is_ok());
        assert_eq!(
            game.render_frame()
                .cells()
                .iter()
                .filter(|cell| {
                    cell.position_world() == position && cell.texture() == Foo::TEXTURE
                })
                .count(),
            2
        );
    }

    #[test]
    fn rejects_an_object_with_a_duplicate_id() {
        let mut game = new_game();
        let id = GameObjectId::new();
        let position = Vector2Int::new(4, 7);

        assert!(
            game.insert_object(Foo { id, position }).is_ok(),
            "the first object should be inserted"
        );
        assert_eq!(
            game.insert_object(Foo { id, position }).map(|_| ()),
            Err(InsertObjectError::ObjectAlreadyPlaced { id })
        );
    }

    #[test]
    fn render_frame_places_objects_above_the_board_at_world_positions() {
        let mut game = new_game();
        let position = Vector2Int::new(4, 7);
        game.insert_object(Foo::at(position)).unwrap();

        let frame = game.render_frame();

        assert_eq!(
            frame
                .cells()
                .iter()
                .filter(|cell| cell.position_world() == position)
                .map(|cell| cell.texture())
                .collect::<Vec<_>>(),
            [assets::BLANK, Foo::TEXTURE]
        );
    }

    #[test]
    fn render_frame_resolves_rotated_snake_parts_relative_to_the_snake() {
        let mut game = new_game();
        let mut snake = spawn_snake();
        snake.set_position(Vector2Int::new(10, 10));
        snake.rotate_to(MoveDirection::Down);
        game.insert_object(snake).unwrap();

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

    #[test]
    fn playable_bounds_exclude_the_board_walls() {
        let bounds = new_game().playable_bounds();

        assert_eq!(bounds.start, Vector2Int::new(1, 1));
        assert_eq!(bounds.end, Vector2Int::new(22, 22));
    }

    #[test]
    fn queues_input_until_it_is_flushed() {
        let mut game = new_game();
        game.queue_input(Some(InputKey::Move(MoveDirection::Up)));
        game.queue_input(None);
        game.queue_input(Some(InputKey::Quit));

        assert_eq!(
            game.flush_input().into_iter().collect::<Vec<_>>(),
            [InputKey::Move(MoveDirection::Up), InputKey::Quit]
        );
        assert!(game.flush_input().is_empty());
    }
}
