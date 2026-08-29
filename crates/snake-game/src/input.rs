//! input adapters
use crossterm::event::KeyCode;
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum InputKey {
    Up,
    Down,
    Left,
    Right,
}
impl InputKey {
    pub fn from_keycode(key_code: KeyCode) -> Option<Self> {
        match key_code {
            KeyCode::Up => Some(Self::Up),
            KeyCode::Down => Some(Self::Down),
            KeyCode::Left => Some(Self::Left),
            KeyCode::Right => Some(Self::Right),
            _ => None,
        }
    }
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
