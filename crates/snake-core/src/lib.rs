pub mod assets;
pub mod collision;
pub mod game_object;
pub mod models;
pub mod movement;
pub mod random;
pub mod render;
pub mod signal;

#[cfg(test)]
pub(crate) mod test_utils;

use std::ops::{Add, AddAssign};

pub use game_object::{GameObject, GameObjectId};
pub use movement::{Move, MoveDirection};
pub use render::{Render, RenderItem, Texture, TextureColor, ZIndex};

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Transform {
    pub position: Vector2Int,
    pub rotation: TransformRotation,
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct TransformRotation {
    body: Rotation,
    look: Option<Rotation>,
}

impl TransformRotation {
    #[must_use]
    pub const fn body(self) -> Rotation {
        self.body
    }

    #[must_use]
    pub fn look(self) -> Rotation {
        self.look.unwrap_or(self.body)
    }

    pub fn look_to(&mut self, direction: MoveDirection) -> Rotation {
        let requested_rotation = Rotation::from(direction);

        if requested_rotation == self.body {
            self.look = None;
        } else if let Some(rotation) = movement::compute_new_rotation(direction, self.body) {
            self.look = Some(rotation);
        }

        self.look()
    }

    pub fn consume_look(&mut self) -> Rotation {
        self.body = self.look.take().unwrap_or(self.body);
        self.body
    }
}

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Vector2Int {
    pub x: i32,
    pub y: i32,
}

impl Vector2Int {
    pub const ZERO: Self = Vector2Int { x: 0, y: 0 };

    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// calculates midpoint rounded down
    pub fn midpoint(self, rhs: Self) -> Self {
        Self {
            x: (self.x + rhs.x) / 2,
            y: (self.y + rhs.y) / 2,
        }
    }
}

impl From<(i32, i32)> for Vector2Int {
    fn from((x, y): (i32, i32)) -> Self {
        Self::new(x, y)
    }
}
impl Add for Vector2Int {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x.saturating_add(rhs.x), self.y.saturating_add(rhs.y))
    }
}
impl AddAssign for Vector2Int {
    fn add_assign(&mut self, rhs: Self) {
        self.x = self.x.saturating_add(rhs.x);
        self.y = self.y.saturating_add(rhs.y);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Bounds {
    pub start: Vector2Int,
    pub end: Vector2Int,
}
impl Bounds {
    pub fn middle(&self) -> Vector2Int {
        self.start.midpoint(self.end)
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Rotation(u16);
impl Rotation {
    pub const RIGHT: Rotation = Self(0);
    pub const DOWN: Rotation = Self(90);
    pub const LEFT: Rotation = Self(180);
    pub const UP: Rotation = Self(270);
    pub const DEGREES_UPPER: u16 = 360;

    pub fn new(value: u16) -> Self {
        Self(value.rem_euclid(Self::DEGREES_UPPER))
    }

    pub fn rotate(self, rotation: Rotation) -> Self {
        Self::new(self.into_inner() + rotation.into_inner())
    }

    pub const fn into_inner(self) -> u16 {
        self.0
    }
}
impl From<MoveDirection> for Rotation {
    fn from(value: MoveDirection) -> Self {
        match value {
            MoveDirection::Right => Self::RIGHT,
            MoveDirection::Down => Self::DOWN,
            MoveDirection::Left => Self::LEFT,
            MoveDirection::Up => Self::UP,
        }
    }
}
