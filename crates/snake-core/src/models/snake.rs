use std::io::ErrorKind::StaleNetworkFileHandle;

use crate::{Rotation, Transform, input::InputKey};

/// snake player
#[derive(Debug)]
pub struct Snake {
    hp: SnakeHp,
    body: SnakeBody,
    transform: Transform,
}

impl Snake {
    // TODO: impl
    pub fn spawn() -> Self {
        let hp = SnakeHp::default();
        Self {
            hp,
            body: SnakeBody::from_hp(hp),
            transform: Transform::default(),
        }
    }

    pub fn kill(&mut self) {
        self.hp = SnakeHp::ZERO;
    }

    // TODO: create based on `>~~{`
    pub fn render_body(&self) -> String {
        todo!()
    }

    pub fn move_to(&mut self, input: InputKey) {
        if let Some(rotation) = compute_new_rotation(input, self.transform.rotation) {
            self.transform.rotation = rotation;
        }
    }

    /// direction the snake's head is facing at, dictated by it's rotation
    pub fn head_direction(&self) -> SnakeRotation {
        self.transform.rotation.into()
    }

    fn compute_parts_size(&self) {}
}

/// computes some new rotation by input, or none if no change to it is necessary.
fn compute_new_rotation(input: InputKey, rotation: Rotation) -> Option<Rotation> {
    match (input, rotation) {
        // TODO: study the mathematical approach here, this is reasonably only bc of the 4-axis rotation constraint.
        (InputKey::Up, Rotation::RIGHT | Rotation::LEFT) => Some(Rotation::UP),
        (InputKey::Right, Rotation::UP | Rotation::DOWN) => Some(Rotation::RIGHT),
        (InputKey::Down, Rotation::RIGHT | Rotation::LEFT) => Some(Rotation::DOWN),
        (InputKey::Left, Rotation::UP | Rotation::DOWN) => Some(Rotation::LEFT),
        _ => None,
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SnakeError {
    SizeTooBig { max: u16, actual: u16 },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SnakeHp(u16);

impl SnakeHp {
    pub const ZERO: Self = Self(0);
    pub const MIN_VALUE: u16 = 4;
    pub fn into_inner(self) -> u16 {
        self.0
    }
}
impl Default for SnakeHp {
    fn default() -> Self {
        Self(Self::MIN_VALUE)
    }
}

// TODO: make logic to order each snake part
/// body that governs which part is head/appendage/tail of the snake.
#[derive(Debug, PartialEq, Eq)]
pub struct SnakeBody(Vec<SnakePart>);
impl SnakeBody {
    pub const MAX_SIZE: u16 = 500;
    pub fn from_hp(hp: SnakeHp) -> Self {
        Self::parts_from_size(hp.into_inner())
    }

    pub fn try_new(size: u16) -> Result<Self, SnakeError> {
        if size > Self::MAX_SIZE {
            return Err(SnakeError::SizeTooBig {
                max: Self::MAX_SIZE,
                actual: size,
            });
        }

        Ok(Self::parts_from_size(size))
    }

    // TODO: make fns to bump parts and mutate by position (to rotate the snek)

    fn parts_from_size(size: u16) -> SnakeBody {
        Self(vec![SnakePart::default(); size.into()])
    }
}

/// the snake's body part
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct SnakePart {
    /// direction the part is facing
    rotation: SnakeRotation,
}

/// snake can only move in 90 deg increments
#[repr(u16)]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum SnakeRotation {
    #[default]
    Deg0 = 0,
    // TODO: how to use u16
    // Deg0 = 0_u16,
    Deg90 = 90,
    Deg180 = 180,
    Deg270 = 270,
}
impl SnakeRotation {
    pub fn into_inner(self) -> u16 {
        self as u16
    }
}
impl From<Rotation> for SnakeRotation {
    // TODO: remove if it does not make sense
    fn from(value: Rotation) -> Self {
        match value.into_inner() {
            0..=89 => Self::Deg0,
            90..=179 => Self::Deg90,
            180..=269 => Self::Deg180,
            270 => Self::Deg270,
            // TODO: return err
            _ => panic!("invalid rotation"),
        }
    }
}
impl From<SnakeRotation> for Rotation {
    fn from(value: SnakeRotation) -> Self {
        Self::new(value.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Rotation, input::InputKey, models::snake::compute_new_rotation};

    #[test]
    fn compute_new_rotation_computes_some() {
        let cases = [
            (Rotation::LEFT, InputKey::Up, Rotation::UP),
            (Rotation::RIGHT, InputKey::Up, Rotation::UP),
            (Rotation::UP, InputKey::Left, Rotation::LEFT),
            (Rotation::DOWN, InputKey::Right, Rotation::RIGHT),
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
            (Rotation::UP, InputKey::Up),
            (Rotation::DOWN, InputKey::Up),
            (Rotation::LEFT, InputKey::Left),
            (Rotation::RIGHT, InputKey::Right),
        ];

        for (rotation, input_key) in cases {
            assert_eq!(None, compute_new_rotation(input_key, rotation));
        }
    }
}
