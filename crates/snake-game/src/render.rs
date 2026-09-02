use std::io;

use snake_core::{Render, RenderItem, Texture, Vector2Int, ZIndex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RenderViewport {
    width_cells: usize,
    height_cells: usize,
}

impl RenderViewport {
    pub(crate) const fn new(width_cells: usize, height_cells: usize) -> Self {
        Self {
            width_cells,
            height_cells,
        }
    }

    pub(crate) const fn width_cells(self) -> usize {
        self.width_cells
    }

    pub(crate) const fn height_cells(self) -> usize {
        self.height_cells
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RenderCell {
    position_world: Vector2Int,
    texture: Texture,
    z_index: ZIndex,
    fills_cell: bool,
}

impl RenderCell {
    #[cfg(test)]
    pub(crate) const fn new(position_world: Vector2Int, texture: Texture, z_index: ZIndex) -> Self {
        Self {
            position_world,
            texture,
            z_index,
            fills_cell: false,
        }
    }

    fn from_render_item(position_origin: Vector2Int, item: RenderItem) -> Self {
        Self {
            position_world: position_origin + item.position_local(),
            texture: item.texture(),
            z_index: item.z_index(),
            fills_cell: item.fills_cell(),
        }
    }

    #[cfg(test)]
    pub(crate) const fn from_filled_cell(
        position_world: Vector2Int,
        texture: Texture,
        z_index: ZIndex,
    ) -> Self {
        Self {
            position_world,
            texture,
            z_index,
            fills_cell: true,
        }
    }

    pub(crate) const fn position_world(self) -> Vector2Int {
        self.position_world
    }

    pub(crate) const fn texture(self) -> Texture {
        self.texture
    }

    pub(crate) const fn fills_cell(self) -> bool {
        self.fills_cell
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RenderFrame {
    viewport: RenderViewport,
    cells: Vec<RenderCell>,
}

impl RenderFrame {
    pub(crate) fn new(viewport: RenderViewport, mut cells: Vec<RenderCell>) -> Self {
        cells.sort_by_key(|cell| cell.z_index);
        Self { viewport, cells }
    }

    pub(crate) fn cells(&self) -> &[RenderCell] {
        &self.cells
    }

    pub(crate) const fn viewport(&self) -> RenderViewport {
        self.viewport
    }
}

pub(crate) fn append_render_cells(
    cells: &mut Vec<RenderCell>,
    position_origin: Vector2Int,
    renderable: &dyn Render,
) {
    renderable.visit_render_items(&mut |item| {
        cells.push(RenderCell::from_render_item(position_origin, item));
    });
}

pub(crate) trait Renderer {
    fn render(&mut self, frame: &RenderFrame) -> io::Result<()>;
}

#[cfg(test)]
mod tests {
    use snake_core::{Texture, TextureColor, Vector2Int, ZIndex};

    use super::{RenderCell, RenderFrame, RenderViewport};

    #[test]
    fn frame_orders_cells_by_z_index_and_preserves_equal_order() {
        let frame = RenderFrame::new(
            RenderViewport::new(24, 24),
            vec![
                render_cell('h', ZIndex::DEFAULT),
                render_cell('b', ZIndex::BACKGROUND),
                render_cell('s', ZIndex::DEFAULT),
            ],
        );

        assert_eq!(
            frame
                .cells()
                .iter()
                .map(|cell| cell.texture().character())
                .collect::<Vec<_>>(),
            ['b', 'h', 's']
        );
    }

    fn render_cell(character: char, z_index: ZIndex) -> RenderCell {
        RenderCell::new(
            Vector2Int::default(),
            Texture::new(character, TextureColor::White),
            z_index,
        )
    }
}
