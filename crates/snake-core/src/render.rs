use crate::Vector2Int;

/// Draw order key where lower values render first.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZIndex(i32);

impl ZIndex {
    pub const BACKGROUND: Self = Self(-1);
    pub const DEFAULT: Self = Self(0);

    pub const fn new(value: i32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureColor {
    White,
    DarkGreen,
    Red,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Texture {
    character: char,
    color: TextureColor,
}

impl Texture {
    pub const fn new(character: char, color: TextureColor) -> Self {
        Self { character, color }
    }

    pub const fn character(self) -> char {
        self.character
    }

    pub const fn color(self) -> TextureColor {
        self.color
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderItem {
    position_local: Vector2Int,
    texture: Texture,
    z_index: ZIndex,
    fills_cell: bool,
}

impl RenderItem {
    pub const fn glyph(position_local: Vector2Int, texture: Texture, z_index: ZIndex) -> Self {
        Self {
            position_local,
            texture,
            z_index,
            fills_cell: false,
        }
    }

    pub const fn filled_cell(
        position_local: Vector2Int,
        texture: Texture,
        z_index: ZIndex,
    ) -> Self {
        Self {
            position_local,
            texture,
            z_index,
            fills_cell: true,
        }
    }

    pub const fn position_local(self) -> Vector2Int {
        self.position_local
    }

    pub const fn texture(self) -> Texture {
        self.texture
    }

    pub const fn z_index(self) -> ZIndex {
        self.z_index
    }

    pub const fn fills_cell(self) -> bool {
        self.fills_cell
    }
}

pub trait Render {
    fn visit_render_items(&self, visit: &mut dyn FnMut(RenderItem));
}
