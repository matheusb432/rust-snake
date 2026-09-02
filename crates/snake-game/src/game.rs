use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::Entry},
    error::Error,
    fmt,
    rc::Rc,
};

use snake_core::{Bounds, GameObject, GameObjectId, Vector2Int};

use crate::{
    board::{BOARD_SIZE_X, BOARD_SIZE_Y, Board},
    render::{RenderFrame, RenderViewport, append_render_cells},
};

pub(crate) struct Game {
    objects: HashMap<GameObjectId, Rc<RefCell<dyn GameObject>>>,
    object_render_order: Vec<GameObjectId>,
    board: Board,
    playable_bounds: Bounds,
}

impl Game {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            object_render_order: Vec::new(),
            board: Board::new(),
            playable_bounds: Bounds {
                start: Vector2Int::new(1, 1),
                end: Vector2Int::new((BOARD_SIZE_X - 2) as i32, (BOARD_SIZE_Y - 2) as i32),
            },
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

    pub fn render_frame(&self) -> RenderFrame {
        let mut cells = Vec::new();
        append_render_cells(&mut cells, Vector2Int::default(), &self.board);

        for id in &self.object_render_order {
            let object = self
                .objects
                .get(id)
                .expect("render order must contain only inserted game objects")
                .borrow();
            append_render_cells(&mut cells, object.position(), &*object);
        }

        RenderFrame::new(RenderViewport::new(BOARD_SIZE_X, BOARD_SIZE_Y), cells)
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
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
        GameObject, GameObjectId, Render, RenderItem, Rotation, Texture, TextureColor, Vector2Int,
        ZIndex, models::snake::Snake,
    };

    use super::{Game, InsertObjectError};

    struct Foo {
        id: GameObjectId,
        position: Vector2Int,
    }

    impl Foo {
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
                Texture::new('f', TextureColor::White),
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

    #[test]
    fn inserts_an_object_and_returns_its_handle() {
        let mut game = Game::new();
        let foo = Foo::at(Vector2Int::new(4, 7));
        let updated_position = Vector2Int::new(8, 9);

        let foo_handle = game.insert_object(foo).unwrap();
        foo_handle.borrow_mut().position = updated_position;

        assert!(
            game.render_frame().cells().iter().any(|cell| {
                cell.position_world() == updated_position && cell.texture().character() == 'f'
            }),
            "the stored object should reflect mutations through its returned handle"
        );
    }

    #[test]
    fn inserts_objects_at_the_same_position() {
        let mut game = Game::new();
        let position = Vector2Int::new(24, 7);

        assert!(game.insert_object(Foo::at(position)).is_ok());
        assert!(game.insert_object(Foo::at(position)).is_ok());
        assert_eq!(
            game.render_frame()
                .cells()
                .iter()
                .filter(|cell| {
                    cell.position_world() == position && cell.texture().character() == 'f'
                })
                .count(),
            2
        );
    }

    #[test]
    fn rejects_an_object_with_a_duplicate_id() {
        let mut game = Game::new();
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
        let mut game = Game::new();
        let position = Vector2Int::new(4, 7);
        game.insert_object(Foo::at(position)).unwrap();

        let frame = game.render_frame();

        assert_eq!(
            frame
                .cells()
                .iter()
                .filter(|cell| cell.position_world() == position)
                .map(|cell| cell.texture().character())
                .collect::<Vec<_>>(),
            [' ', 'f']
        );
    }

    #[test]
    fn render_frame_resolves_snake_parts_relative_to_the_snake() {
        let mut game = Game::new();
        let mut snake = Snake::spawn();
        snake.set_position(Vector2Int::new(10, 10));
        game.insert_object(snake).unwrap();

        let frame = game.render_frame();
        let snake_cells = frame
            .cells()
            .iter()
            .filter(|cell| matches!(cell.texture().character(), '{' | '~'))
            .map(|cell| (cell.position_world(), cell.texture().character()))
            .collect::<Vec<_>>();

        assert_eq!(
            snake_cells,
            [
                (Vector2Int::new(10, 10), '{'),
                (Vector2Int::new(9, 10), '~'),
                (Vector2Int::new(8, 10), '~'),
                (Vector2Int::new(7, 10), '~'),
            ]
        );
    }

    #[test]
    fn playable_bounds_exclude_the_board_walls() {
        let bounds = Game::new().playable_bounds();

        assert_eq!(bounds.start, Vector2Int::new(1, 1));
        assert_eq!(bounds.end, Vector2Int::new(22, 22));
    }
}
