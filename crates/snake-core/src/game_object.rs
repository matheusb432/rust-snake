use std::{cell::RefCell, fmt};

use uuid::Uuid;

use crate::{Render, Rotation, Vector2Int};

pub trait GameObject: Render {
    fn position(&self) -> Vector2Int;
    fn rotation(&self) -> Rotation;
    fn id(&self) -> GameObjectId;
}

impl<T: GameObject + ?Sized> GameObject for RefCell<T> {
    fn position(&self) -> Vector2Int {
        self.borrow().position()
    }

    fn rotation(&self) -> Rotation {
        self.borrow().rotation()
    }

    fn id(&self) -> GameObjectId {
        self.borrow().id()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct GameObjectId(Uuid);
impl GameObjectId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for GameObjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for GameObjectId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
