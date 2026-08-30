use std::{collections::HashMap, error::Error, fmt, rc::Rc};

use snake_core::{GameObject, GameObjectId};

use crate::board::Board;

pub(crate) type GameObjectIterator<'a> = Box<dyn Iterator<Item = &'a dyn GameObject> + 'a>;

pub(crate) struct Game {
    objects: HashMap<GameObjectId, Rc<dyn GameObject>>,
    board: Board,
}

impl Game {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            board: Board::new(),
        }
    }

    pub fn place_object(&mut self, object: Rc<dyn GameObject>) -> Result<(), PlaceObjectError> {
        let id = object.id();
        if self.objects.contains_key(&id) {
            return Err(PlaceObjectError::ObjectAlreadyPlaced { id });
        }

        self.objects.insert(id, object);
        Ok(())
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn objects(&self) -> GameObjectIterator<'_> {
        Box::new(self.objects.values().map(|object| object.as_ref()))
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PlaceObjectError {
    ObjectAlreadyPlaced { id: GameObjectId },
}

impl fmt::Display for PlaceObjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ObjectAlreadyPlaced { id } => {
                write!(formatter, "object {id} is already placed")
            }
        }
    }
}

impl Error for PlaceObjectError {}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use snake_core::{
        GameObject, GameObjectId, Render, Rotation, Texture, TextureColor, Vector2Int,
    };

    use super::Game;

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
        fn texture(&self) -> Texture {
            Texture::new('f', TextureColor::White)
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
    fn places_an_object() {
        let mut game = Game::new();
        let foo = Foo::at(Vector2Int::new(4, 7));
        let id = foo.id();

        assert_eq!(game.place_object(Rc::new(foo)), Ok(()));
        assert!(game.objects().any(|object| object.id() == id));
    }

    #[test]
    fn places_objects_at_the_same_position() {
        let mut game = Game::new();
        let position = Vector2Int::new(24, 7);

        assert_eq!(game.place_object(Rc::new(Foo::at(position))), Ok(()));
        assert_eq!(game.place_object(Rc::new(Foo::at(position))), Ok(()));
        assert_eq!(
            game.objects()
                .filter(|object| object.position() == position)
                .count(),
            2
        );
    }
}
