use gpui_kit::{Context, IntoElement, Render, Window, div};

/// What a dragged column edge shows while it moves: nothing.
pub(crate) struct Ghost;

impl Render for Ghost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}
