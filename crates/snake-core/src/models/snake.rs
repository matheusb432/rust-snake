use std::collections::VecDeque;

use crate::{
    GameObject, GameObjectId, Move, MoveDirection, Render, RenderItem, Rotation, Texture,
    Transform, Vector2Int, ZIndex, assets, movement::compute_move_forward,
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
        self.rotate();
    }

    pub fn move_forward(&mut self, magnitude: i32) {
        let position_previous = self.transform.position;
        let facing_direction = self.transform.rotation.body();
        let Some(position) =
            compute_move_forward(self.transform.position, facing_direction, magnitude)
        else {
            return;
        };
        self.transform.position = position;

        let movement_offset = Vector2Int::new(
            self.transform.position.x - position_previous.x,
            self.transform.position.y - position_previous.y,
        );
        self.body.move_parts_forward(movement_offset);
    }

    pub fn hp(&self) -> u16 {
        self.body.size()
    }

    pub fn set_position(&mut self, position: Vector2Int) {
        self.transform.position = position;
    }

    /// direction the snake's head is facing at, dictated by it's rotation
    pub fn head_direction(&self) -> Rotation {
        self.transform.rotation.look()
    }

    fn rotate(&mut self) {
        self.transform.rotation.consume_look();
    }
}

impl Render for Snake {
    fn visit_render_items(&self, visit: &mut dyn FnMut(RenderItem)) {
        for (position_local, texture, rotation) in self.body.parts_textures() {
            visit(RenderItem::glyph(
                position_local,
                texture,
                rotation,
                ZIndex::DEFAULT,
            ));
        }
    }
}

impl GameObject for Snake {
    fn position(&self) -> Vector2Int {
        self.transform.position
    }

    fn rotation(&self) -> Rotation {
        self.transform.rotation.look()
    }

    fn id(&self) -> GameObjectId {
        self.id
    }
}

impl Move for Snake {
    fn rotate_to(&mut self, direction: MoveDirection) {
        let look_rotation = self.transform.rotation.look_to(direction);
        if let Some(head) = self.body.parts.front_mut() {
            head.rotation = look_rotation;
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
    pub(in crate::models::snake) fn move_parts_forward(&mut self, movement_offset: Vector2Int) {
        if movement_offset == Vector2Int::ZERO {
            return;
        }

        for idx in (1..self.parts.len()).rev() {
            let predecessor = self.parts[idx - 1].clone();
            let part = &mut self.parts[idx];
            part.position = Vector2Int::new(
                predecessor.position.x - movement_offset.x,
                predecessor.position.y - movement_offset.y,
            );
            part.rotation = predecessor.rotation;
        }

        if let Some(head) = self.parts.front_mut() {
            head.position = Vector2Int::default();
        }
    }

    fn size(&self) -> u16 {
        u16::try_from(self.parts.len()).expect("snake body size cannot exceed its u16 maximum")
    }

    fn parts_textures(&self) -> impl Iterator<Item = (Vector2Int, Texture, Rotation)> + '_ {
        self.parts
            .iter()
            .map(|part| (part.position, part.texture, part.rotation))
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
                position: Vector2Int::new(-(index as i32), 0),
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
    use crate::{
        GameObject, Move, MoveDirection, Rotation, Texture, Vector2Int, assets,
        test_utils::collect_render_items,
    };

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

    #[test]
    fn move_forward_moves_once_and_keeps_body_positions_local_to_the_snake() {
        let mut snake = Snake::spawn();
        snake.set_position(Vector2Int::new(10, 10));

        snake.move_forward(1);

        assert_eq!(snake.position(), Vector2Int::new(11, 10));
        assert_eq!(
            render_positions_local(&snake),
            [
                Vector2Int::new(0, 0),
                Vector2Int::new(-1, 0),
                Vector2Int::new(-2, 0),
                Vector2Int::new(-3, 0),
            ]
        );
    }

    #[test]
    fn move_forward_preserves_the_body_trail_after_a_turn() {
        let mut snake = Snake::spawn();
        snake.set_position(Vector2Int::new(10, 10));
        snake.move_forward(1);
        snake.rotate_to(MoveDirection::Down);
        snake.tick();

        snake.move_forward(1);

        assert_eq!(snake.position(), Vector2Int::new(11, 11));
        assert_eq!(
            render_positions_local(&snake),
            [
                Vector2Int::new(0, 0),
                Vector2Int::new(0, -1),
                Vector2Int::new(-1, -1),
                Vector2Int::new(-2, -1),
            ]
        );
        assert_eq!(
            render_textures(&snake),
            [
                assets::SNAKE_HEAD,
                assets::SNAKE_PART,
                assets::SNAKE_PART,
                assets::SNAKE_PART,
            ]
        );
        assert_head_looks(&snake, Rotation::DOWN);
    }

    #[test]
    fn latest_valid_look_replaces_the_next_movement() {
        let mut snake = Snake::spawn();
        snake.set_position(Vector2Int::new(10, 10));

        snake.rotate_to(MoveDirection::Up);
        assert_head_looks(&snake, Rotation::UP);

        snake.rotate_to(MoveDirection::Left);
        assert_head_looks(&snake, Rotation::UP);

        snake.rotate_to(MoveDirection::Down);
        assert_head_looks(&snake, Rotation::DOWN);
        snake.tick();

        snake.move_forward(1);

        assert_eq!(snake.position(), Vector2Int::new(10, 11));
        assert_head_looks(&snake, Rotation::DOWN);
    }

    #[test]
    fn looking_in_the_body_direction_cancels_the_next_turn() {
        let mut snake = Snake::spawn();
        snake.set_position(Vector2Int::new(10, 10));
        snake.rotate_to(MoveDirection::Up);

        snake.rotate_to(MoveDirection::Right);
        snake.tick();
        snake.move_forward(1);

        assert_eq!(snake.position(), Vector2Int::new(11, 10));
        assert_head_looks(&snake, Rotation::RIGHT);
    }

    #[test]
    fn tick_consumes_the_look_for_subsequent_validation() {
        let mut snake = Snake::spawn();
        snake.set_position(Vector2Int::new(10, 10));
        snake.rotate_to(MoveDirection::Down);

        snake.tick();
        snake.move_forward(1);
        snake.rotate_to(MoveDirection::Left);
        snake.tick();
        snake.move_forward(1);

        assert_eq!(snake.position(), Vector2Int::new(9, 11));
    }

    fn render_positions_local(snake: &Snake) -> Vec<Vector2Int> {
        collect_render_items(snake)
            .into_iter()
            .map(|item| item.position_local())
            .collect()
    }

    fn render_textures(snake: &Snake) -> Vec<Texture> {
        collect_render_items(snake)
            .into_iter()
            .map(|item| item.texture())
            .collect()
    }

    #[track_caller]
    fn assert_head_looks(snake: &Snake, expected_rotation: Rotation) {
        let head = collect_render_items(snake)
            .into_iter()
            .next()
            .expect("a snake should render a head");

        assert_eq!(
            (head.texture(), head.rotation()),
            (assets::SNAKE_HEAD, expected_rotation)
        );
    }
}
