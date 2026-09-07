use std::ops::AddAssign;

use snake_core::{
    GameObject, GameObjectId, Render, RenderItem, Rotation, Texture, TextureColor, Vector2Int,
    ZIndex,
};

#[derive(Debug, Clone)]
pub(super) struct Scorebar {
    id: GameObjectId,
    pub score: Score,
    position: Vector2Int,
}

impl Scorebar {
    pub(super) fn new(position: Vector2Int) -> Self {
        Self {
            id: GameObjectId::new(),
            score: Score::EMPTY,
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
    fn visit_render_items(&self, visit: &mut dyn FnMut(RenderItem)) {
        let content = format!("Score: {}", self.score.0);
        let column_start = -(content.len() as i32);
        for (column, character) in (column_start..0).zip(content.chars()) {
            visit(RenderItem::screen_glyph(
                Vector2Int::new(column, 0),
                Texture::new(character, TextureColor::White),
                ZIndex::new(1),
            ));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Score(u32);
impl Score {
    pub const EMPTY: Self = Score(0);
    pub const UNIT: Self = Score(100);

    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn add_unit(&mut self) {
        self.0 += Self::UNIT.0;
    }
}
impl AddAssign for Score {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_add(rhs.0);
    }
}
