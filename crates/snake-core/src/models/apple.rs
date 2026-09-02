use rand::RngExt;

use crate::{
    Bounds, GameObject, GameObjectId, Render, RenderItem, Rotation, Transform, Vector2Int, ZIndex,
    assets,
};

pub struct Apple {
    id: GameObjectId,
    transform: Transform,
}
impl Apple {
    pub fn spawn(position_bounds: Bounds) -> Self {
        let mut apple = Self {
            id: GameObjectId::new(),
            transform: Transform::default(),
        };
        apple.respawn(position_bounds);
        apple
    }

    pub fn respawn(&mut self, position_bounds: Bounds) {
        // TODO: make it a parameter to test deterministicslly
        let mut rng = rand::rng();
        let Bounds { start, end } = position_bounds;

        self.transform.position = Vector2Int::new(
            rng.random_range(start.x..=end.x),
            rng.random_range(start.y..=end.y),
        );
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
    fn visit_render_items(&self, visit: &mut dyn FnMut(RenderItem)) {
        visit(RenderItem::glyph(
            Vector2Int::default(),
            assets::APPLE,
            ZIndex::DEFAULT,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::Apple;
    use crate::{Bounds, GameObject, Vector2Int};

    #[test]
    fn spawn_accepts_each_inclusive_bound() {
        for position in [Vector2Int::new(1, 1), Vector2Int::new(22, 22)] {
            let bounds = Bounds {
                start: position,
                end: position,
            };

            let apple = Apple::spawn(bounds);

            assert_eq!(apple.position(), position);
        }
    }
}
