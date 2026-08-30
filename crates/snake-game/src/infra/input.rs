//! input adapters
use crossterm::event::KeyCode;
use snake_core::MoveDirection;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) enum InputKey {
    Move(MoveDirection),
    Quit,
}

impl InputKey {
    pub(crate) const QUIT_CHARACTER: char = 'q';

    pub fn from_keycode(key_code: KeyCode) -> Option<Self> {
        match key_code {
            KeyCode::Up => Some(Self::Move(MoveDirection::Up)),
            KeyCode::Down => Some(Self::Move(MoveDirection::Down)),
            KeyCode::Left => Some(Self::Move(MoveDirection::Left)),
            KeyCode::Right => Some(Self::Move(MoveDirection::Right)),
            KeyCode::Char(Self::QUIT_CHARACTER) => Some(Self::Quit),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyCode;
    use snake_core::MoveDirection;

    use super::InputKey;

    #[test]
    fn recognizes_movement_keys() {
        let cases = [
            (KeyCode::Up, MoveDirection::Up),
            (KeyCode::Down, MoveDirection::Down),
            (KeyCode::Left, MoveDirection::Left),
            (KeyCode::Right, MoveDirection::Right),
        ];

        for (key_code, direction) in cases {
            assert_eq!(
                InputKey::from_keycode(key_code),
                Some(InputKey::Move(direction))
            );
        }
    }

    #[test]
    fn recognizes_quit_key() {
        assert_eq!(
            InputKey::from_keycode(KeyCode::Char(InputKey::QUIT_CHARACTER)),
            Some(InputKey::Quit)
        );
    }

    #[test]
    fn ignores_irrelevant_keys() {
        assert_eq!(InputKey::from_keycode(KeyCode::Enter), None);
    }
}
