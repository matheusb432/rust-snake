use crate::{
    GameObject, GameObjectId, Render, RenderItem, Rotation, Texture, TextureColor, Vector2Int,
    ZIndex,
};

/// renderable text tui widget
pub struct Text {
    id: GameObjectId,
    position: Vector2Int,
    glyphs: Vec<Texture>,
    visible: bool,
}

impl Text {
    #[must_use]
    pub fn new(position: Vector2Int, content: &str, color: TextureColor) -> Self {
        Self {
            id: GameObjectId::new(),
            position,
            glyphs: content
                .chars()
                .map(|character| {
                    let character = if character.is_ascii_graphic() || character == ' ' {
                        character
                    } else {
                        '?'
                    };
                    Texture::new(character, color)
                })
                .collect(),
            visible: true,
        }
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

impl GameObject for Text {
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

impl Render for Text {
    fn visit_render_items(&self, visit: &mut dyn FnMut(RenderItem)) {
        if !self.visible {
            return;
        }
        for (column, texture) in (0..i32::MAX).zip(&self.glyphs) {
            visit(RenderItem::screen_glyph(
                Vector2Int::new(column, 0),
                *texture,
                ZIndex::new(1),
            ));
        }
    }
}
