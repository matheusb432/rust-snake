use crate::Vector2Int;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderTarget {
    One(Texture),
    Many(Vec<(Vector2Int, Texture)>),
}

pub trait Render {
    fn texture(&self) -> RenderTarget;
}
