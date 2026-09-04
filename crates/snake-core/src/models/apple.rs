use rand::RngExt;

use crate::{
    Bounds, GameObject, GameObjectId, Render, RenderItem, Rotation, Transform, Vector2Int, ZIndex,
    assets,
    signal::{Signal, SignalEmitter},
};

pub struct Apple {
    id: GameObjectId,
    transform: Transform,
    eaten: bool,
    emitter: SignalEmitter,
}
impl Apple {
    pub fn spawn(position_bounds: Bounds, emitter: SignalEmitter) -> Self {
        let mut apple = Self {
            id: GameObjectId::new(),
            transform: Transform::default(),
            eaten: false,
            emitter,
        };
        apple.respawn(position_bounds);
        apple
    }

    pub fn be_eaten(&mut self) -> AppleEatenOk {
        match self.eaten {
            true => AppleEatenOk::AlreadyEaten,
            false => {
                self.eaten = true;
                self.emitter.emit(Signal::AppleEaten { apple_id: self.id });
                AppleEatenOk::Eaten
            }
        }
    }

    pub fn eaten(&self) -> bool {
        self.eaten
    }

    // TODO: think of cleaner way to set bounds than to require every fn to have it (maybe a
    // Rc<T> of game coordinate data that GameObjects can own?)
    pub fn tick(&mut self, position_bounds: Bounds) {
        if self.eaten {
            self.respawn(position_bounds);
        }
    }

    pub fn respawn(&mut self, position_bounds: Bounds) {
        // TODO: make it a parameter to test deterministicslly
        let mut rng = rand::rng();
        let Bounds { start, end } = position_bounds;

        // TODO: make it never overlap with snake's position
        self.transform.position = Vector2Int::new(
            rng.random_range(start.x..end.x),
            rng.random_range(start.y..end.y),
        );
        self.eaten = false;
    }
}

pub enum AppleEatenOk {
    Eaten,
    AlreadyEaten,
}
impl GameObject for Apple {
    fn rotation(&self) -> Rotation {
        self.transform.rotation.look()
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
            self.transform.rotation.look(),
            ZIndex::DEFAULT,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::Apple;
    use crate::{Bounds, GameObject, Vector2Int, signal::SignalBus};

    fn spawn_apple(bounds: Bounds) -> Apple {
        let emitter = SignalBus::new().emitter();
        Apple::spawn(bounds, emitter)
    }

    #[test]
    fn respawn_excludes_the_end_bound() {
        let position_expected = Vector2Int::new(1, 1);
        let bounds = Bounds {
            start: position_expected,
            end: Vector2Int::new(2, 2),
        };
        let mut apple = spawn_apple(bounds);

        for _ in 0..64 {
            apple.respawn(bounds);

            assert_eq!(apple.position(), position_expected);
        }
    }
}
