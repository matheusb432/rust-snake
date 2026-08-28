use std::{collections::binary_heap::Iter, fmt::Display};

use crossterm::style::Color;

use crate::assets;

pub(crate) const BOARD_SIZE_X: usize = 24;
pub(crate) const BOARD_SIZE_Y: usize = 24;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Tile {
    Char(char),
    Solid,
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let char_str: &str = match self {
            // TODO refactor
            Self::Char(c) => &c.to_string(),
            // TODO solid doesnt make sense for tile w/o Texture, move to Texture
            Self::Solid => "s",
        };
        write!(f, "{}", char_str)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Texture {
    // TODO change acmod
    pub(crate) tile: Tile,
    pub(crate) color: Color,
}

impl Display for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tile)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Board {
    inner: [[Texture; BOARD_SIZE_X]; BOARD_SIZE_Y],
}
impl Board {
    pub fn new() -> Self {
        let mut board = [[assets::BLANK; BOARD_SIZE_X]; BOARD_SIZE_Y];

        // TODO cleanup and write actual boare init (and more rusty)
        for (i, row) in board.iter_mut().enumerate().filter(|(i, _)| i % 4 == 0) {
            for k in 0..row.len() {
                row[k] = assets::WALL_Y;
            }
        }

        Self { inner: board }
    }

    pub fn size(&self) -> usize {
        // TODO: study if as_flattened is more expensive than a ptr alloc
        self.inner.as_flattened().len()
    }

    // TODO create strong types, and impl Iterator for board, maybe?
    pub fn get_texture(&self, x: usize, y: usize) -> Texture {
        self.inner[x][y]
    }
}
