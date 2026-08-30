use std::cell::RefCell;

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

pub trait Render {
    fn texture(&self) -> Texture;
}

impl<T: Render + ?Sized> Render for RefCell<T> {
    fn texture(&self) -> Texture {
        self.borrow().texture()
    }
}
