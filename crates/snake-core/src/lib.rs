pub mod assets;
pub mod game_object;
pub mod models;
pub mod movement;
pub mod render;

pub use game_object::{GameObject, GameObjectId};
pub use movement::{Move, MoveDirection};
pub use render::{Render, RenderTarget, Texture, TextureColor};

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Transform {
    pub position: Vector2Int,
    pub rotation: Rotation,
}

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Vector2Int {
    pub x: i32,
    pub y: i32,
}

impl Vector2Int {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Bounds {
    pub start: Vector2Int,
    pub end: Vector2Int,
}

// TODO: implement vec floating point calcs
/// vector 2 for game coords
#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Rotation(u16);
impl Rotation {
    pub const RIGHT: Rotation = Self(0);
    pub const DOWN: Rotation = Self(90);
    pub const LEFT: Rotation = Self(180);
    pub const UP: Rotation = Self(270);
    pub const DEGREES_UPPER: u16 = 360;
    // TODO: implement (might be better to be infallible. 750° is fine to be interpreted as 30°)
    pub fn new(value: u16) -> Self {
        Self(value.rem_euclid(Self::DEGREES_UPPER))
    }

    pub fn rotate(self, rotation: Rotation) -> Self {
        Self::new(self.into_inner() + rotation.into_inner())
    }

    pub fn into_inner(self) -> u16 {
        self.0
    }
}
