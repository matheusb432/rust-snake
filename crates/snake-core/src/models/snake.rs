use std::collections::VecDeque;

use crate::{
    GameObject, GameObjectId, Move, MoveDirection, Render, RenderTarget, Rotation, Texture,
    Transform, Vector2Int, assets, movement::compute_new_rotation,
};

/// snake player
#[derive(Debug)]
pub struct Snake {
    id: GameObjectId,
    body: SnakeBody,
    transform: Transform,
}

impl Snake {
    pub fn spawn() -> Self {
        Self {
            id: GameObjectId::new(),
            body: SnakeBody::default(),
            transform: Transform::default(),
        }
    }

    pub fn tick(&mut self) {
        // TODO: uncomment
        // self.move_forward(1);
        // self.body
        //     .move_parts_tick(self.head_direction(), self.transform.position);
    }

    pub fn hp(&self) -> u16 {
        self.body.size()
    }

    // TODO: move to GameObject impl
    pub fn move_forward(&mut self, magnitude: i32) {
        let move_i = Vector2Int::from(match self.head_direction() {
            Rotation::RIGHT => (magnitude, 0),
            Rotation::DOWN => (0, magnitude),
            Rotation::LEFT => (-magnitude, 0),
            Rotation::UP => (0, -magnitude),
            _ => return,
        });
        self.transform.position = self.transform.position + move_i;
    }

    pub fn set_position(&mut self, position: Vector2Int) {
        self.transform.position = position;
    }

    /// direction the snake's head is facing at, dictated by it's rotation
    pub fn head_direction(&self) -> Rotation {
        self.transform.rotation
    }
}

impl Render for Snake {
    fn texture(&self) -> RenderTarget {
        RenderTarget::Many(self.body.parts_textures().collect())
    }
}

impl GameObject for Snake {
    fn position(&self) -> Vector2Int {
        self.transform.position
    }

    fn rotation(&self) -> Rotation {
        self.transform.rotation
    }

    fn id(&self) -> GameObjectId {
        self.id
    }
}

impl Move for Snake {
    fn rotate_to(&mut self, direction: MoveDirection) {
        if let Some(rotation) = compute_new_rotation(direction, self.transform.rotation) {
            self.transform.rotation = rotation;
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SnakeError {
    SizeTooSmall { min: u16, actual: u16 },
    SizeTooBig { max: u16, actual: u16 },
}

#[derive(Debug, PartialEq, Eq)]
pub struct SnakeBody {
    parts: VecDeque<SnakePart>,
}

impl SnakeBody {
    pub const MIN_SIZE: u16 = 4;
    pub const MAX_SIZE: u16 = 500;

    pub fn try_new(size: u16) -> Result<Self, SnakeError> {
        if size < Self::MIN_SIZE {
            return Err(SnakeError::SizeTooSmall {
                min: Self::MIN_SIZE,
                actual: size,
            });
        }
        if size > Self::MAX_SIZE {
            return Err(SnakeError::SizeTooBig {
                max: Self::MAX_SIZE,
                actual: size,
            });
        }

        Ok(Self::from_valid_size(size))
    }

    /// moves the front part in the direction, and every other part in the direction of the
    /// following part.
    pub(in crate::models::snake) fn move_parts_tick(
        &mut self,
        rotation: Rotation,
        head_position: Vector2Int,
    ) {
        let mut next_position = head_position;
        let mut next_rotation = rotation;
        if matches!(
            self.parts.front(),
            Some(SnakePart { position, rotation, .. })
                if *position == next_position && *rotation == next_rotation
        ) {
            return;
        }
        for current_part in &mut self.parts.iter_mut().rev() {
            std::mem::swap(&mut current_part.position, &mut next_position);
            std::mem::swap(&mut current_part.rotation, &mut next_rotation);
        }
    }

    fn size(&self) -> u16 {
        u16::try_from(self.parts.len()).expect("snake body size cannot exceed its u16 maximum")
    }

    fn parts_textures(&self) -> impl Iterator<Item = (Vector2Int, Texture)> + '_ {
        self.parts.iter().map(|part| (part.position, part.texture))
    }

    fn from_valid_size(size: u16) -> Self {
        let capacity = usize::from(Self::MAX_SIZE);
        let mut parts = VecDeque::with_capacity(capacity);
        const HEAD: Texture = assets::SNAKE_HEAD;
        const PART: Texture = assets::SNAKE_PART;
        // TODO: add tail texture
        const TAIL: Texture = assets::SNAKE_PART;

        let parts_last_index = usize::from(size) - 1;
        parts.extend((0..usize::from(size)).map(|index| {
            let texture = match index {
                0 => HEAD,
                index if index == parts_last_index => TAIL,
                _ => PART,
            };
            SnakePart {
                position: Vector2Int::default(),
                rotation: Rotation::default(),
                texture,
            }
        }));

        Self { parts }
    }
}

impl Default for SnakeBody {
    fn default() -> Self {
        Self::from_valid_size(Self::MIN_SIZE)
    }
}

/// the snake's body part
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SnakePart {
    position: Vector2Int,
    /// direction the part is facing
    rotation: Rotation,
    texture: Texture,
}

#[cfg(test)]
mod tests {
    use super::{Snake, SnakeBody, SnakeError};

    #[test]
    fn body_accepts_inclusive_size_bounds() {
        assert!(SnakeBody::try_new(SnakeBody::MIN_SIZE).is_ok());
        assert!(SnakeBody::try_new(SnakeBody::MAX_SIZE).is_ok());
    }

    #[test]
    fn body_rejects_invalid_sizes() {
        let invalid_max = SnakeBody::MAX_SIZE + 1;
        let invalid_min = SnakeBody::MIN_SIZE - 1;

        assert_eq!(
            SnakeBody::try_new(invalid_min),
            Err(SnakeError::SizeTooSmall {
                min: SnakeBody::MIN_SIZE,
                actual: invalid_min,
            })
        );
        assert_eq!(
            SnakeBody::try_new(invalid_max),
            Err(SnakeError::SizeTooBig {
                max: SnakeBody::MAX_SIZE,
                actual: invalid_max,
            })
        )
    }

    #[test]
    fn hp_is_derived_from_body_size() {
        let mut snake = Snake::spawn();
        let body_size = SnakeBody::MIN_SIZE + 3;
        snake.body = SnakeBody::try_new(body_size).unwrap();

        assert_eq!(snake.hp(), body_size);
    }
}
