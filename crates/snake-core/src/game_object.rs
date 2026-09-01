use std::fmt;

use uuid::Uuid;

use crate::{Render, Rotation, Vector2Int};

pub trait GameObject: Render {
    fn position(&self) -> Vector2Int;
    fn rotation(&self) -> Rotation;
    fn id(&self) -> GameObjectId;
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
