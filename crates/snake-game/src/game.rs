use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::Entry},
    error::Error,
    fmt,
    rc::Rc,
};

use snake_core::{GameObject, GameObjectId};

use crate::board::Board;

pub(crate) type GameObjectIterator<'a> = Box<dyn Iterator<Item = &'a RefCell<dyn GameObject>> + 'a>;

pub(crate) struct Game {
    objects: HashMap<GameObjectId, Rc<RefCell<dyn GameObject>>>,
    board: Board,
}

impl Game {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            board: Board::new(),
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
        Ok(object_handle)
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn objects(&self) -> GameObjectIterator<'_> {
        Box::new(self.objects.values().map(Rc::as_ref))
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
        GameObject, GameObjectId, Render, RenderTarget, Rotation, Texture, TextureColor, Vector2Int,
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
        fn texture(&self) -> RenderTarget {
            RenderTarget::One(Texture::new('f', TextureColor::White))
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
        let id = foo.id();
        let updated_position = Vector2Int::new(8, 9);

        let foo_handle = game.insert_object(foo).unwrap();
        foo_handle.borrow_mut().position = updated_position;

        assert_eq!(
            game.objects()
                .find(|object| object.borrow().id() == id)
                .map(|object| object.borrow().position()),
            Some(updated_position)
        );
    }

    #[test]
    fn inserts_objects_at_the_same_position() {
        let mut game = Game::new();
        let position = Vector2Int::new(24, 7);

        assert!(game.insert_object(Foo::at(position)).is_ok());
        assert!(game.insert_object(Foo::at(position)).is_ok());
        assert_eq!(
            game.objects()
                .filter(|object| object.borrow().position() == position)
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
}
