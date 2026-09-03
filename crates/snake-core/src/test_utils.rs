use crate::{Render, RenderItem};

pub(crate) fn collect_render_items(renderable: &dyn Render) -> Vec<RenderItem> {
    let mut items = Vec::new();
    renderable.visit_render_items(&mut |item| items.push(item));
    items
}
