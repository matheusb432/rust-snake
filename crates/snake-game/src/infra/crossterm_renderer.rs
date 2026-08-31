use std::io::{self, Stdout, Write};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, PrintStyledContent, Stylize},
};
use snake_core::{GameObject, Texture, TextureColor};

use crate::{
    board::{BOARD_SIZE_X, BOARD_SIZE_Y, Board},
    game::GameObjectIterator,
    render::Renderer,
};

const BOARD_INTERIOR_CELL_WIDTH_COLUMNS: usize = 2;

pub(crate) struct CrosstermRenderer<W = Stdout> {
    output: W,
}

impl<W: Write> CrosstermRenderer<W> {
    fn render_board(&mut self, board: &Board) -> io::Result<()> {
        for board_y in 0..BOARD_SIZE_Y {
            queue!(&mut self.output, MoveTo(0, board_y as u16))?;

            for board_x in 0..BOARD_SIZE_X {
                self.render_texture(
                    board.texture_at(board_x, board_y),
                    Self::board_cell_width_columns(board_x),
                )?;
            }
        }

        Ok(())
    }

    fn render_object(&mut self, object: &dyn GameObject) -> io::Result<()> {
        let position = object.position();
        let grid_x = usize::try_from(position.x).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("object {} has a negative x position", object.id()),
            )
        })?;
        let terminal_y = u16::try_from(position.y).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("object {} has an invalid terminal y position", object.id()),
            )
        })?;
        if usize::from(terminal_y) >= BOARD_SIZE_Y {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("object {} is below the board", object.id()),
            ));
        }

        queue!(
            &mut self.output,
            MoveTo(Self::terminal_column(grid_x)?, terminal_y)
        )?;
        self.render_texture(object.texture(), 1)
    }

    fn render_texture(&mut self, texture: Texture, width_columns: usize) -> io::Result<()> {
        let styled_character = texture
            .character()
            .with(Self::crossterm_color(texture.color()));

        for _ in 0..width_columns {
            queue!(&mut self.output, PrintStyledContent(styled_character))?;
        }

        Ok(())
    }

    fn board_cell_width_columns(board_x: usize) -> usize {
        if board_x == 0 || board_x == BOARD_SIZE_X - 1 {
            1
        } else {
            BOARD_INTERIOR_CELL_WIDTH_COLUMNS
        }
    }

    fn terminal_column(grid_x: usize) -> io::Result<u16> {
        if grid_x >= BOARD_SIZE_X {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("grid x position {grid_x} is outside the board"),
            ));
        }

        let terminal_column = match grid_x {
            0 => 0,
            _ => 1 + (grid_x - 1) * BOARD_INTERIOR_CELL_WIDTH_COLUMNS,
        };

        u16::try_from(terminal_column).map_err(io::Error::other)
    }

    fn crossterm_color(color: TextureColor) -> Color {
        match color {
            TextureColor::White => Color::White,
            TextureColor::DarkGreen => Color::DarkGreen,
            TextureColor::Red => Color::Red,
        }
    }
}

impl Default for CrosstermRenderer<Stdout> {
    fn default() -> Self {
        Self {
            output: io::stdout(),
        }
    }
}

impl<W: Write> Renderer for CrosstermRenderer<W> {
    fn render(&mut self, board: &Board, objects: GameObjectIterator<'_>) -> io::Result<()> {
        self.render_board(board)?;

        for object in objects {
            self.render_object(object)?;
        }

        self.output.flush()
    }
}

#[cfg(test)]
mod tests {
    use snake_core::{GameObject, GameObjectId, Render, Rotation, Texture, Vector2Int, assets};

    use super::{
        BOARD_INTERIOR_CELL_WIDTH_COLUMNS, BOARD_SIZE_X, BOARD_SIZE_Y, Board, CrosstermRenderer,
    };

    struct TestObject {
        id: GameObjectId,
        position: Vector2Int,
    }

    impl TestObject {
        fn at(position: Vector2Int) -> Self {
            Self {
                id: GameObjectId::new(),
                position,
            }
        }
    }

    impl Render for TestObject {
        fn texture(&self) -> Texture {
            assets::APPLE
        }
    }

    impl GameObject for TestObject {
        fn position(&self) -> Vector2Int {
            self.position
        }

        fn rotation(&self) -> Rotation {
            Rotation::default()
        }

        fn id(&self) -> GameObjectId {
            self.id
        }
    }

    #[test]
    fn render_board_keeps_outer_walls_one_column_wide() {
        let mut renderer = CrosstermRenderer { output: Vec::new() };

        renderer.render_board(&Board::new()).unwrap();

        let output = renderer.output;
        assert_eq!(count_byte(&output, b'x'), 4);
        assert_eq!(count_byte(&output, b'|'), (BOARD_SIZE_Y - 2) * 2);
        assert_eq!(
            count_byte(&output, b'_'),
            (BOARD_SIZE_X - 2) * BOARD_INTERIOR_CELL_WIDTH_COLUMNS * 2
        );
        assert_eq!(
            count_byte(&output, b' '),
            (BOARD_SIZE_X - 2) * BOARD_INTERIOR_CELL_WIDTH_COLUMNS * (BOARD_SIZE_Y - 2)
        );
    }

    #[test]
    fn render_object_prints_one_texture_at_its_board_position() {
        let object = TestObject::at(Vector2Int::new(1, 1));
        let mut renderer = CrosstermRenderer { output: Vec::new() };

        renderer.render_object(&object).unwrap();

        assert_eq!(count_byte(&renderer.output, b'&'), 1);
        assert!(
            renderer
                .output
                .windows(6)
                .any(|window| window == b"\x1b[2;2H")
        );
    }

    #[test]
    fn terminal_columns_follow_rendered_cell_widths() {
        let cases = [(0, 0), (1, 1), (22, 43), (23, 45)];

        for (grid_x, terminal_column) in cases {
            assert_eq!(
                CrosstermRenderer::<Vec<u8>>::terminal_column(grid_x).unwrap(),
                terminal_column
            );
        }
    }

    #[test]
    fn render_object_rejects_positions_outside_the_board() {
        for position in [
            Vector2Int::new(-1, 0),
            Vector2Int::new(BOARD_SIZE_X as i32, 0),
            Vector2Int::new(0, -1),
            Vector2Int::new(0, BOARD_SIZE_Y as i32),
        ] {
            let mut renderer = CrosstermRenderer { output: Vec::new() };

            let error = renderer
                .render_object(&TestObject::at(position))
                .unwrap_err();

            assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
        }
    }

    fn count_byte(bytes: &[u8], expected: u8) -> usize {
        bytes.iter().filter(|byte| **byte == expected).count()
    }
}
