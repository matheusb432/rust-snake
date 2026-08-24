#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum InputKey {
    Up,
    Down,
    Left,
    Right,
}

impl std::fmt::Display for InputKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Up => write!(f, "Up"),
            Self::Down => write!(f, "Down"),
            Self::Left => write!(f, "Left"),
            Self::Right => write!(f, "Right"),
        }
    }
}
