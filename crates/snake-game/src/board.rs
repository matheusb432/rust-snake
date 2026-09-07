use std::{array, num::NonZeroUsize};

use snake_core::{
    Bounds, Render, RenderItem, Rotation, Texture, Vector2Int, ZIndex, assets,
    collision::vacant_cells, random::RandomSource,
};

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

    fn rotation_at(board_x: usize, board_y: usize) -> Rotation {
        let position = Vector2Int::new(board_x as i32, board_y as i32);
        match Self::classify_position(position) {
            BoardPosition::WallY => Rotation::DOWN,
            _ => Rotation::RIGHT,
        }
    }

    pub fn playable_bounds() -> Bounds {
        Bounds {
            start: Vector2Int::new(1, 1),
            end: Vector2Int::new((BOARD_SIZE_X - 1) as i32, (BOARD_SIZE_Y - 1) as i32),
        }
    }

    pub fn vacant_position(
        occupied: impl IntoIterator<Item = Vector2Int>,
        random: &mut dyn RandomSource,
    ) -> Option<Vector2Int> {
        let vacant = vacant_cells(Self::playable_bounds(), occupied);
        let length = NonZeroUsize::new(vacant.len())?;
        Some(vacant[random.index(length)])
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

impl Render for Board {
    fn visit_render_items(&self, visit: &mut dyn FnMut(RenderItem)) {
        for board_y in 0..BOARD_SIZE_Y {
            for board_x in 0..BOARD_SIZE_X {
                visit(RenderItem::filled_cell(
                    Vector2Int::new(board_x as i32, board_y as i32),
                    self.texture_at(board_x, board_y),
                    Self::rotation_at(board_x, board_y),
                    ZIndex::BACKGROUND,
                ));
            }
        }
    }
}

fn create_default_board() -> [[Texture; BOARD_SIZE_Y]; BOARD_SIZE_X] {
    array::from_fn(|x| {
        array::from_fn(|y| {
            let position = Vector2Int::new(x as i32, y as i32);

            match Board::classify_position(position) {
                BoardPosition::Corner => assets::WALL_DIAGONAL,
                BoardPosition::WallX | BoardPosition::WallY => assets::WALL,
                BoardPosition::Inside => assets::BLANK,
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use snake_core::{Rotation, Vector2Int, assets, collision::ColliderShape};

    use super::{BOARD_SIZE_X, BOARD_SIZE_Y, Board, create_default_board};

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
                .all(|texture| *texture == assets::WALL)
        );
        assert!(
            board[x_max][1..y_max]
                .iter()
                .all(|texture| *texture == assets::WALL)
        );

        assert!(board[1..x_max].iter().all(|col| col[0] == assets::WALL));
        assert!(board[1..x_max].iter().all(|col| col[y_max] == assets::WALL));

        assert_eq!(Board::rotation_at(0, 1), Rotation::RIGHT);
        assert_eq!(Board::rotation_at(1, 0), Rotation::DOWN);

        assert!(board[1..x_max].iter().all(|col| {
            col[1..y_max]
                .iter()
                .all(|texture| *texture == assets::BLANK)
        }));

        let boundary = ColliderShape::OutsideBounds(Board::playable_bounds());
        for (x, column) in board.iter().enumerate() {
            for (y, texture) in column.iter().enumerate() {
                assert_eq!(
                    boundary.contains_cell(Vector2Int::new(x as i32, y as i32)),
                    *texture != assets::BLANK,
                );
            }
        }
    }
}
