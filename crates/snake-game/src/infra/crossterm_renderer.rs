use std::io::{Stdout, Write, stdout};

use anyhow::{Context, Result, ensure};
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, PrintStyledContent, Stylize},
    terminal::{BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate},
};
use snake_core::TextureColor;

use crate::render::{RenderCell, RenderFrame, RenderViewport, Renderer};

const GRID_CELL_WIDTH_COLUMNS: usize = 2;

pub(crate) struct CrosstermRenderer<W = Stdout> {
    output: W,
    terminal_frame_previous: Option<TerminalFrame>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TerminalPosition([u16; 2]);

impl TerminalPosition {
    const fn new(column: u16, row: u16) -> Self {
        Self([column, row])
    }

    fn try_from_render_position(
        position_world: snake_core::Vector2Int,
        viewport: RenderViewport,
    ) -> Result<Self> {
        let grid_column = usize::try_from(position_world.x).with_context(|| {
            format!(
                "render cell x position {} cannot be represented in the terminal grid",
                position_world.x
            )
        })?;
        let row = u16::try_from(position_world.y).with_context(|| {
            format!(
                "render cell y position {} cannot be represented in the terminal grid",
                position_world.y
            )
        })?;

        ensure!(
            grid_column < viewport.width_cells(),
            "render cell x position {} is outside the viewport",
            position_world.x
        );
        ensure!(
            usize::from(row) < viewport.height_cells(),
            "render cell y position {} is outside the viewport",
            position_world.y
        );

        Ok(Self::new(Self::project_grid_column(grid_column)?, row))
    }

    fn project_grid_column(grid_column: usize) -> Result<u16> {
        let column = match grid_column {
            0 => 0,
            _ => grid_column
                .checked_sub(1)
                .and_then(|column| column.checked_mul(GRID_CELL_WIDTH_COLUMNS))
                .and_then(|column| column.checked_add(1))
                .context("render grid width exceeds terminal column capacity")?,
        };

        u16::try_from(column).context("render grid width exceeds terminal column capacity")
    }

    const fn column(self) -> u16 {
        self.0[0]
    }

    const fn row(self) -> u16 {
        self.0[1]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TerminalFrame {
    viewport: RenderViewport,
    width_columns: u16,
    height_rows: u16,
    cells: Vec<TerminalCell>,
}

impl TerminalFrame {
    fn try_from_render_viewport(viewport: RenderViewport) -> Result<Self> {
        let width_columns = Self::terminal_width_columns(viewport)?;
        let height_rows = u16::try_from(viewport.height_cells())
            .context("render viewport height exceeds terminal row capacity")?;
        let cells = (0..height_rows)
            .flat_map(|row| {
                (0..width_columns)
                    .map(move |column| TerminalCell::blank(TerminalPosition::new(column, row)))
            })
            .collect();

        Ok(Self {
            viewport,
            width_columns,
            height_rows,
            cells,
        })
    }

    fn terminal_width_columns(viewport: RenderViewport) -> Result<u16> {
        let Some(grid_column_last) = viewport.width_cells().checked_sub(1) else {
            return Ok(0);
        };
        let terminal_column_last =
            usize::from(TerminalPosition::project_grid_column(grid_column_last)?);

        terminal_column_last
            .checked_add(1)
            .context("render viewport width exceeds terminal column capacity")?
            .try_into()
            .context("render viewport width exceeds terminal column capacity")
    }

    fn cell_updates(&self, previous: Option<&Self>) -> Vec<TerminalCell> {
        let Some(previous) = previous else {
            return self.cells.clone();
        };
        if previous.width_columns != self.width_columns || previous.height_rows != self.height_rows
        {
            return self.cells.clone();
        }

        self.cells
            .iter()
            .copied()
            .zip(previous.cells.iter().copied())
            .filter_map(|(cell, cell_previous)| (cell != cell_previous).then_some(cell))
            .collect()
    }

    fn cell_index(&self, position: TerminalPosition) -> usize {
        usize::from(position.row()) * usize::from(self.width_columns)
            + usize::from(position.column())
    }

    fn grid_cell_width_columns(&self, position: TerminalPosition) -> usize {
        if position.column() == 0 || position.column() == self.width_columns.saturating_sub(1) {
            1
        } else {
            GRID_CELL_WIDTH_COLUMNS
        }
    }

    const fn viewport(&self) -> RenderViewport {
        self.viewport
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TerminalCell {
    position: TerminalPosition,
    character: char,
    color: TextureColor,
}

impl TerminalCell {
    const fn blank(position: TerminalPosition) -> Self {
        Self {
            position,
            character: ' ',
            color: TextureColor::White,
        }
    }

    fn composite(&mut self, cell: RenderCell) {
        self.character = cell.texture().character(cell.rotation());
        self.color = cell.texture().color();
    }
}

impl<W: Write> CrosstermRenderer<W> {
    fn new(output: W) -> Self {
        Self {
            output,
            terminal_frame_previous: None,
        }
    }

    fn compose_terminal_frame(frame: &RenderFrame) -> Result<TerminalFrame> {
        let mut terminal_frame = TerminalFrame::try_from_render_viewport(frame.viewport())?;

        for cell in frame.cells() {
            Self::composite_render_cell(&mut terminal_frame, *cell)?;
        }

        Ok(terminal_frame)
    }

    fn composite_render_cell(terminal_frame: &mut TerminalFrame, cell: RenderCell) -> Result<()> {
        let position = TerminalPosition::try_from_render_position(
            cell.position_world(),
            terminal_frame.viewport(),
        )?;
        let cell_index = terminal_frame.cell_index(position);
        let width_columns = if cell.fills_cell() {
            terminal_frame.grid_cell_width_columns(position)
        } else {
            1
        };

        for terminal_cell in &mut terminal_frame.cells[cell_index..cell_index + width_columns] {
            terminal_cell.composite(cell);
        }

        Ok(())
    }

    fn encode_terminal_cell_update(output: &mut impl Write, cell: TerminalCell) -> Result<()> {
        let styled_character = cell.character.with(Self::crossterm_color(cell.color));
        queue!(
            output,
            MoveTo(cell.position.column(), cell.position.row()),
            PrintStyledContent(styled_character)
        )
        .context("failed to encode a terminal cell update")
    }

    fn present_terminal_frame(
        &mut self,
        terminal_frame: TerminalFrame,
        cell_updates: Vec<TerminalCell>,
        clear_before_render: bool,
    ) -> Result<()> {
        let mut frame_output = Vec::new();
        queue!(&mut frame_output, BeginSynchronizedUpdate)?;
        if clear_before_render {
            queue!(&mut frame_output, Clear(ClearType::All))?;
        }
        for update in cell_updates {
            Self::encode_terminal_cell_update(&mut frame_output, update)?;
        }
        queue!(&mut frame_output, EndSynchronizedUpdate)?;

        if let Err(error) = self
            .output
            .write_all(&frame_output)
            .and_then(|()| self.output.flush())
        {
            self.terminal_frame_previous = None;
            return Err(error).context("failed to present the terminal frame");
        }

        self.terminal_frame_previous = Some(terminal_frame);
        Ok(())
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
        Self::new(stdout())
    }
}

impl<W: Write> Renderer for CrosstermRenderer<W> {
    fn render(&mut self, frame: &RenderFrame) -> Result<()> {
        let terminal_frame = Self::compose_terminal_frame(frame)?;
        let viewport_changed = self
            .terminal_frame_previous
            .as_ref()
            .is_some_and(|previous| {
                previous.width_columns != terminal_frame.width_columns
                    || previous.height_rows != terminal_frame.height_rows
            });
        let cell_updates = terminal_frame.cell_updates(self.terminal_frame_previous.as_ref());
        if cell_updates.is_empty() && !viewport_changed {
            self.terminal_frame_previous = Some(terminal_frame);
            return Ok(());
        }

        self.present_terminal_frame(terminal_frame, cell_updates, viewport_changed)
    }
}

#[cfg(test)]
mod tests {
    use snake_core::{Rotation, Texture, TextureColor, Vector2Int, ZIndex, assets};

    use super::{CrosstermRenderer, GRID_CELL_WIDTH_COLUMNS, TerminalPosition};
    use crate::render::{RenderCell, RenderFrame, RenderViewport, Renderer};

    const BACKGROUND_TEXTURE: Texture = Texture::new('b', TextureColor::White);

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
        let mut renderer = CrosstermRenderer::new(Vec::new());

        renderer.render(&frame).unwrap();

        assert_eq!(
            count_texture(&renderer.output, assets::APPLE, Rotation::default()),
            1
        );
        assert_eq!(
            count_texture(&renderer.output, assets::SNAKE_PART, Rotation::default()),
            1
        );
        assert!(
            renderer
                .output
                .windows(6)
                .any(|window| window == b"\x1b[2;2H")
        );
    }

    #[test]
    fn identical_frame_is_not_written_twice() {
        let frame = RenderFrame::new(
            RenderViewport::new(1, 1),
            vec![RenderCell::new(
                Vector2Int::new(0, 0),
                assets::APPLE,
                Rotation::default(),
                ZIndex::DEFAULT,
            )],
        );
        let mut renderer = CrosstermRenderer::new(Vec::new());

        renderer.render(&frame).unwrap();
        let output_len_first = renderer.output.len();
        renderer.render(&frame).unwrap();

        assert_eq!(renderer.output.len(), output_len_first);
    }

    #[test]
    fn changed_frame_is_written_as_a_synchronized_update() {
        let frame = RenderFrame::new(
            RenderViewport::new(1, 1),
            vec![RenderCell::new(
                Vector2Int::new(0, 0),
                assets::APPLE,
                Rotation::default(),
                ZIndex::DEFAULT,
            )],
        );
        let mut renderer = CrosstermRenderer::new(Vec::new());

        renderer.render(&frame).unwrap();

        assert!(renderer.output.starts_with(b"\x1b[?2026h"));
        assert!(renderer.output.ends_with(b"\x1b[?2026l"));
    }

    #[test]
    fn overlapped_cells_write_only_the_topmost_visual() {
        let frame = RenderFrame::new(
            RenderViewport::new(1, 1),
            vec![
                filled_render_cell(
                    Vector2Int::new(0, 0),
                    BACKGROUND_TEXTURE,
                    Rotation::default(),
                ),
                RenderCell::new(
                    Vector2Int::new(0, 0),
                    assets::APPLE,
                    Rotation::default(),
                    ZIndex::DEFAULT,
                ),
            ],
        );
        let mut renderer = CrosstermRenderer::new(Vec::new());

        renderer.render(&frame).unwrap();

        assert_eq!(
            count_texture(&renderer.output, BACKGROUND_TEXTURE, Rotation::default()),
            0
        );
        assert_eq!(
            count_texture(&renderer.output, assets::APPLE, Rotation::default()),
            1
        );
    }

    #[test]
    fn moving_object_writes_only_new_and_vacated_cells() {
        let viewport = RenderViewport::new(2, 1);
        let frame_at = |apple_x| {
            RenderFrame::new(
                viewport,
                vec![
                    filled_render_cell(
                        Vector2Int::new(0, 0),
                        BACKGROUND_TEXTURE,
                        Rotation::default(),
                    ),
                    filled_render_cell(
                        Vector2Int::new(1, 0),
                        BACKGROUND_TEXTURE,
                        Rotation::default(),
                    ),
                    RenderCell::new(
                        Vector2Int::new(apple_x, 0),
                        assets::APPLE,
                        Rotation::default(),
                        ZIndex::DEFAULT,
                    ),
                ],
            )
        };
        let mut renderer = CrosstermRenderer::new(Vec::new());
        renderer.render(&frame_at(0)).unwrap();
        renderer.output.clear();

        renderer.render(&frame_at(1)).unwrap();

        assert_eq!(
            count_texture(&renderer.output, BACKGROUND_TEXTURE, Rotation::default()),
            1
        );
        assert_eq!(
            count_texture(&renderer.output, assets::APPLE, Rotation::default()),
            1
        );
    }

    #[test]
    fn smaller_viewport_clears_cells_outside_the_new_frame() {
        let frame_wide = RenderFrame::new(
            RenderViewport::new(2, 1),
            vec![
                filled_render_cell(
                    Vector2Int::new(0, 0),
                    BACKGROUND_TEXTURE,
                    Rotation::default(),
                ),
                filled_render_cell(
                    Vector2Int::new(1, 0),
                    BACKGROUND_TEXTURE,
                    Rotation::default(),
                ),
            ],
        );
        let frame_narrow = RenderFrame::new(
            RenderViewport::new(1, 1),
            vec![filled_render_cell(
                Vector2Int::new(0, 0),
                BACKGROUND_TEXTURE,
                Rotation::default(),
            )],
        );
        let mut renderer = CrosstermRenderer::new(Vec::new());
        renderer.render(&frame_wide).unwrap();
        renderer.output.clear();

        renderer.render(&frame_narrow).unwrap();

        assert!(renderer.output.windows(4).any(|bytes| bytes == b"\x1b[2J"));
    }

    #[test]
    fn empty_viewport_clears_the_previous_frame() {
        let frame_visible = RenderFrame::new(
            RenderViewport::new(1, 1),
            vec![filled_render_cell(
                Vector2Int::new(0, 0),
                BACKGROUND_TEXTURE,
                Rotation::default(),
            )],
        );
        let frame_empty = RenderFrame::new(RenderViewport::new(0, 0), Vec::new());
        let mut renderer = CrosstermRenderer::new(Vec::new());
        renderer.render(&frame_visible).unwrap();
        renderer.output.clear();

        renderer.render(&frame_empty).unwrap();

        assert!(renderer.output.windows(4).any(|bytes| bytes == b"\x1b[2J"));
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
        let mut renderer = CrosstermRenderer::new(Vec::new());

        renderer.render(&frame).unwrap();

        assert_eq!(
            count_texture(&renderer.output, assets::WALL_DIAGONAL, Rotation::default()),
            2
        );
        assert_eq!(
            count_texture(&renderer.output, assets::WALL, Rotation::DOWN),
            GRID_CELL_WIDTH_COLUMNS
        );
    }

    #[test]
    fn terminal_columns_follow_projected_cell_widths() {
        let cases = [(0, 0), (1, 1), (22, 43), (23, 45)];

        for (grid_x, terminal_column) in cases {
            assert_eq!(
                TerminalPosition::project_grid_column(grid_x).unwrap(),
                terminal_column
            );
        }
    }

    #[test]
    fn render_rejects_positions_outside_the_viewport() {
        let viewport = RenderViewport::new(24, 24);
        for (position, error_expected) in [
            (
                Vector2Int::new(-1, 0),
                "render cell x position -1 cannot be represented in the terminal grid",
            ),
            (
                Vector2Int::new(24, 0),
                "render cell x position 24 is outside the viewport",
            ),
            (
                Vector2Int::new(0, -1),
                "render cell y position -1 cannot be represented in the terminal grid",
            ),
            (
                Vector2Int::new(0, 24),
                "render cell y position 24 is outside the viewport",
            ),
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
            let mut renderer = CrosstermRenderer::new(Vec::new());

            let error = renderer.render(&frame).unwrap_err();

            assert_eq!(error.to_string(), error_expected);
        }
    }

    fn filled_render_cell(
        position_world: Vector2Int,
        texture: Texture,
        rotation: Rotation,
    ) -> RenderCell {
        RenderCell::from_filled_cell(position_world, texture, rotation, ZIndex::BACKGROUND)
    }

    fn count_texture(bytes: &[u8], texture: Texture, rotation: Rotation) -> usize {
        std::str::from_utf8(bytes)
            .expect("Crossterm test output should be valid UTF-8")
            .chars()
            .filter(|character| *character == texture.character(rotation))
            .count()
    }
}
