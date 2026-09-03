use crate::{Rotation, Vector2Int};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

pub trait Move {
    fn rotate_to(&mut self, direction: MoveDirection);
}

/// (0,0) is top-left, so down is negative `y`
pub fn compute_move_forward(
    position: Vector2Int,
    rotation: Rotation,
    magnitude: i32,
) -> Option<Vector2Int> {
    let movement_offset = Vector2Int::from(match rotation {
        Rotation::RIGHT => (magnitude, 0),
        Rotation::DOWN => (0, magnitude),
        Rotation::LEFT => (-magnitude, 0),
        Rotation::UP => (0, -magnitude),
        _ => return None,
    });

    Some(position + movement_offset)
}

/// computes some new rotation by input, or none if no change to it is necessary.
pub fn compute_new_rotation(direction: MoveDirection, rotation: Rotation) -> Option<Rotation> {
    match (direction, rotation) {
        (MoveDirection::Up, Rotation::RIGHT | Rotation::LEFT) => Some(Rotation::UP),
        (MoveDirection::Right, Rotation::UP | Rotation::DOWN) => Some(Rotation::RIGHT),
        (MoveDirection::Down, Rotation::RIGHT | Rotation::LEFT) => Some(Rotation::DOWN),
        (MoveDirection::Left, Rotation::UP | Rotation::DOWN) => Some(Rotation::LEFT),
        _ => None,
    }
}
#[cfg(test)]
mod tests {
    use crate::{
        MoveDirection, Rotation, Vector2Int,
        movement::{compute_move_forward, compute_new_rotation},
    };

    #[test]
    fn compute_move_forward_moves_in_the_facing_direction() {
        let position = Vector2Int::new(10, 10);
        let cases = [
            (Rotation::RIGHT, Vector2Int::new(12, 10)),
            (Rotation::DOWN, Vector2Int::new(10, 12)),
            (Rotation::LEFT, Vector2Int::new(8, 10)),
            (Rotation::UP, Vector2Int::new(10, 8)),
        ];

        for (rotation, expected_position) in cases {
            assert_eq!(
                Some(expected_position),
                compute_move_forward(position, rotation, 2)
            );
        }
    }

    #[test]
    fn compute_new_rotation_computes_some() {
        let cases = [
            (Rotation::LEFT, MoveDirection::Up, Rotation::UP),
            (Rotation::RIGHT, MoveDirection::Up, Rotation::UP),
            (Rotation::UP, MoveDirection::Left, Rotation::LEFT),
            (Rotation::DOWN, MoveDirection::Right, Rotation::RIGHT),
        ];

        for (rotation, input_key, expected_rotation) in cases {
            assert_eq!(
                Some(expected_rotation),
                compute_new_rotation(input_key, rotation)
            );
        }
    }

    #[test]
    fn compute_new_rotation_returns_none_when_no_rotation_necessary() {
        let cases = [
            (Rotation::UP, MoveDirection::Up),
            (Rotation::DOWN, MoveDirection::Up),
            (Rotation::LEFT, MoveDirection::Left),
            (Rotation::RIGHT, MoveDirection::Right),
        ];

        for (rotation, input_key) in cases {
            assert_eq!(None, compute_new_rotation(input_key, rotation));
        }
    }
}
