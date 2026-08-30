use std::hash::RandomState;

use rand::RngExt;

use crate::{GameObject, GameObjectId, Render, Rotation, Texture, Transform, Vector2Int, assets};

pub struct Apple {
    id: GameObjectId,
    transform: Transform,
}
impl Apple {
    pub fn new() -> Self {
        Self {
            id: GameObjectId::new(),
            transform: Transform::default(),
        }
    }

    // TODO think of less goofy name?
    pub fn be_eaten(&mut self, position_upper_bounds: Vector2Int) {
        // TODO: make it a parameter to test deterministicslly
        let mut rng = rand::rng();
        let Vector2Int {
            x: x_upper,
            y: y_upper,
        } = position_upper_bounds;

        self.transform.position = Vector2Int {
            x: rng.random_range(0..=x_upper),
            y: rng.random_range(0..=y_upper),
        }
    }
}
impl GameObject for Apple {
    fn rotation(&self) -> Rotation {
        self.transform.rotation
    }

    fn id(&self) -> GameObjectId {
        self.id
    }

    fn position(&self) -> crate::Vector2Int {
        self.transform.position
    }
}

impl Render for Apple {
    fn texture(&self) -> Texture {
        assets::APPLE
    }
}
