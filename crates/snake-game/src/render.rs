use std::io;

use crate::{board::Board, game::GameObjectIterator};

pub(crate) trait Renderer {
    fn render(&mut self, board: &Board, objects: GameObjectIterator<'_>) -> io::Result<()>;
}
