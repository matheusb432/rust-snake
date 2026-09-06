use snake_core::{GameObject, GameObjectId, Render, RenderItem, Rotation, Vector2Int};

pub(super) struct Scorebar {
    id: GameObjectId,
    position: Vector2Int,
    // TODO: add fields to track score (prolly as a u32?)
}

impl Scorebar {
    pub(super) fn new(position: Vector2Int) -> Self {
        Self {
            id: GameObjectId::new(),
            position,
        }
    }
}

impl GameObject for Scorebar {
    fn id(&self) -> GameObjectId {
        self.id
    }

    fn position(&self) -> Vector2Int {
        self.position
    }

    fn rotation(&self) -> Rotation {
        Rotation::RIGHT
    }
}

impl Render for Scorebar {
    fn visit_render_items(&self, _visit: &mut dyn FnMut(RenderItem)) {
        todo!()
    }
}
