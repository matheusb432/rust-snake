use crate::{Rotation, Vector2Int};

/// Draw order key where lower values render first.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZIndex(i32);

impl ZIndex {
    pub const BACKGROUND: Self = Self(-1);
    pub const DEFAULT: Self = Self(0);

    #[must_use]
    pub const fn new(value: i32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureColor {
    White,
    DarkGreen,
    Green,
    Gold,
    Red,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Texture {
    character: TextureCharacter,
    color: TextureColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextureCharacter {
    Fixed(char),
    Rotated {
        right: char,
        down: char,
        left: char,
        up: char,
    },
}

impl Texture {
    #[must_use]
    pub const fn new(character: char, color: TextureColor) -> Self {
        Self {
            character: TextureCharacter::Fixed(character),
            color,
        }
    }

    #[must_use]
    pub const fn new_rotated(
        right: char,
        down: char,
        left: char,
        up: char,
        color: TextureColor,
    ) -> Self {
        Self {
            character: TextureCharacter::Rotated {
                right,
                down,
                left,
                up,
            },
            color,
        }
    }

    #[must_use]
    pub const fn color(self) -> TextureColor {
        self.color
    }

    #[must_use]
    pub const fn character(self, rotation: Rotation) -> char {
        match self.character {
            TextureCharacter::Fixed(character) => character,
            TextureCharacter::Rotated {
                right,
                down,
                left,
                up,
            } => match rotation {
                Rotation::DOWN => down,
                Rotation::LEFT => left,
                Rotation::UP => up,
                _ => right,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderSpace {
    World,
    Screen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderItem {
    position_local: Vector2Int,
    texture: Texture,
    rotation: Rotation,
    z_index: ZIndex,
    fills_cell: bool,
    space: RenderSpace,
}

impl RenderItem {
    #[must_use]
    pub const fn glyph(
        position_local: Vector2Int,
        texture: Texture,
        rotation: Rotation,
        z_index: ZIndex,
    ) -> Self {
        Self {
            position_local,
            texture,
            rotation,
            z_index,
            fills_cell: false,
            space: RenderSpace::World,
        }
    }

    #[must_use]
    pub const fn filled_cell(
        position_local: Vector2Int,
        texture: Texture,
        rotation: Rotation,
        z_index: ZIndex,
    ) -> Self {
        Self {
            position_local,
            texture,
            rotation,
            z_index,
            fills_cell: true,
            space: RenderSpace::World,
        }
    }

    #[must_use]
    pub const fn screen_glyph(
        position_local: Vector2Int,
        texture: Texture,
        z_index: ZIndex,
    ) -> Self {
        Self {
            position_local,
            texture,
            rotation: Rotation::RIGHT,
            z_index,
            fills_cell: false,
            space: RenderSpace::Screen,
        }
    }

    #[must_use]
    pub const fn space(self) -> RenderSpace {
        self.space
    }

    #[must_use]
    pub const fn position_local(self) -> Vector2Int {
        self.position_local
    }

    #[must_use]
    pub const fn texture(self) -> Texture {
        self.texture
    }

    #[must_use]
    pub const fn rotation(self) -> Rotation {
        self.rotation
    }

    #[must_use]
    pub const fn z_index(self) -> ZIndex {
        self.z_index
    }

    #[must_use]
    pub const fn fills_cell(self) -> bool {
        self.fills_cell
    }
}

pub trait Render {
    fn visit_render_items(&self, visit: &mut dyn FnMut(RenderItem));
}
