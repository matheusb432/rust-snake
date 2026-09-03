//! input adapters
use std::collections::VecDeque;

use crossterm::event::KeyCode;
use snake_core::MoveDirection;

/// the input queue stores up to `InputQueue::MAX` items, anything exceeding that is discarded
#[derive(Debug, Clone)]
pub struct InputQueue(VecDeque<InputKey>);

pub enum InputQueuePushOk {
    RemovedFirst(InputKey),
    DidNotRemove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputQueueState {
    Empty,
    Full,
    /// has N spaces open
    VacantWith(usize),
}

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
    pub fn push_back(&mut self, value: InputKey) -> InputQueuePushOk {
        // best to remove first else it could exceed capacity and resize
        let removed_item = self.pop_if_full();
        self.0.push_back(value);

        match removed_item {
            Some(removed_item) => InputQueuePushOk::RemovedFirst(removed_item),
            None => InputQueuePushOk::DidNotRemove,
        }
    }

    fn pop_if_full(&mut self) -> Option<InputKey> {
        if self.0.len() == Self::MAX_ITEMS {
            self.0.pop_front()
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn get_state(&self) -> InputQueueState {
        let vacant_slots = Self::MAX_ITEMS - self.0.len();
        match vacant_slots {
            Self::MAX_ITEMS => InputQueueState::Empty,
            0 => InputQueueState::Full,
            vacant_slots => InputQueueState::VacantWith(vacant_slots),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) enum InputKey {
    Move(MoveDirection),
    Quit,
    Reset,
    Pause,
}

impl InputKey {
    pub(crate) const QUIT_CHARACTER: char = 'q';
    pub(crate) const PAUSE_CHARACTER: char = 'p';
    pub(crate) const RESET_CHARACTER: char = 'r';

    pub fn from_keycode(key_code: KeyCode) -> Option<Self> {
        match key_code {
            KeyCode::Up => Some(Self::Move(MoveDirection::Up)),
            KeyCode::Down => Some(Self::Move(MoveDirection::Down)),
            KeyCode::Left => Some(Self::Move(MoveDirection::Left)),
            KeyCode::Right => Some(Self::Move(MoveDirection::Right)),
            KeyCode::Char(Self::QUIT_CHARACTER) => Some(Self::Quit),
            KeyCode::Char(Self::PAUSE_CHARACTER) => Some(Self::Pause),
            KeyCode::Char(Self::RESET_CHARACTER) => Some(Self::Reset),
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
