use crate::{
    Render, RenderItem,
    models::snake::{Snake, SnakeSignal},
    signal::SignalBusBuilder,
};

pub(crate) fn collect_render_items(renderable: &dyn Render) -> Vec<RenderItem> {
    let mut items = Vec::new();
    renderable.visit_render_items(&mut |item| items.push(item));
    items
}

pub(crate) fn spawn_snake() -> Snake {
    let mut signals = SignalBusBuilder::<()>::new();
    let emitter = signals.register::<SnakeSignal>().unwrap();
    Snake::spawn(emitter)
}
