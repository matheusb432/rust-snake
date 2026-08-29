use uuid::Uuid;

use crate::{Rotation, Vector2};

pub trait GameObject {
    fn position(&self) -> Vector2;
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
