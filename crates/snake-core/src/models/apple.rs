use crate::{
    GameObject, GameObjectId, Render, RenderItem, Rotation, Transform, Vector2Int, ZIndex, assets,
    models::snake::SnakeSignal, signal::SignalEmitter,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppleSignal {
    Eaten { apple_id: GameObjectId },
}

pub struct Apple {
    id: GameObjectId,
    transform: Transform,
    eaten: bool,
    emitter: SignalEmitter<AppleSignal>,
}
impl Apple {
    #[must_use]
    pub fn spawn(position: Vector2Int, emitter: SignalEmitter<AppleSignal>) -> Self {
        Self {
            id: GameObjectId::new(),
            transform: Transform {
                position,
                ..Transform::default()
            },
            eaten: false,
            emitter,
        }
    }

    pub fn on_snake_signal(&mut self, signal: &SnakeSignal) {
        if let SnakeSignal::FoodEaten { food_id } = signal
            && *food_id == self.id
            && !self.eaten
        {
            self.eaten = true;
            self.emitter.emit(AppleSignal::Eaten { apple_id: self.id });
        }
    }

    #[must_use]
    pub fn collision_cell(&self) -> Option<(GameObjectId, Vector2Int)> {
        (!self.eaten).then_some((self.id, self.transform.position))
    }

    #[must_use]
    pub fn eaten(&self) -> bool {
        self.eaten
    }

    pub fn respawn(&mut self, position: Vector2Int) {
        self.transform.position = position;
        self.eaten = false;
    }
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
        if self.eaten {
            return;
        }
        visit(RenderItem::glyph(
            Vector2Int::default(),
            assets::APPLE,
            self.transform.rotation.look(),
            ZIndex::DEFAULT,
        ));
    }
}
