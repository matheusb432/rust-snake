use crate::{Render, RenderItem, models::snake::Snake, signal::SignalBus};

pub(crate) fn collect_render_items(renderable: &dyn Render) -> Vec<RenderItem> {
    let mut items = Vec::new();
    renderable.visit_render_items(&mut |item| items.push(item));
    items
}

pub(crate) fn spawn_snake() -> Snake {
    let emitter = SignalBus::new().emitter();
    Snake::spawn(emitter)
}
