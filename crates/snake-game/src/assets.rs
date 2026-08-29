use crossterm::style::Color;

use crate::render::Texture;

pub(super) const SNAKE_HEAD: Texture = Texture::Char {
    character: '{',
    color: Color::DarkGreen,
};
pub(super) const SNAKE_PART: Texture = Texture::Char {
    character: '~',
    color: Color::DarkGreen,
};
pub(super) const WALL_X: Texture = Texture::Char {
    character: '|',
    color: Color::White,
};
pub(super) const WALL_Y: Texture = Texture::Char {
    character: '_',
    color: Color::White,
};
pub(super) const WALL_DIAGONAL: Texture = Texture::Char {
    character: 'x',
    color: Color::White,
};
pub(super) const BLANK: Texture = Texture::Char {
    character: ' ',
    color: Color::White,
};
pub(super) const APPLE: Texture = Texture::Char {
    character: '&',
    color: Color::Red,
};
