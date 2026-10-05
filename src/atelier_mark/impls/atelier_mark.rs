use gpui_kit::{
    App, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window, div, linear_color_stop, linear_gradient, svg,
};

use crate::scale::px;

use super::super::{
    consts::{ASPECT, CORNER, PATH, WIDTH},
    structs::{AtelierMark, Tile},
};

impl AtelierMark {
    pub fn new(size: f32) -> Self {
        Self { size, accent: None }
    }

    /// Colours the tile with `accent` in place of terracotta.
    pub fn accent(mut self, accent: impl Into<Hsla>) -> Self {
        self.accent = Some(accent.into());
        self
    }

    /// The "A", for [`crate::icon::Assets`] to serve.
    pub(crate) fn bytes(path: &str) -> Option<&'static [u8]> {
        (path == PATH).then_some(include_bytes!("../../../assets/atelier-mark.svg"))
    }
}

impl RenderOnce for AtelierMark {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let tile = Tile::of(self.accent);
        let (size, corner) = (self.size, self.size * CORNER);
        let width = size * WIDTH;
        div()
            .debug_selector(|| "atelier-mark".into())
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(size))
            .rounded(px(corner))
            .overflow_hidden()
            .bg(linear_gradient(180., linear_color_stop(tile.top, 0.), linear_color_stop(tile.bottom, 1.)))
            .child(
                div()
                    .debug_selector(|| "atelier-mark-a".into())
                    .relative()
                    .flex_none()
                    .w(px(width))
                    .h(px(width / ASPECT))
                    .child(svg().path(PATH).size_full().text_color(tile.letter)),
            )
    }
}
