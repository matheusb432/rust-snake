use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque, hash_map::Entry},
    error::Error,
    fmt,
    rc::Rc,
    time::Duration,
};

use anyhow::{Result, anyhow, bail};
use snake_core::{
    Bounds, GameObject, GameObjectId, Vector2Int,
    signal::{DispatchSignalError, SignalBus, SignalEmitter},
};

use crate::{
    board::{BOARD_SIZE_X, BOARD_SIZE_Y, Board},
    infra::input::{InputKey, InputQueue},
    render::{RenderFrame, RenderViewport, append_render_cells},
};

const FRAME_INTERVAL: Duration = Duration::from_millis(1000 / 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GameSignal {
    Over,
    Reset,
    PauseChanged { is_paused: bool },
}

pub(crate) struct Game {
    /// mapped game objects by id
    objects: HashMap<GameObjectId, Rc<RefCell<dyn GameObject>>>,
    /// render order determined by z-index when resolved
    object_render_order: Vec<GameObjectId>,
    input_handlers: Vec<Box<dyn FnMut(InputKey)>>,
    update_handlers: Vec<Box<dyn FnMut(Duration, Bounds)>>,
    board: Board,
    playable_bounds: Bounds,
    state: GameState,
    input_queue: InputQueue,
    frame_time_elapsed: Duration,
    emitter: SignalEmitter<GameSignal>,
}

impl Game {
    pub fn new(emitter: SignalEmitter<GameSignal>) -> Self {
        Self {
            objects: HashMap::new(),
            object_render_order: Vec::new(),
            input_handlers: Vec::new(),
            update_handlers: Vec::new(),
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

    pub const fn playable_bounds(&self) -> Bounds {
        self.playable_bounds
    }

    pub fn state(&self) -> GameState {
        self.state
    }

    /// handlers run in order of registration
    pub fn on_input(&mut self, handler: impl FnMut(InputKey) + 'static) {
        self.input_handlers.push(Box::new(handler));
    }

    // handlers run in order of registration
    pub fn on_update(&mut self, handler: impl FnMut(Duration, Bounds) + 'static) {
        self.update_handlers.push(Box::new(handler));
    }

    pub fn insert_object<T>(&mut self, object: Rc<RefCell<T>>) -> Result<(), InsertObjectError>
    where
        T: GameObject + 'static,
    {
        let id = object.borrow().id();
        let Entry::Vacant(entry) = self.objects.entry(id) else {
            return Err(InsertObjectError::ObjectAlreadyPlaced { id });
        };

        entry.insert(object);
        self.object_render_order.push(id);
        Ok(())
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

    pub fn update(
        &mut self,
        delta_time: Duration,
        signal_bus: &mut SignalBus<Self, anyhow::Error>,
    ) -> Result<GameUpdate> {
        if self.apply_input()? == GameUpdate::Quit {
            return Ok(GameUpdate::Quit);
        }

        if self.state() == GameState::InGame {
            for handler in &mut self.update_handlers {
                handler(delta_time, self.playable_bounds);
            }
        }

        signal_bus
            .dispatch_pending(self)
            .map_err(|error| match error {
                DispatchSignalError::UnexpectedSignalType { expected } => {
                    anyhow!("signal payload is not a {expected}")
                }
                DispatchSignalError::Handler(error) => error,
            })?;

        Ok(GameUpdate::Continue)
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

    pub fn start(&mut self) -> anyhow::Result<()> {
        if self.state != GameState::NotStarted {
            bail!("game already started, it cannot be started.")
        }
        self.state = GameState::InGame;
        Ok(())
    }

    pub fn end(&mut self) {
        self.state = GameState::GameOver;
        self.emitter.emit(GameSignal::Over);
    }

    pub fn unpause(&mut self) -> anyhow::Result<()> {
        if self.state != GameState::Paused {
            bail!("game is not paused, it cannot be unpaused.")
        }
        self.state = GameState::InGame;
        Ok(())
    }

    pub fn toggle_pause(&mut self) {
        let is_paused = self.state != GameState::Paused;
        self.state = if is_paused {
            GameState::Paused
        } else {
            GameState::InGame
        };
        self.emitter.emit(GameSignal::PauseChanged { is_paused });
    }
    pub fn reset(&mut self) {
        self.state = GameState::NotStarted;
        self.emitter.emit(GameSignal::Reset);
    }

    fn apply_input(&mut self) -> Result<GameUpdate> {
        for input_key in self.flush_input() {
            let game_state = self.state;
            match (input_key, game_state) {
                (input_key, GameState::GameOver) => {
                    if let InputKey::Reset = input_key {
                        self.reset();
                    }
                    break;
                }
                (InputKey::Move(direction), game_state) => {
                    match game_state {
                        GameState::NotStarted => self.start()?,
                        GameState::Paused => self.unpause()?,
                        GameState::InGame => (),
                        GameState::GameOver => break,
                    }
                    for handler in &mut self.input_handlers {
                        handler(InputKey::Move(direction));
                    }
                }
                (InputKey::Pause, _) => {
                    self.toggle_pause();
                }
                (InputKey::Quit, _) => return Ok(GameUpdate::Quit),
                (InputKey::Reset, GameState::InGame) => {
                    self.reset();
                }
                (InputKey::Reset, _) => {}
            }
        }

        Ok(GameUpdate::Continue)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GameUpdate {
    Continue,
    Quit,
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
    use std::{cell::RefCell, rc::Rc};

    use snake_core::{
        GameObject, GameObjectId, MoveDirection, Render, RenderItem, Rotation, Texture,
        TextureColor, Vector2Int, ZIndex, assets, signal::SignalBusBuilder,
    };

    use super::{Game, GameSignal, InsertObjectError};
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
        let mut signals = SignalBusBuilder::<Game>::new();
        Game::new(signals.register::<GameSignal>().unwrap())
    }
    // TODO: create shared test_utils

    #[test]
    fn emits_game_signal_variants_through_one_emitter() {
        let mut signals = SignalBusBuilder::<Game>::new();
        let mut game = Game::new(signals.register::<GameSignal>().unwrap());
        let received = Rc::new(RefCell::new(Vec::new()));
        signals.on::<GameSignal>({
            let received = received.clone();
            move |signal, _| received.borrow_mut().push(*signal)
        });
        let mut signals = signals.build();

        game.toggle_pause();
        game.toggle_pause();
        game.end();
        game.reset();
        assert!(received.borrow().is_empty());
        signals.dispatch_pending(&mut game).unwrap();

        assert_eq!(
            *received.borrow(),
            [
                GameSignal::PauseChanged { is_paused: true },
                GameSignal::PauseChanged { is_paused: false },
                GameSignal::Over,
                GameSignal::Reset,
            ]
        );
    }

    #[test]
    fn renders_mutations_through_an_inserted_object_handle() {
        let mut game = new_game();
        let foo = Rc::new(RefCell::new(Foo::at(Vector2Int::new(4, 7))));
        let updated_position = Vector2Int::new(8, 9);

        game.insert_object(foo.clone()).unwrap();
        foo.borrow_mut().position = updated_position;

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

        assert!(
            game.insert_object(Rc::new(RefCell::new(Foo::at(position))))
                .is_ok()
        );
        assert!(
            game.insert_object(Rc::new(RefCell::new(Foo::at(position))))
                .is_ok()
        );
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
            game.insert_object(Rc::new(RefCell::new(Foo { id, position })))
                .is_ok(),
            "the first object should be inserted"
        );
        assert_eq!(
            game.insert_object(Rc::new(RefCell::new(Foo { id, position }))),
            Err(InsertObjectError::ObjectAlreadyPlaced { id })
        );
    }

    #[test]
    fn render_frame_places_objects_above_the_board_at_world_positions() {
        let mut game = new_game();
        let position = Vector2Int::new(4, 7);
        game.insert_object(Rc::new(RefCell::new(Foo::at(position))))
            .unwrap();

        let frame = game.render_frame();

        assert_eq!(
            frame
                .cells()
                .iter()
                .filter(|cell| {
                    cell.position_world() == position
                        && matches!(cell.texture(), assets::BLANK | Foo::TEXTURE)
                })
                .map(|cell| cell.texture())
                .collect::<Vec<_>>(),
            [assets::BLANK, Foo::TEXTURE]
        );
    }

    #[test]
    fn playable_bounds_exclude_the_board_walls() {
        let bounds = new_game().playable_bounds;

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
