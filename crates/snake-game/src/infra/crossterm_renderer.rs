use std::io::{self, Stdout, Write};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, PrintStyledContent, Stylize},
};
use snake_core::{Rotation, Texture, TextureColor};

use crate::render::{RenderCell, RenderFrame, RenderViewport, Renderer};

const GRID_CELL_WIDTH_COLUMNS: usize = 2;

pub(crate) struct CrosstermRenderer<W = Stdout> {
    output: W,
}

impl<W: Write> CrosstermRenderer<W> {
    fn render_cell(&mut self, cell: RenderCell, viewport: RenderViewport) -> io::Result<()> {
        let position_world = cell.position_world();
        let grid_x = usize::try_from(position_world.x).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "render cell has a negative x position: {}",
                    position_world.x
                ),
            )
        })?;
        let terminal_y = u16::try_from(position_world.y).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "render cell has an invalid y position: {}",
                    position_world.y
                ),
            )
        })?;
        if usize::from(terminal_y) >= viewport.height_cells() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "render cell y position {} is outside the viewport",
                    position_world.y
                ),
            ));
        }

        queue!(
            &mut self.output,
            MoveTo(Self::terminal_column(grid_x, viewport)?, terminal_y)
        )?;
        let width_columns = if cell.fills_cell() {
            Self::grid_cell_width_columns(grid_x, viewport)
        } else {
            1
        };
        self.render_texture(cell.texture(), cell.rotation(), width_columns)
    }

    fn render_texture(
        &mut self,
        texture: Texture,
        rotation: Rotation,
        width_columns: usize,
    ) -> io::Result<()> {
        let styled_character = texture
            .character(rotation)
            .with(Self::crossterm_color(texture.color()));

        for _ in 0..width_columns {
            queue!(&mut self.output, PrintStyledContent(styled_character))?;
        }

        Ok(())
    }

    fn grid_cell_width_columns(grid_x: usize, viewport: RenderViewport) -> usize {
        if grid_x == 0 || grid_x == viewport.width_cells() - 1 {
            1
        } else {
            GRID_CELL_WIDTH_COLUMNS
        }
    }

    fn terminal_column(grid_x: usize, viewport: RenderViewport) -> io::Result<u16> {
        if grid_x >= viewport.width_cells() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("grid x position {grid_x} is outside the viewport"),
            ));
        }

        let terminal_column = match grid_x {
            0 => 0,
            _ => 1 + (grid_x - 1) * GRID_CELL_WIDTH_COLUMNS,
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
    fn render(&mut self, frame: &RenderFrame) -> io::Result<()> {
        for cell in frame.cells() {
            self.render_cell(*cell, frame.viewport())?;
        }

        self.output.flush()
    }
}

#[cfg(test)]
mod tests {
    use snake_core::{Rotation, Texture, Vector2Int, ZIndex, assets};

    use super::{CrosstermRenderer, GRID_CELL_WIDTH_COLUMNS};
    use crate::render::{RenderCell, RenderFrame, RenderViewport, Renderer};

    #[test]
    fn render_frame_prints_cells_at_their_world_positions() {
        let frame = RenderFrame::new(
            RenderViewport::new(24, 24),
            vec![
                RenderCell::new(
                    Vector2Int::new(1, 1),
                    assets::APPLE,
                    Rotation::default(),
                    ZIndex::DEFAULT,
                ),
                RenderCell::new(
                    Vector2Int::new(2, 3),
                    assets::SNAKE_PART,
                    Rotation::default(),
                    ZIndex::DEFAULT,
                ),
            ],
        );
        let mut renderer = CrosstermRenderer { output: Vec::new() };

        renderer.render(&frame).unwrap();

        assert_eq!(count_byte(&renderer.output, b'&'), 1);
        assert_eq!(count_byte(&renderer.output, b'~'), 1);
        assert!(
            renderer
                .output
                .windows(6)
                .any(|window| window == b"\x1b[2;2H")
        );
    }

    #[test]
    fn filled_cells_use_projected_grid_widths() {
        let viewport = RenderViewport::new(4, 1);
        let frame = RenderFrame::new(
            viewport,
            vec![
                filled_render_cell(
                    Vector2Int::new(0, 0),
                    assets::WALL_DIAGONAL,
                    Rotation::default(),
                ),
                filled_render_cell(Vector2Int::new(1, 0), assets::WALL, Rotation::DOWN),
                filled_render_cell(
                    Vector2Int::new(3, 0),
                    assets::WALL_DIAGONAL,
                    Rotation::default(),
                ),
            ],
        );
        let mut renderer = CrosstermRenderer { output: Vec::new() };

        renderer.render(&frame).unwrap();

        assert_eq!(count_byte(&renderer.output, b'x'), 2);
        assert_eq!(count_byte(&renderer.output, b'_'), GRID_CELL_WIDTH_COLUMNS);
    }

    #[test]
    fn terminal_columns_follow_projected_cell_widths() {
        let viewport = RenderViewport::new(24, 24);
        let cases = [(0, 0), (1, 1), (22, 43), (23, 45)];

        for (grid_x, terminal_column) in cases {
            assert_eq!(
                CrosstermRenderer::<Vec<u8>>::terminal_column(grid_x, viewport).unwrap(),
                terminal_column
            );
        }
    }

    #[test]
    fn render_rejects_positions_outside_the_viewport() {
        let viewport = RenderViewport::new(24, 24);
        for position in [
            Vector2Int::new(-1, 0),
            Vector2Int::new(24, 0),
            Vector2Int::new(0, -1),
            Vector2Int::new(0, 24),
        ] {
            let frame = RenderFrame::new(
                viewport,
                vec![RenderCell::new(
                    position,
                    assets::APPLE,
                    Rotation::default(),
                    ZIndex::DEFAULT,
                )],
            );
            let mut renderer = CrosstermRenderer { output: Vec::new() };

            let error = renderer.render(&frame).unwrap_err();

            assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
        }
    }

    fn filled_render_cell(
        position_world: Vector2Int,
        texture: Texture,
        rotation: Rotation,
    ) -> RenderCell {
        RenderCell::from_filled_cell(position_world, texture, rotation, ZIndex::BACKGROUND)
    }

    fn count_byte(bytes: &[u8], expected: u8) -> usize {
        bytes.iter().filter(|byte| **byte == expected).count()
    }
}
