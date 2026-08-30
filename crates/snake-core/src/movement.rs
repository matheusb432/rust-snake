#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

pub trait Move {
    fn move_to(&mut self, direction: MoveDirection);
}
