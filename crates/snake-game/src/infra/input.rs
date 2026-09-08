//! input adapters
use std::collections::VecDeque;

use crossterm::event::KeyCode;
use snake_core::MoveDirection;

/// the input queue stores up to `InputQueue::MAX` items, anything exceeding that is discarded
#[derive(Debug, Clone)]
pub struct InputQueue(VecDeque<InputKey>);

impl InputQueue {
    pub const MAX_ITEMS: usize = 10;
    pub fn new() -> Self {
        Self(Self::empty_inner())
    }

    fn empty_inner() -> VecDeque<InputKey> {
        VecDeque::with_capacity(Self::MAX_ITEMS)
    }

    /// returns current queue and leaves inner queue cleared
    pub fn flush(&mut self) -> VecDeque<InputKey> {
        let mut flushed_queue = Self::empty_inner();
        // this is more convenient (and i _think_ efficient, but havent measured) than cloning self
        // _then_ clearing. its also a choice for a better api to not require this fn to
        // move out of self.
        std::mem::swap(&mut flushed_queue, &mut self.0);

        flushed_queue
    }

    /// will remove first item if size is at max
    pub fn push_back(&mut self, value: InputKey) {
        // best to remove first else it could exceed capacity and resize
        self.pop_if_full();
        self.0.push_back(value);
    }

    fn pop_if_full(&mut self) -> Option<InputKey> {
        if self.0.len() == Self::MAX_ITEMS {
            self.0.pop_front()
        } else {
            None
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) enum InputKey {
    Move(MoveDirection),
    Quit,
    Reset,
    Pause,
    Other,
}

impl InputKey {
    pub(crate) const QUIT_CHARACTER: char = 'q';
    pub(crate) const PAUSE_CHARACTER: char = 'p';
    pub(crate) const RESET_CHARACTER: char = 'r';

    pub fn from_keycode(key_code: KeyCode) -> Self {
        match key_code {
            KeyCode::Up => Self::Move(MoveDirection::Up),
            KeyCode::Down => Self::Move(MoveDirection::Down),
            KeyCode::Left => Self::Move(MoveDirection::Left),
            KeyCode::Right => Self::Move(MoveDirection::Right),
            KeyCode::Char(Self::QUIT_CHARACTER) => Self::Quit,
            KeyCode::Char(Self::PAUSE_CHARACTER) => Self::Pause,
            KeyCode::Char(Self::RESET_CHARACTER) => Self::Reset,
            _ => Self::Other,
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
            assert_eq!(InputKey::from_keycode(key_code), InputKey::Move(direction));
        }
    }

    #[test]
    fn recognizes_quit_key() {
        assert_eq!(
            InputKey::from_keycode(KeyCode::Char(InputKey::QUIT_CHARACTER)),
            InputKey::Quit
        );
    }

    #[test]
    fn recognizes_other_keys_for_the_start_prompt() {
        assert_eq!(InputKey::from_keycode(KeyCode::Enter), InputKey::Other);
    }
}
