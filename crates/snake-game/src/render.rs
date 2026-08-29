use std::{
    array,
    io::{self, Write},
};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use snake_core::Vector2;

use crate::assets;

pub(crate) const BOARD_SIZE_X: usize = 24;
pub(crate) const BOARD_SIZE_Y: usize = 24;

// TODO: refactor to struct if no other variants are necessary
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Texture {
    Char { character: char, color: Color },
}

/// necessary because each linebreak is roughly twice the size of a single character. this therefore
/// makes the board more symmetric.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TerminalProjection {
    interior_cell_width_columns: usize,
}

impl TerminalProjection {
    pub(crate) const DOUBLE_WIDTH_INTERIOR: Self = Self {
        interior_cell_width_columns: 2,
    };

    fn board_cell_width_columns(self, board_x: usize) -> usize {
        if board_x == 0 || board_x == BOARD_SIZE_X - 1 {
            1
        } else {
            self.interior_cell_width_columns
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoardPosition {
    Corner,
    WallX,
    WallY,
    Inside,
}

impl BoardPosition {
    fn new(x_is_edge: bool, y_is_edge: bool) -> Self {
        match (x_is_edge, y_is_edge) {
            (true, true) => Self::Corner,
            (true, false) => Self::WallX,
            (false, true) => Self::WallY,
            (false, false) => Self::Inside,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Board {
    inner: [[Texture; BOARD_SIZE_Y]; BOARD_SIZE_X],
}
impl Board {
    pub fn new() -> Self {
        Self {
            inner: create_default_board(),
        }
    }

    pub fn classify_position(position: Vector2) -> BoardPosition {
        let x_max = (BOARD_SIZE_X - 1) as f32;
        let y_max = (BOARD_SIZE_Y - 1) as f32;
        let x_is_edge = position.x == 0.0 || position.x == x_max;
        let y_is_edge = position.y == 0.0 || position.y == y_max;

        BoardPosition::new(x_is_edge, y_is_edge)
    }
}

pub(crate) fn render_board(
    output: &mut impl Write,
    board: &Board,
    projection: TerminalProjection,
) -> io::Result<()> {
    let mut color_current = None;

    for board_y in 0..BOARD_SIZE_Y {
        queue!(output, MoveTo(0, board_y as u16))?;

        for board_x in 0..BOARD_SIZE_X {
            let Texture::Char { character, color } = board.inner[board_x][board_y];

            if color_current != Some(color) {
                queue!(output, SetForegroundColor(color))?;
                color_current = Some(color);
            }

            for _ in 0..projection.board_cell_width_columns(board_x) {
                queue!(output, Print(character))?;
            }
        }
    }

    queue!(output, ResetColor)?;
    output.flush()
}

fn create_default_board() -> [[Texture; BOARD_SIZE_Y]; BOARD_SIZE_X] {
    array::from_fn(|x| {
        array::from_fn(|y| {
            let position = Vector2 {
                x: x as f32,
                y: y as f32,
            };

            match Board::classify_position(position) {
                BoardPosition::Corner => assets::WALL_DIAGONAL,
                BoardPosition::WallX => assets::WALL_X,
                BoardPosition::WallY => assets::WALL_Y,
                BoardPosition::Inside => assets::BLANK,
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::{BOARD_SIZE_X, BOARD_SIZE_Y, create_default_board};
    use crate::assets;

    #[test]
    fn create_default_board_surrounds_blank_inside_with_walls() {
        let board = create_default_board();
        let x_max = BOARD_SIZE_X - 1;
        let y_max = BOARD_SIZE_Y - 1;

        assert_eq!(
            [
                board[0][0],
                board[0][y_max],
                board[x_max][0],
                board[x_max][y_max],
            ],
            [assets::WALL_DIAGONAL; 4]
        );

        assert!(
            board[0][1..y_max]
                .iter()
                .all(|texture| *texture == assets::WALL_X)
        );
        assert!(
            board[x_max][1..y_max]
                .iter()
                .all(|texture| *texture == assets::WALL_X)
        );

        assert!(board[1..x_max].iter().all(|col| col[0] == assets::WALL_Y));
        assert!(
            board[1..x_max]
                .iter()
                .all(|col| col[y_max] == assets::WALL_Y)
        );

        assert!(board[1..x_max].iter().all(|col| {
            col[1..y_max]
                .iter()
                .all(|texture| *texture == assets::BLANK)
        }));
    }
}
