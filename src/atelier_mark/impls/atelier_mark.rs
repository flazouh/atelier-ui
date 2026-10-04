use gpui_kit::{
    App, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window, div, linear_color_stop, linear_gradient, svg,
};

use crate::{scale::px, theme::ActiveTheme};

use super::super::{
    consts::{ASPECT, CORNER, GLOW_ALPHA, GLOW_RISE, PATH, WIDTH},
    enums::MarkLook,
    structs::{AtelierMark, Tile},
};

impl AtelierMark {
    pub fn new(size: f32) -> Self {
        Self { size, look: None, accent: None }
    }

    /// Draws this look whatever the theme.
    pub fn look(mut self, look: MarkLook) -> Self {
        self.look = Some(look);
        self
    }

    /// Colours the mark with `accent` in place of terracotta: the glow of the Halo, the tile of the Terracotta.
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
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let look = self.look.unwrap_or_else(|| MarkLook::of(cx.theme().appearance));
        let tile = Tile::of(look, self.accent);
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
            .children(tile.glow.map(|glow| {
                div().absolute().bottom_0().left_0().right_0().h(px(size * GLOW_RISE)).bg(linear_gradient(
                    180.,
                    linear_color_stop(glow.opacity(0.), 0.),
                    linear_color_stop(glow.opacity(GLOW_ALPHA), 1.),
                ))
            }))
            .children(tile.edge.map(|edge| div().absolute().inset_0().rounded(px(corner)).border_1().border_color(edge)))
            .child(
                div()
                    .debug_selector(|| "atelier-mark-a".into())
                    .relative()
                    .flex_none()
                    .w(px(width))
                    .h(px(width / ASPECT))
                    .child(svg().path(PATH).size_full().text_color(tile.ink)),
            )
    }
}
