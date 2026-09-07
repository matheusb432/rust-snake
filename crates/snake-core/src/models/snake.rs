use std::{collections::VecDeque, time::Duration};

use crate::{
    Bounds, GameObject, GameObjectId, Move, MoveDirection, Render, RenderItem, Rotation, Texture,
    Transform, Vector2Int, ZIndex, assets, collision::ColliderShape,
    movement::compute_move_forward, signal::SignalEmitter,
};

const MOVEMENT_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnakeSignal {
    Killed { snake_id: GameObjectId },
    FoodEaten { food_id: GameObjectId },
}

/// snake player
#[derive(Debug)]
pub struct Snake {
    id: GameObjectId,
    body: SnakeBody,
    transform: Transform,
    lifecycle: SnakeLifecycle,
    emitter: SignalEmitter<SnakeSignal>,
}

impl Snake {
    pub fn spawn(emitter: SignalEmitter<SnakeSignal>) -> Self {
        Self {
            id: GameObjectId::new(),
            body: SnakeBody::default(),
            transform: Transform::default(),
            lifecycle: SnakeLifecycle::SPAWNED,
            emitter,
        }
    }

    pub fn respawn(&mut self, transform: Transform) {
        self.body = SnakeBody::default();
        self.transform = transform;
        self.lifecycle = SnakeLifecycle::SPAWNED;
    }

    fn kill(&mut self) {
        if let SnakeLifecycle::Alive(state) = self.lifecycle {
            self.lifecycle = SnakeLifecycle::Dead(state.kill());
            self.emitter.emit(SnakeSignal::Killed { snake_id: self.id });
        }
    }

    pub fn resolve_collisions(
        &mut self,
        bounds: Bounds,
        food: impl IntoIterator<Item = (GameObjectId, Vector2Int)>,
    ) {
        if !self.is_alive() {
            return;
        }
        let contact = detect_contact(
            self.head_position(),
            self.occupied_cells().skip(1),
            bounds,
            food,
        );
        if let Some(contact) = contact {
            self.on_collision(contact);
        }
    }

    pub fn on_collision(&mut self, contact: SnakeContact) {
        if !self.is_alive() {
            return;
        }
        match contact {
            SnakeContact::Solid => self.kill(),
            SnakeContact::Food(food_id) => {
                self.add_part();
                self.emitter.emit(SnakeSignal::FoodEaten { food_id });
            }
        }
    }

    pub fn tick(&mut self, delta_time: Duration) {
        let SnakeLifecycle::Alive(state) = &mut self.lifecycle else {
            return;
        };

        state.tick(&mut self.body, &mut self.transform, delta_time);
    }

    pub fn hp(&self) -> u16 {
        self.body.size()
    }

    pub fn add_part(&mut self) {
        // TODO: remove checks to see if alive if unnecessary
        if matches!(self.lifecycle, SnakeLifecycle::Alive(_)) {
            self.body.add_part();
        }
    }

    pub fn set_position(&mut self, position: Vector2Int) {
        if matches!(self.lifecycle, SnakeLifecycle::Alive(_)) {
            self.transform.position = position;
        }
    }

    /// direction the snake's head is facing at, dictated by it's rotation
    pub fn head_direction(&self) -> Rotation {
        self.transform.rotation.look()
    }

    pub fn head_position(&self) -> Vector2Int {
        self.transform.position
    }

    pub fn occupied_cells(&self) -> impl Iterator<Item = Vector2Int> + '_ {
        self.body
            .parts
            .iter()
            .map(|part| self.transform.position + part.position)
    }

    pub fn is_alive(&self) -> bool {
        matches!(self.lifecycle, SnakeLifecycle::Alive(_))
    }

    pub fn request_movement(&mut self, direction: MoveDirection) {
        let SnakeLifecycle::Alive(state) = &mut self.lifecycle else {
            return;
        };

        SnakeState::<Alive>::rotate_to(&mut self.body, &mut self.transform, direction);
        state.state.movement_requested = true;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnakeContact {
    Solid,
    Food(GameObjectId),
}

pub fn detect_contact(
    head: Vector2Int,
    body: impl IntoIterator<Item = Vector2Int>,
    bounds: Bounds,
    food: impl IntoIterator<Item = (GameObjectId, Vector2Int)>,
) -> Option<SnakeContact> {
    if ColliderShape::OutsideBounds(bounds).contains_cell(head)
        || body
            .into_iter()
            .any(|position| ColliderShape::Cell(position).contains_cell(head))
    {
        Some(SnakeContact::Solid)
    } else {
        food.into_iter().find_map(|(id, position)| {
            ColliderShape::Cell(position)
                .contains_cell(head)
                .then_some(SnakeContact::Food(id))
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct Alive {
    movement_interval: Duration,
    movement_time_elapsed: Duration,
    movement_requested: bool,
}

#[derive(Debug, Clone, Copy)]
struct Dead {
    movement_interval: Duration,
}

#[derive(Debug, Clone, Copy)]
struct SnakeState<State> {
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum SnakeLifecycle {
    Alive(SnakeState<Alive>),
    Dead(SnakeState<Dead>),
}
impl SnakeLifecycle {
    pub const SPAWNED: SnakeLifecycle = SnakeLifecycle::Alive(SnakeState::alive(MOVEMENT_INTERVAL));
}
impl SnakeState<Alive> {
    const fn alive(movement_interval: Duration) -> Self {
        Self {
            state: Alive {
                movement_interval,
                movement_time_elapsed: Duration::ZERO,
                movement_requested: false,
            },
        }
    }

    fn tick(&mut self, body: &mut SnakeBody, transform: &mut Transform, delta_time: Duration) {
        self.state.movement_time_elapsed = self
            .state
            .movement_time_elapsed
            .saturating_add(delta_time)
            .min(self.state.movement_interval);
        let did_movement_interval_elapse =
            self.state.movement_time_elapsed == self.state.movement_interval;
        let movement_requested = std::mem::take(&mut self.state.movement_requested);
        if movement_requested || did_movement_interval_elapse {
            self.move_forward(body, transform, 1);
        }
    }

    fn move_forward(&mut self, body: &mut SnakeBody, transform: &mut Transform, magnitude: i32) {
        let position_previous = transform.position;
        let facing_direction = transform.rotation.consume_look();
        let Some(position) = compute_move_forward(transform.position, facing_direction, magnitude)
        else {
            return;
        };
        transform.position = position;

        let movement_offset = Vector2Int::new(
            transform.position.x - position_previous.x,
            transform.position.y - position_previous.y,
        );
        body.move_parts_forward(movement_offset);
        self.state.movement_time_elapsed = Duration::ZERO;
    }

    fn rotate_to(body: &mut SnakeBody, transform: &mut Transform, direction: MoveDirection) {
        let look_rotation = transform.rotation.look_to(direction);
        if let Some(head) = body.parts.front_mut() {
            head.rotation = look_rotation;
        }
    }

    const fn kill(self) -> SnakeState<Dead> {
        SnakeState {
            state: Dead {
                movement_interval: self.state.movement_interval,
            },
        }
    }
}

impl SnakeState<Dead> {
    const fn respawn(self) -> SnakeState<Alive> {
        SnakeState::alive(self.state.movement_interval)
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
        if !matches!(self.lifecycle, SnakeLifecycle::Alive(_)) {
            return;
        }

        SnakeState::<Alive>::rotate_to(&mut self.body, &mut self.transform, direction);
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

    /// pushes a new part at the tail position
    pub fn add_part(&mut self) {
        let Some(part_last) = self.parts.back() else {
            self.parts.push_back(SnakePart::default_part());
            return;
        };
        let part_new = part_last.clone();
        self.parts.push_back(part_new);
    }

    // TODO refactor
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

        parts.extend((0..usize::from(size)).map(|index| {
            let texture = match index {
                0 => SnakePart::HEAD,
                _ => SnakePart::PART,
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

impl SnakePart {
    pub const HEAD: Texture = assets::SNAKE_HEAD;
    pub const PART: Texture = assets::SNAKE_PART;
    pub const fn default_part() -> Self {
        Self {
            position: Vector2Int::ZERO,
            rotation: Rotation::RIGHT,
            texture: Self::PART,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{MOVEMENT_INTERVAL, Snake, SnakeBody, SnakeContact, SnakeError, detect_contact};
    use crate::{
        Bounds, GameObject, GameObjectId, Move, MoveDirection, Rotation, Texture, Vector2Int,
        assets,
        test_utils::{collect_render_items, spawn_snake},
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
    fn solid_contacts_take_precedence_over_food() {
        let bounds = Bounds {
            start: Vector2Int::new(1, 1),
            end: Vector2Int::new(23, 23),
        };
        let head = Vector2Int::new(10, 10);
        let tail_previous = Vector2Int::new(9, 10);
        let food_id = GameObjectId::new();
        let food_other_id = GameObjectId::new();
        for (head, body, food, expected) in [
            (
                head,
                vec![head],
                vec![(food_id, head)],
                Some(SnakeContact::Solid),
            ),
            (
                Vector2Int::new(23, 10),
                vec![],
                vec![(food_id, Vector2Int::new(23, 10))],
                Some(SnakeContact::Solid),
            ),
            (
                head,
                vec![tail_previous],
                vec![(food_other_id, tail_previous), (food_id, head)],
                Some(SnakeContact::Food(food_id)),
            ),
            (head, vec![tail_previous], vec![], None),
            (tail_previous, vec![head], vec![(food_id, head)], None),
        ] {
            assert_eq!(detect_contact(head, body, bounds, food), expected);
        }
    }

    #[test]
    fn hp_is_derived_from_body_size() {
        let mut snake = spawn_snake();
        let body_size = SnakeBody::MIN_SIZE + 3;
        snake.body = SnakeBody::try_new(body_size).unwrap();

        assert_eq!(snake.hp(), body_size);
    }

    #[test]
    fn move_forward_moves_once_and_keeps_body_positions_local_to_the_snake() {
        let mut snake = spawn_snake();
        snake.set_position(Vector2Int::new(10, 10));

        snake.tick(MOVEMENT_INTERVAL);

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
    fn input_movement_does_reset_the_tick_interval() {
        let mut snake = spawn_snake();
        let position = Vector2Int::new(10, 10);
        let one_millisecond = Duration::from_millis(1);
        let almost_one_interval = MOVEMENT_INTERVAL - one_millisecond;
        snake.set_position(position);

        snake.tick(almost_one_interval);
        snake.request_movement(MoveDirection::Right);
        snake.request_movement(MoveDirection::Right);
        snake.tick(Duration::ZERO);
        snake.tick(one_millisecond);

        assert_eq!(snake.position(), Vector2Int::new(11, 10));

        snake.tick(almost_one_interval);

        assert_eq!(snake.position(), Vector2Int::new(12, 10));
    }

    #[test]
    fn killed_snake_does_not_move() {
        let mut snake = spawn_snake();
        let position = Vector2Int::new(10, 10);
        snake.set_position(position);
        snake.kill();

        snake.request_movement(MoveDirection::Down);
        snake.tick(MOVEMENT_INTERVAL);

        assert_eq!(snake.position(), position);
    }

    #[test]
    fn move_forward_preserves_the_body_trail_after_a_turn() {
        let mut snake = spawn_snake();
        snake.set_position(Vector2Int::new(10, 10));
        snake.tick(MOVEMENT_INTERVAL);
        snake.request_movement(MoveDirection::Down);
        snake.tick(Duration::ZERO);

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
        let mut snake = spawn_snake();
        snake.set_position(Vector2Int::new(10, 10));

        snake.rotate_to(MoveDirection::Up);
        assert_head_looks(&snake, Rotation::UP);

        snake.rotate_to(MoveDirection::Left);
        assert_head_looks(&snake, Rotation::UP);

        snake.rotate_to(MoveDirection::Down);
        assert_head_looks(&snake, Rotation::DOWN);
        snake.tick(MOVEMENT_INTERVAL);

        assert_eq!(snake.position(), Vector2Int::new(10, 11));
        assert_head_looks(&snake, Rotation::DOWN);
    }

    #[test]
    fn looking_in_the_body_direction_cancels_the_next_turn() {
        let mut snake = spawn_snake();
        snake.set_position(Vector2Int::new(10, 10));
        snake.rotate_to(MoveDirection::Up);

        snake.rotate_to(MoveDirection::Right);
        snake.tick(MOVEMENT_INTERVAL);

        assert_eq!(snake.position(), Vector2Int::new(11, 10));
        assert_head_looks(&snake, Rotation::RIGHT);
    }

    #[test]
    fn tick_consumes_the_look_for_subsequent_validation() {
        let mut snake = spawn_snake();
        snake.set_position(Vector2Int::new(10, 10));
        snake.rotate_to(MoveDirection::Down);

        snake.tick(MOVEMENT_INTERVAL);
        snake.rotate_to(MoveDirection::Left);
        snake.tick(MOVEMENT_INTERVAL);

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
