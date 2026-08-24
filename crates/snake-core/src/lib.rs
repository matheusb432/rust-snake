pub mod input;
pub mod models;

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Transform {
    pub position: Vector2,
    pub rotation: Rotation,
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
