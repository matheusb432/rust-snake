use std::array;

use snake_core::{Texture, Vector2Int};

use crate::assets;

pub(crate) const BOARD_SIZE_X: usize = 24;
pub(crate) const BOARD_SIZE_Y: usize = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoardPosition {
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

    pub fn texture_at(&self, board_x: usize, board_y: usize) -> Texture {
        self.inner[board_x][board_y]
    }

    fn classify_position(position: Vector2Int) -> BoardPosition {
        let x_max = (BOARD_SIZE_X - 1) as i32;
        let y_max = (BOARD_SIZE_Y - 1) as i32;
        let x_is_edge = position.x == 0 || position.x == x_max;
        let y_is_edge = position.y == 0 || position.y == y_max;

        BoardPosition::new(x_is_edge, y_is_edge)
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

fn create_default_board() -> [[Texture; BOARD_SIZE_Y]; BOARD_SIZE_X] {
    array::from_fn(|x| {
        array::from_fn(|y| {
            let position = Vector2Int::new(x as i32, y as i32);

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
