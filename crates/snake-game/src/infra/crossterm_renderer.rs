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

pub(crate) struct CrosstermRenderer {
    output: Stdout,
}

impl CrosstermRenderer {
    fn render_board(&mut self, board: &Board) -> io::Result<()> {
        for board_y in 0..BOARD_SIZE_Y {
            queue!(&mut self.output, MoveTo(0, board_y as u16))?;

            for board_x in 0..BOARD_SIZE_X {
                self.render_texture(
                    board.texture_at(board_x, board_y),
                    Self::cell_width_columns(board_x),
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
        queue!(
            &mut self.output,
            MoveTo(Self::terminal_column(grid_x)?, terminal_y)
        )?;
        self.render_texture(object.texture(), Self::cell_width_columns(grid_x))
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

    fn cell_width_columns(grid_x: usize) -> usize {
        if grid_x == 0 || grid_x == BOARD_SIZE_X - 1 {
            1
        } else {
            BOARD_INTERIOR_CELL_WIDTH_COLUMNS
        }
    }

    fn terminal_column(grid_x: usize) -> io::Result<u16> {
        let terminal_column = match grid_x {
            0 => 0,
            1..BOARD_SIZE_X => grid_x * BOARD_INTERIOR_CELL_WIDTH_COLUMNS - 1,
            _ => grid_x * BOARD_INTERIOR_CELL_WIDTH_COLUMNS - 2,
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

impl Default for CrosstermRenderer {
    fn default() -> Self {
        Self {
            output: io::stdout(),
        }
    }
}

impl Renderer for CrosstermRenderer {
    fn render(&mut self, board: &Board, objects: GameObjectIterator<'_>) -> io::Result<()> {
        self.render_board(board)?;

        for object in objects {
            self.render_object(object)?;
        }

        self.output.flush()
    }
}
