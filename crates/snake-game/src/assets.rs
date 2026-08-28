use crossterm::style::Color;

use crate::render::{Texture, Tile};

pub(super) const SNAKE_HEAD: Texture = Texture {
    tile: Tile::Char('{'),
    color: Color::DarkGreen,
};
pub(super) const SNAKE_PART: Texture = Texture {
    tile: Tile::Char('~'),
    color: Color::DarkGreen,
};
pub(super) const WALL_X: Texture = Texture {
    tile: Tile::Char('|'),
    color: Color::White,
};
pub(super) const WALL_Y: Texture = Texture {
    tile: Tile::Char('_'),
    color: Color::White,
};
pub(super) const BLANK: Texture = Texture {
    tile: Tile::Char(' '),
    color: Color::White,
};
pub(super) const APPLE: Texture = Texture {
    tile: Tile::Char('&'),
    color: Color::Red,
};
