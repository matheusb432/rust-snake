use std::collections::HashSet;

use crate::{Bounds, Vector2Int};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColliderShape {
    Cell(Vector2Int),
    OutsideBounds(Bounds),
}

impl ColliderShape {
    #[must_use]
    pub fn contains_cell(self, position: Vector2Int) -> bool {
        match self {
            Self::Cell(cell) => cell == position,
            Self::OutsideBounds(Bounds { start, end }) => {
                !(start.x..end.x).contains(&position.x) || !(start.y..end.y).contains(&position.y)
            }
        }
    }
}

pub fn vacant_cells(
    bounds: Bounds,
    occupied: impl IntoIterator<Item = Vector2Int>,
) -> Vec<Vector2Int> {
    let occupied: HashSet<_> = occupied.into_iter().collect();
    (bounds.start.y..bounds.end.y)
        .flat_map(|y| (bounds.start.x..bounds.end.x).map(move |x| Vector2Int::new(x, y)))
        .filter(|position| !occupied.contains(position))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{ColliderShape, vacant_cells};
    use crate::{Bounds, Vector2Int};

    #[test]
    fn shapes_detect_cells_and_exclude_the_end_bound() {
        let bounds = Bounds {
            start: Vector2Int::new(1, 1),
            end: Vector2Int::new(23, 23),
        };
        for (position, outside) in [
            (Vector2Int::new(1, 1), false),
            (Vector2Int::new(22, 22), false),
            (Vector2Int::new(0, 10), true),
            (Vector2Int::new(10, 0), true),
            (Vector2Int::new(23, 10), true),
            (Vector2Int::new(10, 23), true),
            (Vector2Int::new(-1, 10), true),
            (Vector2Int::new(24, 10), true),
        ] {
            assert_eq!(
                ColliderShape::OutsideBounds(bounds).contains_cell(position),
                outside
            );
            assert!(ColliderShape::Cell(position).contains_cell(position));
            assert!(!ColliderShape::Cell(position).contains_cell(position + Vector2Int::new(1, 0)));
        }
    }

    #[test]
    fn vacancy_excludes_occupied_cells_and_handles_a_full_board() {
        let bounds = Bounds {
            start: Vector2Int::new(1, 1),
            end: Vector2Int::new(3, 3),
        };
        let cells = vacant_cells(bounds, []);
        assert_eq!(
            cells,
            [
                Vector2Int::new(1, 1),
                Vector2Int::new(2, 1),
                Vector2Int::new(1, 2),
                Vector2Int::new(2, 2),
            ]
        );
        assert_eq!(
            vacant_cells(
                bounds,
                [cells[0], cells[0], cells[1], cells[2], Vector2Int::ZERO]
            ),
            [cells[3]]
        );
        assert!(vacant_cells(bounds, cells).is_empty());
        assert!(
            vacant_cells(
                Bounds {
                    start: bounds.end,
                    end: bounds.start
                },
                []
            )
            .is_empty()
        );
    }
}
